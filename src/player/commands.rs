#![allow(dead_code)]

use crate::player::{queue::QueueEntry, repeat::RepeatMode, shuffle::ShuffleMode};

/// Commands sent from the main application to the MPV player thread.
#[derive(Debug)]
pub enum AudioCommand {
    /// Load a URL and start playing immediately, replacing any current track.
    Play { url: String, entry: QueueEntry },
    /// Pause playback.
    Pause,
    /// Resume paused playback.
    Resume,
    /// Toggle between paused and playing.
    TogglePause,
    /// Seek forward (positive) or backward (negative) by delta seconds.
    Seek(f64),
    /// Jump to an absolute position in seconds.
    SeekAbsolute(f64),
    /// Set volume 0–100.
    SetVolume(u32),
    /// Skip to the next track in the queue.
    Next,
    /// Step back to the previous track in history.
    Prev,
    /// Stop playback without advancing the queue.
    Stop,
    /// Append a track to the end of the upcoming queue.
    Enqueue(QueueEntry),
    /// Clear the upcoming queue (does not affect the currently playing track).
    ClearQueue,
    /// Set repeat mode directly.
    SetRepeatMode(RepeatMode),
    /// Cycle repeat mode Off -> One -> All -> Off.
    ToggleRepeatMode,
    /// Set shuffle mode directly.
    SetShuffleMode(ShuffleMode),
    /// Cycle shuffle mode Off -> Random -> Favourites -> Discovery -> Off.
    ToggleShuffleMode,
    /// Enable or disable crossfade state.
    SetCrossfadeEnabled(bool),
    /// Set crossfade duration in seconds.
    SetCrossfadeSeconds(u8),
    /// Cleanly shut down the MPV thread.
    Shutdown,
}
