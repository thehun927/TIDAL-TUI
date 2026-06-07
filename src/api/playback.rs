//! Playback manifest resolution — get_stream_url used from Phase 3 onward.
#![allow(dead_code)]

use base64::{engine::general_purpose::STANDARD, Engine};
use crate::{
    api::{client::TidalClient, models::*},
    errors::{AppError, Result},
};

/// Fetch the playback manifest for a track and resolve it to a `StreamManifest`.
///
/// Endpoint: GET /tracks/{id}/playbackinfo
///
/// The manifest field is base64-encoded. Two MIME types are possible:
/// - `application/vnd.tidal.bts` — proprietary JSON `{ "urls": ["..."] }`
/// - `application/dash+xml`      — MPEG-DASH MPD; extract `<BaseURL>` for now
pub async fn get_stream_url(
    client:  &TidalClient,
    id:      &str,
    quality: AudioQuality,
) -> Result<StreamManifest> {
    let cc = client.config.country_code.as_str();
    let resp: SingleResponse<PlaybackInfoResource> = client
        .get(
            &format!("/tracks/{id}/playbackinfo"),
            &[
                ("countryCode",       cc),
                ("audioquality",      quality.as_api_str()),
                ("playbackmode",      "STREAM"),
                ("assetpresentation", "FULL"),
            ],
        )
        .await?;

    let attrs = resp.data.attributes;
    let raw   = STANDARD.decode(&attrs.manifest).map_err(|e| AppError::Api {
        status:  0,
        message: format!("Failed to base64-decode manifest: {e}"),
    })?;

    let url = match attrs.manifest_mime_type.as_str() {
        "application/vnd.tidal.bts" => parse_bts_manifest(&raw)?,
        "application/dash+xml"      => extract_dash_base_url(&raw)?,
        other => {
            return Err(AppError::Api {
                status:  0,
                message: format!("Unknown manifest MIME type: {other}"),
            })
        }
    };

    Ok(StreamManifest {
        url,
        codec: attrs.audio_quality,
        audio_quality: quality,
    })
}

/// Parse a TIDAL BTS manifest: `{ "mimeType": "...", "urls": ["..."] }`
fn parse_bts_manifest(raw: &[u8]) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct BtsManifest {
        urls: Vec<String>,
    }
    let m: BtsManifest = serde_json::from_slice(raw)?;
    m.urls.into_iter().next().ok_or_else(|| AppError::Api {
        status:  0,
        message: "BTS manifest contains no URLs".into(),
    })
}

/// Extract the first `<BaseURL>` element from a DASH/MPD manifest.
/// Full MPD parsing is deferred to Phase 3.
fn extract_dash_base_url(raw: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(raw).map_err(|e| AppError::Api {
        status:  0,
        message: format!("DASH manifest is not valid UTF-8: {e}"),
    })?;
    if let (Some(start), Some(end)) = (text.find("<BaseURL>"), text.find("</BaseURL>")) {
        return Ok(text[start + 9..end].to_string());
    }
    Err(AppError::Api {
        status:  0,
        message: "No <BaseURL> element found in DASH manifest".into(),
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bts_manifest_returns_first_url() {
        let manifest = br#"{"mimeType":"audio/flac","urls":["https://example.com/stream.flac"]}"#;
        let url = parse_bts_manifest(manifest).unwrap();
        assert_eq!(url, "https://example.com/stream.flac");
    }

    #[test]
    fn parse_bts_manifest_empty_urls_errors() {
        let manifest = br#"{"mimeType":"audio/flac","urls":[]}"#;
        assert!(parse_bts_manifest(manifest).is_err());
    }

    #[test]
    fn extract_dash_base_url_works() {
        let mpd = b"<?xml?><MPD><BaseURL>https://cdn.example.com/audio/</BaseURL></MPD>";
        let url = extract_dash_base_url(mpd).unwrap();
        assert_eq!(url, "https://cdn.example.com/audio/");
    }

    #[test]
    fn extract_dash_base_url_missing_errors() {
        let mpd = b"<?xml?><MPD></MPD>";
        assert!(extract_dash_base_url(mpd).is_err());
    }
}
