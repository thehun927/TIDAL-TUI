#![allow(dead_code)]

use crate::{errors::{AppError, Result}};

/// Apply ReplayGain settings to a live `MpvHandler`.
pub fn apply(mpv: &mut mpv::MpvHandler, mode: &str, preamp_db: f32) -> Result<()> {
    mpv.set_property("replaygain", mode)
        .map_err(|e| AppError::Player(e.to_string()))?;
    mpv.set_property("replaygain-preamp", preamp_db as f64)
        .map_err(|e| AppError::Player(e.to_string()))?;
    Ok(())
}
