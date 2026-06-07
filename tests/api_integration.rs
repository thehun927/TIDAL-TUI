//! Live integration tests against the TIDAL API.
//!
//! These are marked `#[ignore]` and require:
//!   - A valid `client_id` + `client_secret` in ~/.config/tidal-tui/config.toml
//!   - An internet connection
//!
//! Run with:
//!   cargo test --test api_integration -- --ignored --nocapture

use tidal_tui::api::TidalClient;
use tidal_tui::config::Config;

fn make_client() -> TidalClient {
    let config = Config::load().expect("Config must load");
    TidalClient::new(config.api)
}

#[tokio::test]
#[ignore]
async fn fetch_known_track() {
    let client = make_client();
    client.load_persisted_tokens().await.unwrap();
    let track = tidal_tui::api::catalog::get_track(&client, "59978679").await.unwrap();
    assert!(!track.attributes.title.is_empty(), "Track title should not be empty");
    assert!(track.attributes.duration > 0, "Duration should be positive");
}

#[tokio::test]
#[ignore]
async fn fetch_known_album() {
    let client = make_client();
    client.load_persisted_tokens().await.unwrap();
    let album = tidal_tui::api::catalog::get_album(&client, "59727856").await.unwrap();
    assert!(!album.attributes.title.is_empty());
}

#[tokio::test]
#[ignore]
async fn fetch_album_items() {
    let client = make_client();
    client.load_persisted_tokens().await.unwrap();
    let items = tidal_tui::api::catalog::get_album_items(&client, "59727856").await.unwrap();
    assert!(!items.is_empty(), "Album should have at least one track");
}

#[tokio::test]
#[ignore]
async fn fetch_known_artist() {
    let client = make_client();
    client.load_persisted_tokens().await.unwrap();
    let artist = tidal_tui::api::catalog::get_artist(&client, "16810799").await.unwrap();
    assert!(!artist.attributes.name.is_empty());
}

#[tokio::test]
#[ignore]
async fn search_returns_results() {
    let client = make_client();
    client.load_persisted_tokens().await.unwrap();
    let results = tidal_tui::api::search::search(&client, "daft punk", 5).await.unwrap();
    assert!(
        !results.tracks.is_empty() || !results.albums.is_empty(),
        "Search for 'daft punk' should return at least one result"
    );
}
