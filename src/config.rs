use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api:      ApiConfig,
    pub audio:    AudioConfig,
    pub playback: PlaybackConfig,
    pub shuffle:  ShuffleConfig,
    pub repeat:   RepeatConfig,
    pub ui:       UiConfig,
}

fn default_country_code() -> String { "US".to_string() }
fn default_redirect_port() -> u16   { 8080 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub client_id:     String,
    pub client_secret: String,
    /// ISO 3166-1 alpha-2 country code — required on every TIDAL API request.
    /// Defaults to "US" if omitted from the config file.
    #[serde(default = "default_country_code")]
    pub country_code:  String,
    /// localhost port for the PKCE OAuth2 redirect server.
    /// Register http://localhost:<port>/callback in the TIDAL developer dashboard.
    /// Defaults to 8080 if omitted from the config file.
    #[serde(default = "default_redirect_port")]
    pub redirect_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// LOW | HIGH | LOSSLESS | HI_RES
    pub quality:        String,
    pub device:         String,
    pub exclusive_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackConfig {
    pub gapless:           bool,
    pub crossfade_seconds: u8,
    pub crossfade_enabled: bool,
    /// album | track | off
    pub replaygain_mode:   String,
    pub replaygain_preamp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShuffleConfig {
    /// off | random | favourites | discovery
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatConfig {
    /// off | one | all
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme:           String,
    pub artwork_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        // Embed the default config at compile time — no runtime file I/O.
        toml::from_str(include_str!("../config/default.toml"))
            .expect("config/default.toml is always valid")
    }
}

impl Config {
    /// Returns `~/.config/tidal-tui/config.toml`.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tidal-tui")
            .join("config.toml")
    }

    /// Load user config from disk, falling back to defaults for missing keys.
    /// Creates the config directory and writes a default file on first run.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, include_str!("../config/default.toml"))?;
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&raw)
            .map_err(|e| AppError::Config(e.to_string()))?;
        Ok(config)
    }
}
