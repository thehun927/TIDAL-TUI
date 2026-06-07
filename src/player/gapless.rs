#![allow(dead_code)]

use crate::{errors::{AppError, Result}};

/// Configure gapless playback on a live `MpvHandler`.
///
/// Prefer setting this during startup through the builder. Runtime changes may
/// not affect the currently loaded file until the next track starts.
pub fn set_gapless(mpv: &mut mpv::MpvHandler, enabled: bool) -> Result<()> {
    let value = if enabled { "yes" } else { "no" };
    mpv.set_property("gapless-audio", value)
        .map_err(|e| AppError::Player(e.to_string()))
}
