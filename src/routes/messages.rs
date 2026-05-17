use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use pebble_core::{Message, MessageSummary};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListMessagesParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn list_messages_by_folder(
    State(state): State<AppStateRef>,
    Path(folder_id): Path<String>,
    Query(params): Query<ListMessagesParams>,
) -> Result<Json<Vec<MessageSummary>>, ApiError> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let store = state.store.clone();

    let messages = store
        .with_read_async(move |conn| {
            let sql = "SELECT m.id, m.account_id, m.remote_id, m.message_id_header, m.in_reply_to, \
                 m.references_header, m.thread_id, m.subject, m.snippet, m.from_address, \
                 m.from_name, m.to_list, m.cc_list, m.bcc_list, \
                 m.has_attachments, m.is_read, m.is_starred, m.is_draft, \
                 m.date, m.remote_version, m.is_deleted, m.deleted_at, m.created_at, m.updated_at \
                 FROM messages m \
                 JOIN message_folders mf ON m.id = mf.message_id \
                 WHERE mf.folder_id = ?1 AND m.is_deleted = 0 \
                 ORDER BY m.date DESC \
                 LIMIT ?2 OFFSET ?3";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params![folder_id, limit, offset], |row| {
                let to_json: String = row.get(11)?;
                let cc_json: String = row.get(12)?;
                let bcc_json: String = row.get(13)?;
                let has_attachments: i32 = row.get(14)?;
                let is_read: i32 = row.get(15)?;
                let is_starred: i32 = row.get(16)?;
                let is_draft: i32 = row.get(17)?;
                let is_deleted: i32 = row.get(20)?;
                Ok(MessageSummary {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    remote_id: row.get(2)?,
                    message_id_header: row.get(3)?,
                    in_reply_to: row.get(4)?,
                    references_header: row.get(5)?,
                    thread_id: row.get(6)?,
                    subject: row.get(7)?,
                    snippet: row.get(8)?,
                    from_address: row.get(9)?,
                    from_name: row.get(10)?,
                    to_list: serde_json::from_str(&to_json).unwrap_or_default(),
                    cc_list: serde_json::from_str(&cc_json).unwrap_or_default(),
                    bcc_list: serde_json::from_str(&bcc_json).unwrap_or_default(),
                    has_attachments: has_attachments != 0,
                    is_read: is_read != 0,
                    is_starred: is_starred != 0,
                    is_draft: is_draft != 0,
                    date: row.get(18)?,
                    remote_version: row.get(19)?,
                    is_deleted: is_deleted != 0,
                    deleted_at: row.get(21)?,
                    created_at: row.get(22)?,
                    updated_at: row.get(23)?,
                })
            })?;
            let mut messages = Vec::new();
            for row in rows {
                messages.push(row?);
            }
            Ok(messages)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to list messages: {e}")))?;

    Ok(Json(messages))
}

pub async fn get_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
) -> Result<Json<Message>, ApiError> {
    let store = state.store.clone();

    let message = store
        .with_read_async(move |conn| {
            let sql = "SELECT id, account_id, remote_id, message_id_header, in_reply_to, \
                 references_header, thread_id, subject, snippet, from_address, \
                 from_name, to_list, cc_list, bcc_list, \
                 body_text, body_html_raw, \
                 has_attachments, is_read, is_starred, is_draft, \
                 date, remote_version, is_deleted, deleted_at, created_at, updated_at \
                 FROM messages WHERE id = ?1";
            let result = conn
                .query_row(sql, rusqlite::params![message_id], |row| {
                    let to_json: String = row.get(11)?;
                    let cc_json: String = row.get(12)?;
                    let bcc_json: String = row.get(13)?;
                    let has_attachments: i32 = row.get(16)?;
                    let is_read: i32 = row.get(17)?;
                    let is_starred: i32 = row.get(18)?;
                    let is_draft: i32 = row.get(19)?;
                    let is_deleted: i32 = row.get(22)?;
                    Ok(Message {
                        id: row.get(0)?,
                        account_id: row.get(1)?,
                        remote_id: row.get(2)?,
                        message_id_header: row.get(3)?,
                        in_reply_to: row.get(4)?,
                        references_header: row.get(5)?,
                        thread_id: row.get(6)?,
                        subject: row.get(7)?,
                        snippet: row.get(8)?,
                        from_address: row.get(9)?,
                        from_name: row.get(10)?,
                        to_list: serde_json::from_str(&to_json).unwrap_or_default(),
                        cc_list: serde_json::from_str(&cc_json).unwrap_or_default(),
                        bcc_list: serde_json::from_str(&bcc_json).unwrap_or_default(),
                        body_text: row.get(14)?,
                        body_html_raw: row.get(15)?,
                        has_attachments: has_attachments != 0,
                        is_read: is_read != 0,
                        is_starred: is_starred != 0,
                        is_draft: is_draft != 0,
                        date: row.get(20)?,
                        remote_version: row.get(21)?,
                        is_deleted: is_deleted != 0,
                        deleted_at: row.get(23)?,
                        created_at: row.get(24)?,
                        updated_at: row.get(25)?,
                    })
                })
                .optional();
            match result {
                Ok(Some(msg)) => Ok(Some(msg)),
                Ok(None) => Ok(None),
                Err(e) => Err(pebble_core::PebbleError::Storage(e.to_string())),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get message: {e}")))?;

    match message {
        Some(msg) => Ok(Json(msg)),
        None => Err(ApiError::BadRequest("Message not found".to_string())),
    }
}

#[derive(Deserialize)]
pub struct UpdateFlagsRequest {
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
}

pub async fn update_message_flags(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
    Json(body): Json<UpdateFlagsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let is_read = body.is_read;
    let is_starred = body.is_starred;

    store
        .with_write_async(move |conn| {
            let mut sets = Vec::new();
            let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(read) = is_read {
                sets.push(format!("is_read = ?{}", values.len() + 1));
                values.push(Box::new(read as i32));
            }
            if let Some(starred) = is_starred {
                sets.push(format!("is_starred = ?{}", values.len() + 1));
                values.push(Box::new(starred as i32));
            }

            if sets.is_empty() {
                return Ok(());
            }

            let now = pebble_core::now_timestamp();
            sets.push(format!("updated_at = ?{}", values.len() + 1));
            values.push(Box::new(now));

            let id_idx = values.len() + 1;
            values.push(Box::new(message_id));

            let sql = format!(
                "UPDATE messages SET {} WHERE id = ?{}",
                sets.join(", "),
                id_idx
            );
            let params: Vec<&dyn rusqlite::types::ToSql> =
                values.iter().map(|v| v.as_ref()).collect();
            conn.execute(&sql, params.as_slice())?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to update flags: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct MoveMessageRequest {
    pub target_folder_id: String,
}

pub async fn move_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
    Json(body): Json<MoveMessageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let target_folder_id = body.target_folder_id;

    store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            let tx = conn.unchecked_transaction()?;

            tx.execute(
                "DELETE FROM message_folders WHERE message_id = ?1",
                rusqlite::params![message_id],
            )?;

            tx.execute(
                "INSERT INTO message_folders (message_id, folder_id) VALUES (?1, ?2)",
                rusqlite::params![message_id, target_folder_id],
            )?;

            tx.execute(
                "UPDATE messages SET is_deleted = 0, deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, message_id],
            )?;

            tx.commit()?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to move message: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn delete_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            conn.execute(
                "UPDATE messages SET is_deleted = 1, deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, message_id],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete message: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

use rusqlite::OptionalExtension;
