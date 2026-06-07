use std::sync::mpsc;
use std::thread;

use mpv::{EndFileReason, Event, Format, MpvHandlerBuilder};
use rand::thread_rng;

use crate::{
    config::{PlaybackConfig, RepeatConfig, ShuffleConfig},
    player::{
        commands::AudioCommand,
        events::{EndReason, PlayerEvent},
        queue::PlaybackQueue,
        repeat::RepeatMode,
        shuffle::ShuffleMode,
    },
};

// ---------------------------------------------------------------------------
// Property observe IDs (arbitrary u32 constants used to identify observations)
// ---------------------------------------------------------------------------
const OBS_TIME_POS: u32 = 1;
const OBS_DURATION: u32 = 2;
const OBS_VOLUME:   u32 = 3;
const OBS_PAUSE:    u32 = 4;

#[derive(Debug, Clone)]
struct PlayerState {
    repeat_mode:       RepeatMode,
    shuffle_mode:      ShuffleMode,
    crossfade_enabled: bool,
    crossfade_seconds: u8,
}

// ---------------------------------------------------------------------------
// Public handle returned to the main application
// ---------------------------------------------------------------------------

/// Owned by the main thread. Wraps both channel ends for the player.
pub struct PlayerHandle {
    pub cmd_tx:   mpsc::SyncSender<AudioCommand>,
    pub event_rx: mpsc::Receiver<PlayerEvent>,
}

impl PlayerHandle {
    /// Send a command to the MPV thread. Non-blocking up to the channel capacity (32).
    pub fn send(&self, cmd: AudioCommand) -> Result<(), mpsc::SendError<AudioCommand>> {
        self.cmd_tx.send(cmd)
    }

    /// Poll for a player event without blocking.
    pub fn try_recv_event(&self) -> Option<PlayerEvent> {
        self.event_rx.try_recv().ok()
    }
}

// ---------------------------------------------------------------------------
// Thread spawner
// ---------------------------------------------------------------------------

/// Spawn the dedicated MPV OS thread and return a `PlayerHandle`.
///
/// `MpvHandler` holds a `*mut mpv_handle` raw pointer and is therefore `!Send`.
/// It is created and kept entirely inside this thread — it never crosses a thread boundary.
pub fn spawn_player_thread(
    playback: PlaybackConfig,
    shuffle: ShuffleConfig,
    repeat: RepeatConfig,
) -> PlayerHandle {
    let (cmd_tx, cmd_rx)     = mpsc::sync_channel::<AudioCommand>(32);
    let (event_tx, event_rx) = mpsc::sync_channel::<PlayerEvent>(64);

    thread::Builder::new()
        .name("mpv-player".into())
        .spawn(move || {
            if let Err(e) = run_player_thread(playback, shuffle, repeat, cmd_rx, event_tx) {
                eprintln!("[mpv] Player thread exited with error: {e}");
            }
        })
        .expect("Failed to spawn MPV player thread");

    PlayerHandle { cmd_tx, event_rx }
}

// ---------------------------------------------------------------------------
// Player thread entry point
// ---------------------------------------------------------------------------

fn run_player_thread(
    config:   PlaybackConfig,
    shuffle:  ShuffleConfig,
    repeat:   RepeatConfig,
    cmd_rx:   mpsc::Receiver<AudioCommand>,
    event_tx: mpsc::SyncSender<PlayerEvent>,
) -> Result<(), Box<dyn std::error::Error>> {

    // Build MpvHandler — audio-only, no video window.
    let mpv = build_mpv(&config)?;
    let mut mpv = mpv;

    // Observe properties for continuous status updates.
    mpv.observe_property::<f64>("time-pos",  OBS_TIME_POS)?;
    mpv.observe_property::<f64>("duration",  OBS_DURATION)?;
    mpv.observe_property::<f64>("volume",    OBS_VOLUME)?;
    mpv.observe_property::<bool>("pause",    OBS_PAUSE)?;

    let mut queue            = PlaybackQueue::new();
    let mut current_track_id = String::new();
    let mut running          = true;
    let mut state = PlayerState {
        repeat_mode: repeat_mode_from_config(&repeat.mode),
        shuffle_mode: shuffle_mode_from_config(&shuffle.mode),
        crossfade_enabled: config.crossfade_enabled,
        crossfade_seconds: config.crossfade_seconds,
    };

    let _ = crate::player::crossfade::set_crossfade(
        &mut mpv,
        if state.crossfade_enabled { state.crossfade_seconds } else { 0 },
    );

    while running {
        // Drain all pending commands before processing the next mpv event.
        loop {
            match cmd_rx.try_recv() {
                Ok(cmd) => {
                    if let Err(e) = handle_command(
                        cmd,
                        &mut mpv,
                        &mut queue,
                        &mut state,
                        &event_tx,
                        &mut current_track_id,
                        &mut running,
                    ) {
                        let _ = event_tx.send(PlayerEvent::Error(e.to_string()));
                    }
                }
                Err(mpsc::TryRecvError::Empty)        => break,
                Err(mpsc::TryRecvError::Disconnected) => { running = false; break; }
            }
        }

        // Wait for the next mpv event with a 16 ms timeout (~60 fps responsiveness).
        if let Some(event) = mpv.wait_event(0.016) {
            handle_mpv_event(
                event,
                &mut mpv,
                &mut queue,
                &state,
                &event_tx,
                &mut current_track_id,
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// MpvHandler builder with audio-output fallback
// ---------------------------------------------------------------------------

fn build_mpv(config: &PlaybackConfig) -> Result<mpv::MpvHandler, Box<dyn std::error::Error>> {
    // Try the preferred audio outputs in order: alsa → pulse → auto.
    for ao in &["alsa", "pulse", "auto"] {
        if let Ok(mpv) = try_build_mpv(config, ao) {
            tracing::debug!("mpv initialised with ao={ao}");
            return Ok(mpv);
        }
    }
    Err("Failed to initialise mpv with any audio output (tried alsa, pulse, auto)".into())
}

fn try_build_mpv(config: &PlaybackConfig, ao: &str) -> Result<mpv::MpvHandler, Box<dyn std::error::Error>> {
    let mut b = MpvHandlerBuilder::new()?;
    b.set_option("vo", "null")?;          // No video window in a terminal app.
    b.set_option("ao", ao)?;
    b.set_option("gapless-audio", if config.gapless { "yes" } else { "no" })?;
    b.set_option("replaygain",    config.replaygain_mode.as_str())?;
    let preamp = format!("{:.1}", config.replaygain_preamp);
    b.set_option("replaygain-preamp", preamp.as_str())?;
    let mpv = b.build()?;
    Ok(mpv)
}

// ---------------------------------------------------------------------------
// Command handler
// ---------------------------------------------------------------------------

fn handle_command(
    cmd:              AudioCommand,
    mpv:              &mut mpv::MpvHandler,
    queue:            &mut PlaybackQueue,
    state:            &mut PlayerState,
    event_tx:         &mpsc::SyncSender<PlayerEvent>,
    current_track_id: &mut String,
    running:          &mut bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use AudioCommand::*;
    match cmd {
        Play { url, entry } => {
            *current_track_id = entry.track_id.clone();
            queue.current = Some(entry);
            mpv.command(&["loadfile", &url, "replace"])?;
        }

        Pause  => { mpv.set_property("pause", true)?;  }
        Resume => { mpv.set_property("pause", false)?; }

        TogglePause => {
            let paused: bool = mpv.get_property("pause").unwrap_or(false);
            mpv.set_property("pause", !paused)?;
        }

        Seek(delta) => {
            mpv.command(&["seek", &delta.to_string(), "relative"])?;
        }

        SeekAbsolute(pos) => {
            mpv.command(&["seek", &pos.to_string(), "absolute"])?;
        }

        SetVolume(vol) => {
            let v = vol.min(100) as f64;
            mpv.set_property("volume", v)?;
        }

        Next => {
            if let Some(next) = queue.advance() {
                let url = next.stream_url.clone();
                *current_track_id = next.track_id.clone();
                mpv.command(&["loadfile", &url, "replace"])?;
            } else {
                mpv.command(&["stop"])?;
                emit_refill_or_empty(state, event_tx, current_track_id);
            }
        }

        Prev => {
            if let Some(prev) = queue.step_back() {
                let url = prev.stream_url.clone();
                *current_track_id = prev.track_id.clone();
                mpv.command(&["loadfile", &url, "replace"])?;
            }
        }

        Stop => {
            mpv.command(&["stop"])?;
        }

        Enqueue(entry) => {
            queue.upcoming.push_back(entry);
            if state.shuffle_mode == ShuffleMode::Random {
                queue.shuffle_upcoming(&mut thread_rng());
            }
        }

        ClearQueue => {
            queue.upcoming.clear();
        }

        SetRepeatMode(mode) => {
            state.repeat_mode = mode;
            let _ = event_tx.send(PlayerEvent::RepeatModeChanged(state.repeat_mode));
        }

        ToggleRepeatMode => {
            state.repeat_mode = state.repeat_mode.next();
            let _ = event_tx.send(PlayerEvent::RepeatModeChanged(state.repeat_mode));
        }

        SetShuffleMode(mode) => {
            state.shuffle_mode = mode;
            if state.shuffle_mode == ShuffleMode::Random {
                queue.shuffle_upcoming(&mut thread_rng());
            }
            let _ = event_tx.send(PlayerEvent::ShuffleModeChanged(state.shuffle_mode));
        }

        ToggleShuffleMode => {
            state.shuffle_mode = state.shuffle_mode.next();
            if state.shuffle_mode == ShuffleMode::Random {
                queue.shuffle_upcoming(&mut thread_rng());
            }
            let _ = event_tx.send(PlayerEvent::ShuffleModeChanged(state.shuffle_mode));
        }

        SetCrossfadeEnabled(enabled) => {
            state.crossfade_enabled = enabled;
            let _ = crate::player::crossfade::set_crossfade(
                mpv,
                if enabled { state.crossfade_seconds } else { 0 },
            );
            let _ = event_tx.send(PlayerEvent::CrossfadeChanged {
                enabled: state.crossfade_enabled,
                seconds: state.crossfade_seconds,
            });
        }

        SetCrossfadeSeconds(seconds) => {
            state.crossfade_seconds = seconds;
            let _ = crate::player::crossfade::set_crossfade(
                mpv,
                if state.crossfade_enabled { seconds } else { 0 },
            );
            let _ = event_tx.send(PlayerEvent::CrossfadeChanged {
                enabled: state.crossfade_enabled,
                seconds: state.crossfade_seconds,
            });
        }

        Shutdown => {
            *running = false;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MPV event handler
// ---------------------------------------------------------------------------

fn handle_mpv_event(
    event:            Event<'_>,
    mpv:              &mut mpv::MpvHandler,
    queue:            &mut PlaybackQueue,
    state:            &PlayerState,
    event_tx:         &mpsc::SyncSender<PlayerEvent>,
    current_track_id: &mut String,
) {
    match event {
        Event::StartFile => {
            let _ = event_tx.send(PlayerEvent::TrackStarted {
                track_id: current_track_id.clone(),
            });
        }

        Event::EndFile(result) => {
            let reason = match &result {
                Ok(EndFileReason::MPV_END_FILE_REASON_EOF) => EndReason::Eof,
                Ok(EndFileReason::MPV_END_FILE_REASON_STOP)
                | Ok(EndFileReason::MPV_END_FILE_REASON_QUIT) => EndReason::Stopped,
                _ => EndReason::Error,
            };

            let track_id = current_track_id.clone();

            if let Err(error) = &result {
                let _ = event_tx.send(PlayerEvent::Error(format!(
                    "Playback failed: {error}"
                )));
            }

            if reason == EndReason::Eof {
                match state.repeat_mode {
                    RepeatMode::Off => {
                        if queue.advance().is_some() {
                            load_current_entry(mpv, queue, current_track_id);
                        } else {
                            emit_refill_or_empty(state, event_tx, current_track_id);
                        }
                    }
                    RepeatMode::One => {
                        load_current_entry(mpv, queue, current_track_id);
                    }
                    RepeatMode::All => {
                        if queue.advance().is_some() {
                            load_current_entry(mpv, queue, current_track_id);
                        } else if queue.restart_cycle().is_some() {
                            load_current_entry(mpv, queue, current_track_id);
                        } else {
                            emit_refill_or_empty(state, event_tx, current_track_id);
                        }
                    }
                }
            }

            let _ = event_tx.send(PlayerEvent::TrackEnded { track_id, reason });
        }

        Event::Pause   => { let _ = event_tx.send(PlayerEvent::Paused);  }
        Event::Unpause => { let _ = event_tx.send(PlayerEvent::Resumed); }

        Event::Idle => {
            if queue.upcoming.is_empty() && queue.current.is_none() {
                let _ = event_tx.send(PlayerEvent::QueueEmpty);
            }
        }

        Event::PropertyChange { name, change, .. } => {
            match name {
                "time-pos" => {
                    if let Format::Double(pos) = change {
                        let _ = event_tx.send(PlayerEvent::PositionChanged(pos));
                    }
                }
                "duration" => {
                    if let Format::Double(dur) = change {
                        let _ = event_tx.send(PlayerEvent::DurationChanged(dur));
                    }
                }
                "volume" => {
                    if let Format::Double(v) = change {
                        let _ = event_tx.send(PlayerEvent::VolumeChanged(v as u32));
                    }
                }
                _ => {}
            }
        }

        // Graceful shutdown — the loop exits via the Shutdown AudioCommand.
        Event::Shutdown => {}
        _ => {}
    }
}

fn load_current_entry(
    mpv: &mut mpv::MpvHandler,
    queue: &PlaybackQueue,
    current_track_id: &mut String,
) {
    if let Some(current) = queue.current.as_ref() {
        *current_track_id = current.track_id.clone();
        let _ = mpv.command(&["loadfile", &current.stream_url, "replace"]);
    }
}

fn emit_refill_or_empty(
    state: &PlayerState,
    event_tx: &mpsc::SyncSender<PlayerEvent>,
    current_track_id: &str,
) {
    match state.shuffle_mode {
        ShuffleMode::Favourites => {
            let _ = event_tx.send(PlayerEvent::QueueRefillRequested {
                mode: ShuffleMode::Favourites,
                based_on_track_id: None,
            });
        }
        ShuffleMode::Discovery => {
            let _ = event_tx.send(PlayerEvent::QueueRefillRequested {
                mode: ShuffleMode::Discovery,
                based_on_track_id: Some(current_track_id.to_string()),
            });
        }
        _ => {
            let _ = event_tx.send(PlayerEvent::QueueEmpty);
        }
    }
}

fn repeat_mode_from_config(mode: &str) -> RepeatMode {
    match mode.to_ascii_lowercase().as_str() {
        "one" => RepeatMode::One,
        "all" => RepeatMode::All,
        _ => RepeatMode::Off,
    }
}

fn shuffle_mode_from_config(mode: &str) -> ShuffleMode {
    match mode.to_ascii_lowercase().as_str() {
        "random" => ShuffleMode::Random,
        "favourites" => ShuffleMode::Favourites,
        "discovery" => ShuffleMode::Discovery,
        _ => ShuffleMode::Off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_mode_from_config_parses_expected_values() {
        assert_eq!(repeat_mode_from_config("off"), RepeatMode::Off);
        assert_eq!(repeat_mode_from_config("one"), RepeatMode::One);
        assert_eq!(repeat_mode_from_config("all"), RepeatMode::All);
    }

    #[test]
    fn shuffle_mode_from_config_parses_expected_values() {
        assert_eq!(shuffle_mode_from_config("off"), ShuffleMode::Off);
        assert_eq!(shuffle_mode_from_config("random"), ShuffleMode::Random);
        assert_eq!(shuffle_mode_from_config("favourites"), ShuffleMode::Favourites);
        assert_eq!(shuffle_mode_from_config("discovery"), ShuffleMode::Discovery);
    }
}
