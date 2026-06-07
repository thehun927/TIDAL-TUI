use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("TIDAL API error: {status} — {message}")]
    Api { status: u16, message: String },

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Player error: {0}")]
    Player(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Image error: {0}")]
    Image(String),

    #[error("Serialisation error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
