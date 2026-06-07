mod api;
mod app;
mod cache;
mod config;
mod errors;
mod player;
mod tui;

use tracing_subscriber::{EnvFilter, fmt};

fn main() -> anyhow::Result<()> {
    // Initialise logging (controlled by RUST_LOG env var).
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Load configuration (creates default if missing).
    let config = config::Config::load()?;
    tracing::debug!(?config, "Configuration loaded");

    // --- CLI flags (checked before entering the TUI) ---
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--login".to_string()) {
        // Run the interactive PKCE login flow and exit.
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(async {
            let client = api::TidalClient::new(config.api.clone());
            client.login_interactive().await.map_err(anyhow::Error::from)
        });
    }

    if args.contains(&"--debug-api".to_string()) {
        // Exercise every API module against the live TIDAL API and print results.
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(debug_api_run(config));
    }

    if let Some(pos) = args.iter().position(|arg| arg == "--play-url") {
        let url = args
            .get(pos + 1)
            .ok_or_else(|| anyhow::anyhow!("--play-url requires a URL argument"))?
            .clone();
        return play_url_test(config, url);
    }

    // --- Normal TUI path ---
    let mut terminal = ratatui::init();
    let result = app::App::new(config).run(&mut terminal);
    ratatui::restore();
    result.map_err(anyhow::Error::from)
}

// ---------------------------------------------------------------------------
// --debug-api: exercises every API module, prints results to stdout.
// Requires client_id / client_secret in the config file.
// ---------------------------------------------------------------------------

async fn debug_api_run(config: config::Config) -> anyhow::Result<()> {
    use api::{catalog, search, TidalClient};

    println!("=== tidal-tui --debug-api ===\n");

    let client = TidalClient::new(config.api.clone());

    // Load any persisted tokens from disk first.
    if let Err(e) = client.load_persisted_tokens().await {
        eprintln!("Warning: could not load persisted tokens: {e}");
    }

    // 1. Acquire / validate token.
    let token = client.ensure_token().await?;
    println!(
        "[auth]  Token acquired: {}...{}",
        &token[..8.min(token.len())],
        &token[token.len().saturating_sub(4)..]
    );

    // 2. Fetch a well-known track: Daft Punk — One More Time (id 59978679).
    println!("\n[catalog] Fetching track 59978679 ...");
    match catalog::get_track(&client, "59978679").await {
        Ok(t) => println!("  Track : {} ({}s)", t.attributes.title, t.attributes.duration),
        Err(e) => eprintln!("  Error : {e}"),
    }

    // 3. Fetch the corresponding album: Random Access Memories (id 59727856).
    println!("\n[catalog] Fetching album 59727856 ...");
    match catalog::get_album(&client, "59727856").await {
        Ok(a) => println!("  Album : {}", a.attributes.title),
        Err(e) => eprintln!("  Error : {e}"),
    }

    // 4. Album items.
    println!("\n[catalog] Fetching album items for 59727856 ...");
    match catalog::get_album_items(&client, "59727856").await {
        Ok(items) => println!("  Items : {} tracks", items.len()),
        Err(e)    => eprintln!("  Error : {e}"),
    }

    // 5. Artist: Daft Punk (id 16810799).
    println!("\n[catalog] Fetching artist 16810799 ...");
    match catalog::get_artist(&client, "16810799").await {
        Ok(a) => println!("  Artist: {}", a.attributes.name),
        Err(e) => eprintln!("  Error : {e}"),
    }

    // 6. Search.
    println!("\n[search] Searching for 'daft punk' (limit 5) ...");
    match search::search(&client, "daft punk", 5).await {
        Ok(r) => {
            println!("  Tracks  : {}", r.tracks.len());
            println!("  Albums  : {}", r.albums.len());
            println!("  Artists : {}", r.artists.len());
            for t in r.tracks.iter().take(3) {
                println!("    - {} (id={})", t.attributes.title, t.id);
            }
        }
        Err(e) => eprintln!("  Error : {e}"),
    }

    println!("\n=== debug-api complete ===");
    Ok(())
}

fn play_url_test(config: config::Config, url: String) -> anyhow::Result<()> {
    use std::{io, sync::mpsc, thread, time::Duration};

    use player::{spawn_player_thread, AudioCommand, PlayerEvent, QueueEntry};

    if looks_like_tidal_web_url(&url) {
        return Err(anyhow::anyhow!(
            "--play-url expects a direct audio stream URL, not a TIDAL web page URL. \
Use a raw media URL that mpv can open directly. Example: an .mp3/.flac stream URL or \
the resolved playback URL returned by the TIDAL API playback endpoint."
        ));
    }

    println!("Playing: {url}");
    println!("Press Enter to stop.");

    let handle = spawn_player_thread(
        config.playback.clone(),
        config.shuffle.clone(),
        config.repeat.clone(),
    );
    let entry = QueueEntry {
        track_id: "test".into(),
        title: "Test Track".into(),
        artist: "Unknown".into(),
        album: "Unknown".into(),
        duration: 0,
        stream_url: url.clone(),
    };
    handle.send(AudioCommand::Play { url, entry })?;

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let _ = stop_tx.send(());
    });

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        while let Some(event) = handle.try_recv_event() {
            match event {
                PlayerEvent::TrackStarted { track_id } => {
                    println!("[player] Track started: {track_id}");
                }
                PlayerEvent::PositionChanged(pos) => {
                    print!("\r[player] Position: {:.1}s   ", pos);
                }
                PlayerEvent::DurationChanged(dur) => {
                    println!("\n[player] Duration: {:.1}s", dur);
                }
                PlayerEvent::Paused => {
                    println!("\n[player] Paused");
                }
                PlayerEvent::Resumed => {
                    println!("\n[player] Resumed");
                }
                PlayerEvent::VolumeChanged(vol) => {
                    println!("\n[player] Volume: {vol}");
                }
                PlayerEvent::RepeatModeChanged(mode) => {
                    println!("\n[player] Repeat: {mode:?}");
                }
                PlayerEvent::ShuffleModeChanged(mode) => {
                    println!("\n[player] Shuffle: {mode:?}");
                }
                PlayerEvent::CrossfadeChanged { enabled, seconds } => {
                    println!("\n[player] Crossfade: enabled={enabled} seconds={seconds}");
                }
                PlayerEvent::QueueRefillRequested { mode, based_on_track_id } => {
                    println!(
                        "\n[player] Queue refill requested: {mode:?} based_on={based_on_track_id:?}"
                    );
                }
                PlayerEvent::TrackEnded { track_id, reason } => {
                    println!("\n[player] Track ended: {track_id} ({reason:?})");
                }
                PlayerEvent::QueueEmpty => {
                    println!("\n[player] Queue empty");
                }
                PlayerEvent::Error(error) => {
                    eprintln!("\n[player] Error: {error}");
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    println!("\nStopping playback...");
    let _ = handle.send(AudioCommand::Shutdown);
    thread::sleep(Duration::from_millis(100));
    Ok(())
}

fn looks_like_tidal_web_url(url: &str) -> bool {
    url.contains("tidal.com/track/") || url.contains("listen.tidal.com/")
}
