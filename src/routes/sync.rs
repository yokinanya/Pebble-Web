use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{extract::State, Json};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerSyncRequest {
    pub account_id: String,
}

pub async fn trigger_sync(
    State(state): State<AppStateRef>,
    Json(body): Json<TriggerSyncRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .sync_manager
        .trigger_sync(&body.account_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to trigger sync: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
