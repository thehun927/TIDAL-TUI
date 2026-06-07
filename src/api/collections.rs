//! User collection endpoints — all used from Phase 5 onward.
#![allow(dead_code)]

use crate::{
    api::{client::TidalClient, models::*},
    errors::Result,
};

/// GET /users/{user_id}/favorites/tracks
pub async fn get_favorite_tracks(
    client:  &TidalClient,
    user_id: &str,
) -> Result<Vec<TrackResource>> {
    let cc = client.config.country_code.as_str();
    let resp: CollectionResponse<TrackResource> = client
        .get(
            &format!("/users/{user_id}/favorites/tracks"),
            &[("countryCode", cc), ("limit", "50"), ("offset", "0")],
        )
        .await?;
    Ok(resp.data)
}

/// GET /users/{user_id}/playlists
pub async fn get_user_playlists(
    client:  &TidalClient,
    user_id: &str,
) -> Result<Vec<PlaylistResource>> {
    let resp: CollectionResponse<PlaylistResource> = client
        .get(
            &format!("/users/{user_id}/playlists"),
            &[("limit", "50"), ("offset", "0")],
        )
        .await?;
    Ok(resp.data)
}

/// GET /playlists/{id}/items — the tracks in a playlist.
pub async fn get_playlist_items(
    client: &TidalClient,
    id:     &str,
) -> Result<Vec<TrackResource>> {
    let cc = client.config.country_code.as_str();
    let resp: CollectionResponse<TrackResource> = client
        .get(
            &format!("/playlists/{id}/items"),
            &[("countryCode", cc), ("limit", "50"), ("offset", "0")],
        )
        .await?;
    Ok(resp.data)
}

/// POST /users/{user_id}/favorites/tracks — add a track to favourites.
pub async fn add_favorite_track(
    client:   &TidalClient,
    user_id:  &str,
    track_id: &str,
) -> Result<()> {
    // The API returns 201 with an empty body on success; we discard it.
    let _: serde_json::Value = client
        .post(
            &format!("/users/{user_id}/favorites/tracks"),
            &serde_json::json!({
                "data": [{ "type": "tracks", "id": track_id }]
            }),
        )
        .await?;
    Ok(())
}

/// DELETE /users/{user_id}/favorites/tracks/{track_id} — remove from favourites.
pub async fn remove_favorite_track(
    client:   &TidalClient,
    user_id:  &str,
    track_id: &str,
) -> Result<()> {
    client
        .delete(&format!("/users/{user_id}/favorites/tracks/{track_id}"))
        .await
}
