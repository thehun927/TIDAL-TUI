//! Catalog endpoints — get_artist_albums used from Phase 5 onward.
#![allow(dead_code)]

use crate::{
    api::{client::TidalClient, models::*},
    errors::Result,
};

/// GET /tracks/{id}
pub async fn get_track(client: &TidalClient, id: &str) -> Result<TrackResource> {
    let cc = client.config.country_code.as_str();
    let resp: SingleResponse<TrackResource> = client
        .get(&format!("/tracks/{id}"), &[("countryCode", cc)])
        .await?;
    Ok(resp.data)
}

/// GET /albums/{id}
pub async fn get_album(client: &TidalClient, id: &str) -> Result<AlbumResource> {
    let cc = client.config.country_code.as_str();
    let resp: SingleResponse<AlbumResource> = client
        .get(&format!("/albums/{id}"), &[("countryCode", cc)])
        .await?;
    Ok(resp.data)
}

/// GET /albums/{id}/items — the tracks that make up an album.
pub async fn get_album_items(client: &TidalClient, id: &str) -> Result<Vec<TrackResource>> {
    let cc = client.config.country_code.as_str();
    let resp: CollectionResponse<TrackResource> = client
        .get(
            &format!("/albums/{id}/items"),
            &[("countryCode", cc), ("limit", "50"), ("offset", "0")],
        )
        .await?;
    Ok(resp.data)
}

/// GET /artists/{id}
pub async fn get_artist(client: &TidalClient, id: &str) -> Result<ArtistResource> {
    let cc = client.config.country_code.as_str();
    let resp: SingleResponse<ArtistResource> = client
        .get(&format!("/artists/{id}"), &[("countryCode", cc)])
        .await?;
    Ok(resp.data)
}

/// GET /artists/{id}/albums
pub async fn get_artist_albums(client: &TidalClient, id: &str) -> Result<Vec<AlbumResource>> {
    let cc = client.config.country_code.as_str();
    let resp: CollectionResponse<AlbumResource> = client
        .get(
            &format!("/artists/{id}/albums"),
            &[("countryCode", cc), ("limit", "20"), ("offset", "0")],
        )
        .await?;
    Ok(resp.data)
}
