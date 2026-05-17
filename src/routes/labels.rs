use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{
    extract::{Path, State},
    Json,
};
use pebble_store::labels::Label;
use serde::Deserialize;

pub async fn list_labels(
    State(state): State<AppStateRef>,
) -> Result<Json<Vec<Label>>, ApiError> {
    let labels = state
        .store
        .list_labels()
        .map_err(|e| ApiError::Internal(format!("Failed to list labels: {e}")))?;

    Ok(Json(labels))
}

#[derive(Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: Option<String>,
}

pub async fn create_label(
    State(state): State<AppStateRef>,
    Json(body): Json<CreateLabelRequest>,
) -> Result<Json<Label>, ApiError> {
    let store = state.store.clone();
    let name = body.name;
    let color = body.color.unwrap_or_else(|| "#808080".to_string());

    let label = store
        .with_write_async(move |conn| {
            let id = pebble_core::new_id();
            conn.execute(
                "INSERT INTO labels (id, name, color, is_system) VALUES (?1, ?2, ?3, 0)",
                rusqlite::params![id, name, color],
            )?;
            Ok(Label {
                id,
                name,
                color,
                is_system: false,
                rule_id: None,
            })
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create label: {e}")))?;

    Ok(Json(label))
}

pub async fn delete_label(
    State(state): State<AppStateRef>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    store
        .with_write_async(move |conn| {
            conn.execute(
                "DELETE FROM message_labels WHERE label_id = ?1",
                rusqlite::params![id],
            )?;
            conn.execute(
                "DELETE FROM labels WHERE id = ?1",
                rusqlite::params![id],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete label: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct AddLabelToMessageRequest {
    pub label_id: String,
}

pub async fn add_label_to_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
    Json(body): Json<AddLabelToMessageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let label_id = body.label_id;

    store
        .with_write_async(move |conn| {
            conn.execute(
                "INSERT OR IGNORE INTO message_labels (message_id, label_id) VALUES (?1, ?2)",
                rusqlite::params![message_id, label_id],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to add label to message: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn remove_label_from_message(
    State(state): State<AppStateRef>,
    Path((message_id, label_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    store
        .with_write_async(move |conn| {
            conn.execute(
                "DELETE FROM message_labels WHERE message_id = ?1 AND label_id = ?2",
                rusqlite::params![message_id, label_id],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to remove label from message: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
