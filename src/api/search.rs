//! Search endpoint — playlists field used from Phase 5 onward.
#![allow(dead_code)]

use crate::{
    api::{client::TidalClient, models::*},
    errors::Result,
};

pub struct SearchResults {
    pub tracks:    Vec<TrackResource>,
    pub albums:    Vec<AlbumResource>,
    pub artists:   Vec<ArtistResource>,
    pub playlists: Vec<PlaylistResource>,
}

/// GET /search — search across all catalog resource types.
pub async fn search(client: &TidalClient, query: &str, limit: u32) -> Result<SearchResults> {
    let cc    = client.config.country_code.as_str();
    let limit = limit.to_string();
    let resp: SingleResponse<SearchResultResource> = client
        .get(
            "/search",
            &[
                ("query",       query),
                ("countryCode", cc),
                ("type",        "TRACKS,ALBUMS,ARTISTS,PLAYLISTS"),
                ("limit",       &limit),
            ],
        )
        .await?;

    let attrs = resp.data.attributes;
    Ok(SearchResults {
        tracks:    attrs.tracks.unwrap_or_default(),
        albums:    attrs.albums.unwrap_or_default(),
        artists:   attrs.artists.unwrap_or_default(),
        playlists: attrs.playlists.unwrap_or_default(),
    })
}
