#![allow(dead_code)]

use crate::errors::Result;

/// Placeholder hook for future crossfade support.
///
/// libmpv does not expose a simple single "crossfade" property for audio-only
/// use in this crate version, so the real implementation is deferred to Phase 4.
pub fn set_crossfade(_mpv: &mut mpv::MpvHandler, _seconds: u8) -> Result<()> {
    Ok(())
}
