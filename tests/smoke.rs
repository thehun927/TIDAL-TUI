use tidal_tui::config::Config;

#[test]
fn default_config_parses() {
    let config = Config::default();
    assert_eq!(config.audio.quality, "LOSSLESS");
    assert_eq!(config.repeat.mode, "off");
    assert_eq!(config.shuffle.mode, "off");
    assert!(config.playback.gapless);
    assert!(!config.playback.crossfade_enabled);
    assert_eq!(config.ui.theme, "dark");
    assert!(config.ui.artwork_enabled);
}

#[test]
fn config_path_contains_tidal_tui() {
    let path = Config::config_path();
    assert!(path.to_string_lossy().contains("tidal-tui"));
}
