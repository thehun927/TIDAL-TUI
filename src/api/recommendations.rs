//! Track radio / recommendations — used from Phase 4 (Discovery shuffle) onward.
#![allow(dead_code)]

use crate::{
    api::{client::TidalClient, models::*},
    errors::Result,
};

/// GET /tracks/{id}/radio — returns recommended tracks based on the given track.
/// Used by Discovery shuffle mode (Phase 4).
pub async fn get_track_radio(
    client: &TidalClient,
    id:     &str,
    limit:  u32,
) -> Result<Vec<TrackResource>> {
    let cc    = client.config.country_code.as_str();
    let limit = limit.to_string();
    let resp: CollectionResponse<TrackResource> = client
        .get(
            &format!("/tracks/{id}/radio"),
            &[("countryCode", cc), ("limit", &limit)],
        )
        .await?;
    Ok(resp.data)
}
