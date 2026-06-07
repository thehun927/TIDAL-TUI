//! API response models — all fields are intentional public API for later phases.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// JSON:API envelope types
// ---------------------------------------------------------------------------

/// Single-resource response: `{ "data": { ... }, "included": [...] }`
#[derive(Debug, Deserialize)]
pub struct SingleResponse<T> {
    pub data: T,
    #[serde(default)]
    pub included: Vec<serde_json::Value>,
}

/// Collection response: `{ "data": [...], "links": { ... } }`
#[derive(Debug, Deserialize)]
pub struct CollectionResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub links: Option<PageLinks>,
}

/// Pagination links returned in collection responses.
#[derive(Debug, Deserialize)]
pub struct PageLinks {
    #[serde(rename = "self")]
    pub self_: Option<String>,
    pub next:  Option<String>,
    pub prev:  Option<String>,
    pub first: Option<String>,
    pub last:  Option<String>,
}

/// Generic JSON:API resource object: `{ "id": "...", "type": "...", "attributes": { ... } }`
#[derive(Debug, Clone, Deserialize)]
pub struct Resource<A> {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: A,
}

// ---------------------------------------------------------------------------
// AudioQuality
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioQuality {
    Low,
    High,
    Lossless,
    HiRes,
}

impl AudioQuality {
    pub fn as_api_str(self) -> &'static str {
        match self {
            AudioQuality::Low      => "LOW",
            AudioQuality::High     => "HIGH",
            AudioQuality::Lossless => "LOSSLESS",
            AudioQuality::HiRes    => "HI_RES",
        }
    }
}

impl std::str::FromStr for AudioQuality {
    type Err = crate::errors::AppError;
    fn from_str(s: &str) -> crate::errors::Result<Self> {
        match s.to_uppercase().as_str() {
            "LOW"      => Ok(AudioQuality::Low),
            "HIGH"     => Ok(AudioQuality::High),
            "LOSSLESS" => Ok(AudioQuality::Lossless),
            "HI_RES"   => Ok(AudioQuality::HiRes),
            other      => Err(crate::errors::AppError::Config(
                format!("Unknown audio quality: {other}"),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Domain model attribute structs
// ---------------------------------------------------------------------------

/// Attributes inside a track resource.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackAttributes {
    pub title:          String,
    pub duration:       u32, // seconds
    #[serde(default)]
    pub track_number:   Option<u32>,
    #[serde(default)]
    pub volume_number:  Option<u32>,
    #[serde(default)]
    pub isrc:           Option<String>,
    #[serde(default)]
    pub explicit:       Option<bool>,
    #[serde(default)]
    pub popularity:     Option<u32>,
    #[serde(default)]
    pub availability:   Option<Vec<String>>,
    #[serde(default)]
    pub media_metadata: Option<MediaMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MediaMetadata {
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

pub type TrackResource = Resource<TrackAttributes>;

/// Attributes inside an album resource.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumAttributes {
    pub title: String,
    #[serde(default)]
    pub barcode_id:         Option<String>,
    #[serde(default)]
    pub number_of_tracks:   Option<u32>,
    #[serde(default)]
    pub number_of_volumes:  Option<u32>,
    #[serde(default)]
    pub release_date:       Option<String>,
    #[serde(default)]
    pub cover:              Option<String>,
    #[serde(default)]
    pub popularity:         Option<u32>,
    #[serde(default)]
    pub availability:       Option<Vec<String>>,
    #[serde(default)]
    pub explicit:           Option<bool>,
    #[serde(default)]
    pub duration:           Option<u32>,
}

pub type AlbumResource = Resource<AlbumAttributes>;

/// Attributes inside an artist resource.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistAttributes {
    pub name: String,
    #[serde(default)]
    pub popularity: Option<u32>,
    #[serde(default)]
    pub picture:    Option<String>,
}

pub type ArtistResource = Resource<ArtistAttributes>;

/// Attributes inside a playlist resource.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistAttributes {
    pub name: String,
    #[serde(default)]
    pub description:      Option<String>,
    #[serde(default)]
    pub number_of_tracks: Option<u32>,
    #[serde(default)]
    pub duration:         Option<u32>,
    #[serde(default)]
    pub last_modified_at: Option<String>,
    #[serde(default)]
    pub privacy:          Option<String>,
    #[serde(default)]
    pub image_cover:      Option<Vec<ImageCover>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageCover {
    pub url:    String,
    pub width:  u32,
    pub height: u32,
}

pub type PlaylistResource = Resource<PlaylistAttributes>;

// ---------------------------------------------------------------------------
// Search result envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultAttributes {
    #[serde(default)]
    pub tracks:    Option<Vec<TrackResource>>,
    #[serde(default)]
    pub albums:    Option<Vec<AlbumResource>>,
    #[serde(default)]
    pub artists:   Option<Vec<ArtistResource>>,
    #[serde(default)]
    pub playlists: Option<Vec<PlaylistResource>>,
}

pub type SearchResultResource = Resource<SearchResultAttributes>;

// ---------------------------------------------------------------------------
// Playback manifest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackInfoAttributes {
    pub asset_presentation: String,
    pub audio_mode:         String,
    pub audio_quality:      String,
    pub manifest:           String,
    pub manifest_mime_type: String,
}

pub type PlaybackInfoResource = Resource<PlaybackInfoAttributes>;

/// Resolved stream manifest — the URL ready to hand to MPV.
#[derive(Debug, Clone)]
pub struct StreamManifest {
    pub url:           String,
    pub codec:         String,
    pub audio_quality: AudioQuality,
}

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token:  String,
    pub token_type:    String,
    pub expires_in:    u64,
    /// Unix timestamp computed locally after receipt: `now + expires_in`.
    /// Not present in the TIDAL API response — defaults to 0, then overwritten
    /// by `map_token_response` before the token is used or persisted.
    #[serde(default)]
    pub expires_at:    u64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope:         Option<String>,
}

impl TokenSet {
    /// True if the access token has expired (with a 60-second safety buffer).
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now + 60 >= self.expires_at
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_set_expired() {
        let t = TokenSet {
            access_token:  "tok".into(),
            token_type:    "Bearer".into(),
            expires_in:    3600,
            expires_at:    0, // epoch — definitely expired
            refresh_token: None,
            scope:         None,
        };
        assert!(t.is_expired());
    }

    #[test]
    fn token_set_valid() {
        let t = TokenSet {
            access_token:  "tok".into(),
            token_type:    "Bearer".into(),
            expires_in:    3600,
            expires_at:    u64::MAX, // far future
            refresh_token: None,
            scope:         None,
        };
        assert!(!t.is_expired());
    }

    #[test]
    fn audio_quality_round_trip() {
        use std::str::FromStr;
        assert_eq!(AudioQuality::from_str("LOSSLESS").unwrap(), AudioQuality::Lossless);
        assert_eq!(AudioQuality::from_str("HI_RES").unwrap(), AudioQuality::HiRes);
        assert_eq!(AudioQuality::from_str("LOW").unwrap(), AudioQuality::Low);
        assert_eq!(AudioQuality::from_str("HIGH").unwrap(), AudioQuality::High);
        assert_eq!(AudioQuality::Lossless.as_api_str(), "LOSSLESS");
        assert_eq!(AudioQuality::HiRes.as_api_str(), "HI_RES");
    }

    #[test]
    fn audio_quality_unknown_errors() {
        use std::str::FromStr;
        assert!(AudioQuality::from_str("ULTRA").is_err());
    }

    #[test]
    fn single_response_deserialises() {
        let json = r#"{
            "data": {
                "id": "123",
                "type": "tracks",
                "attributes": {
                    "title": "Test Track",
                    "duration": 180
                }
            }
        }"#;
        let resp: SingleResponse<TrackResource> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.id, "123");
        assert_eq!(resp.data.attributes.title, "Test Track");
        assert_eq!(resp.data.attributes.duration, 180);
    }

    #[test]
    fn collection_response_deserialises() {
        let json = r#"{
            "data": [
                { "id": "1", "type": "tracks", "attributes": { "title": "A", "duration": 60 } },
                { "id": "2", "type": "tracks", "attributes": { "title": "B", "duration": 90 } }
            ]
        }"#;
        let resp: CollectionResponse<TrackResource> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].attributes.title, "A");
    }
}
