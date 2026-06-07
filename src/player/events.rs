#![allow(dead_code)]

use crate::player::{repeat::RepeatMode, shuffle::ShuffleMode};

/// Events emitted by the MPV player thread and consumed by the main application.
#[derive(Debug, Clone)]
pub enum PlayerEvent {
    /// A new track has started loading into the player.
    TrackStarted { track_id: String },
    /// Playback of a track ended.
    TrackEnded { track_id: String, reason: EndReason },
    /// Current playback position in seconds — fires ~60× per second via `observe_property`.
    PositionChanged(f64),
    /// Total duration of the current track as confirmed by mpv.
    DurationChanged(f64),
    /// Player transitioned into the paused state.
    Paused,
    /// Player transitioned out of the paused state.
    Resumed,
    /// The upcoming queue is exhausted and the player entered idle.
    QueueEmpty,
    /// Volume level changed (0–100).
    VolumeChanged(u32),
    RepeatModeChanged(RepeatMode),
    ShuffleModeChanged(ShuffleMode),
    CrossfadeChanged { enabled: bool, seconds: u8 },
    QueueRefillRequested {
        mode: ShuffleMode,
        based_on_track_id: Option<String>,
    },
    /// An internal mpv or player-thread error.
    Error(String),
}

/// Why a track's playback ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndReason {
    /// Track finished playing to its natural end.
    Eof,
    /// Stopped by an explicit Stop / Next / Prev command.
    Stopped,
    /// Playback ended due to an error.
    Error,
}
