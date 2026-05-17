use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use pebble_core::{Message, MessageSummary};
use rusqlite::OptionalExtension;
use serde::Deserialize;
use serde_json::json;

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
        None => Err(ApiError::NotFound("Message not found".to_string())),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct MoveMessageRequest {
    pub folder_id: String,
}

pub async fn move_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
    Json(body): Json<MoveMessageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let folder_id = body.folder_id;

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
                rusqlite::params![message_id, folder_id],
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

// --- New handlers ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyModeRequest {
    #[allow(dead_code)]
    pub privacy_mode: Option<String>,
}

pub async fn get_message_with_html(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
    Json(_body): Json<PrivacyModeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    let result = store
        .with_read_async(move |conn| {
            let sql = "SELECT id, account_id, remote_id, message_id_header, in_reply_to, \
                 references_header, thread_id, subject, snippet, from_address, \
                 from_name, to_list, cc_list, bcc_list, \
                 body_text, body_html_raw, \
                 has_attachments, is_read, is_starred, is_draft, \
                 date, remote_version, is_deleted, deleted_at, created_at, updated_at \
                 FROM messages WHERE id = ?1";
            let row_result = conn
                .query_row(sql, rusqlite::params![message_id], |row| {
                    let to_json: String = row.get(11)?;
                    let cc_json: String = row.get(12)?;
                    let bcc_json: String = row.get(13)?;
                    let body_html_raw: String = row.get(15)?;
                    let has_attachments: i32 = row.get(16)?;
                    let is_read: i32 = row.get(17)?;
                    let is_starred: i32 = row.get(18)?;
                    let is_draft: i32 = row.get(19)?;
                    let is_deleted: i32 = row.get(22)?;
                    let msg = Message {
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
                        body_html_raw: body_html_raw.clone(),
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
                    };
                    Ok((msg, body_html_raw))
                })
                .optional();
            match row_result {
                Ok(Some(data)) => Ok(Some(data)),
                Ok(None) => Ok(None),
                Err(e) => Err(pebble_core::PebbleError::Storage(e.to_string())),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get message with html: {e}")))?;

    match result {
        Some((msg, html_raw)) => {
            let msg_json = serde_json::to_value(msg)
                .map_err(|e| ApiError::Internal(format!("Failed to serialize message: {e}")))?;
            let html_part = json!({
                "html": html_raw,
                "loadedRemoteContent": false,
                "trackers_blocked": [],
                "images_blocked": 0
            });
            Ok(Json(json!([msg_json, html_part])))
        }
        None => Err(ApiError::NotFound("Message not found".to_string())),
    }
}

pub async fn render_html(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
    Json(_body): Json<PrivacyModeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    let result = store
        .with_read_async(move |conn| {
            let sql = "SELECT body_html_raw FROM messages WHERE id = ?1";
            let row_result = conn
                .query_row(sql, rusqlite::params![message_id], |row| {
                    let html: String = row.get(0)?;
                    Ok(html)
                })
                .optional();
            match row_result {
                Ok(Some(html)) => Ok(Some(html)),
                Ok(None) => Ok(None),
                Err(e) => Err(pebble_core::PebbleError::Storage(e.to_string())),
            }
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to render html: {e}")))?;

    match result {
        Some(html) => Ok(Json(json!({
            "html": html,
            "loadedRemoteContent": false,
            "trackers_blocked": [],
            "images_blocked": 0
        }))),
        None => Err(ApiError::NotFound("Message not found".to_string())),
    }
}

pub async fn archive_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            let tx = conn.unchecked_transaction()?;

            // Find the account_id for this message
            let account_id: String = tx.query_row(
                "SELECT account_id FROM messages WHERE id = ?1",
                rusqlite::params![message_id],
                |row| row.get(0),
            ).map_err(|_| pebble_core::PebbleError::Storage("Message not found".to_string()))?;

            // Find the archive folder for this account
            let archive_folder_id: Option<String> = tx.query_row(
                "SELECT id FROM folders WHERE account_id = ?1 AND role = 'archive'",
                rusqlite::params![account_id],
                |row| row.get(0),
            ).optional()
            .map_err(|e| pebble_core::PebbleError::Storage(e.to_string()))?;

            if let Some(folder_id) = archive_folder_id {
                tx.execute(
                    "DELETE FROM message_folders WHERE message_id = ?1",
                    rusqlite::params![message_id],
                )?;
                tx.execute(
                    "INSERT INTO message_folders (message_id, folder_id) VALUES (?1, ?2)",
                    rusqlite::params![message_id, folder_id],
                )?;
                tx.execute(
                    "UPDATE messages SET is_deleted = 0, deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, message_id],
                )?;
            } else {
                // No archive folder — soft-delete as fallback
                tx.execute(
                    "UPDATE messages SET is_deleted = 1, deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, message_id],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to archive message: {e}")))?;

    Ok(Json(json!("archived")))
}

pub async fn restore_message(
    State(state): State<AppStateRef>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            conn.execute(
                "UPDATE messages SET is_deleted = 0, deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, message_id],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to restore message: {e}")))?;

    Ok(Json(json!({ "ok": true })))
}

pub async fn list_starred_messages(
    State(state): State<AppStateRef>,
    Path(account_id): Path<String>,
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
                 WHERE m.account_id = ?1 AND m.is_starred = 1 AND m.is_deleted = 0 \
                 ORDER BY m.date DESC \
                 LIMIT ?2 OFFSET ?3";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params![account_id, limit, offset], |row| {
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
        .map_err(|e| ApiError::Internal(format!("Failed to list starred messages: {e}")))?;

    Ok(Json(messages))
}

pub async fn empty_trash(
    State(state): State<AppStateRef>,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();

    let count = store
        .with_write_async(move |conn| {
            let tx = conn.unchecked_transaction()?;

            // Delete from message_folders for all deleted messages of this account
            tx.execute(
                "DELETE FROM message_folders WHERE message_id IN \
                 (SELECT id FROM messages WHERE account_id = ?1 AND is_deleted = 1)",
                rusqlite::params![account_id],
            )?;

            // Permanently delete messages
            let deleted = tx.execute(
                "DELETE FROM messages WHERE account_id = ?1 AND is_deleted = 1",
                rusqlite::params![account_id],
            )?;

            tx.commit()?;
            Ok(deleted)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to empty trash: {e}")))?;

    Ok(Json(json!(count)))
}

// --- Batch operations ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchMessageIds {
    pub message_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchMarkReadRequest {
    pub message_ids: Vec<String>,
    pub is_read: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchStarRequest {
    pub message_ids: Vec<String>,
    pub starred: bool,
}

pub async fn batch_archive(
    State(state): State<AppStateRef>,
    Json(body): Json<BatchMessageIds>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let message_ids = body.message_ids;

    let count = store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            let tx = conn.unchecked_transaction()?;
            let mut total = 0usize;
            for id in &message_ids {
                // Find account_id for this message
                let account_id: Option<String> = tx.query_row(
                    "SELECT account_id FROM messages WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                ).optional()?;

                let Some(account_id) = account_id else { continue };

                // Find archive folder
                let archive_folder_id: Option<String> = tx.query_row(
                    "SELECT id FROM folders WHERE account_id = ?1 AND role = 'archive'",
                    rusqlite::params![account_id],
                    |row| row.get(0),
                ).optional()?;

                if let Some(folder_id) = archive_folder_id {
                    tx.execute(
                        "DELETE FROM message_folders WHERE message_id = ?1",
                        rusqlite::params![id],
                    )?;
                    tx.execute(
                        "INSERT INTO message_folders (message_id, folder_id) VALUES (?1, ?2)",
                        rusqlite::params![id, folder_id],
                    )?;
                    tx.execute(
                        "UPDATE messages SET is_deleted = 0, deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
                        rusqlite::params![now, id],
                    )?;
                } else {
                    tx.execute(
                        "UPDATE messages SET is_deleted = 1, deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                        rusqlite::params![now, id],
                    )?;
                }
                total += 1;
            }
            tx.commit()?;
            Ok(total)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to batch archive: {e}")))?;

    Ok(Json(json!(count)))
}

pub async fn batch_delete(
    State(state): State<AppStateRef>,
    Json(body): Json<BatchMessageIds>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let message_ids = body.message_ids;

    let count = store
        .with_write_async(move |conn| {
            let tx = conn.unchecked_transaction()?;
            let mut total = 0usize;
            for id in &message_ids {
                tx.execute(
                    "DELETE FROM message_folders WHERE message_id = ?1",
                    rusqlite::params![id],
                )?;
                let affected = tx.execute(
                    "DELETE FROM messages WHERE id = ?1",
                    rusqlite::params![id],
                )?;
                total += affected;
            }
            tx.commit()?;
            Ok(total)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to batch delete: {e}")))?;

    Ok(Json(json!(count)))
}

pub async fn batch_mark_read(
    State(state): State<AppStateRef>,
    Json(body): Json<BatchMarkReadRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let message_ids = body.message_ids;
    let is_read = body.is_read;

    let count = store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            let read_val = is_read as i32;
            let mut total = 0usize;
            for id in &message_ids {
                let affected = conn.execute(
                    "UPDATE messages SET is_read = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![read_val, now, id],
                )?;
                total += affected;
            }
            Ok(total)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to batch mark read: {e}")))?;

    Ok(Json(json!(count)))
}

pub async fn batch_star(
    State(state): State<AppStateRef>,
    Json(body): Json<BatchStarRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    let message_ids = body.message_ids;
    let starred = body.starred;

    let count = store
        .with_write_async(move |conn| {
            let now = pebble_core::now_timestamp();
            let starred_val = starred as i32;
            let mut total = 0usize;
            for id in &message_ids {
                let affected = conn.execute(
                    "UPDATE messages SET is_starred = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![starred_val, now, id],
                )?;
                total += affected;
            }
            Ok(total)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to batch star: {e}")))?;

    Ok(Json(json!(count)))
}

pub async fn get_messages_batch(
    State(state): State<AppStateRef>,
    Json(body): Json<BatchMessageIds>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let store = state.store.clone();
    let message_ids = body.message_ids;

    let messages = store
        .with_read_async(move |conn| {
            let mut results = Vec::new();
            for id in &message_ids {
                let sql = "SELECT id, account_id, remote_id, message_id_header, in_reply_to, \
                     references_header, thread_id, subject, snippet, from_address, \
                     from_name, to_list, cc_list, bcc_list, \
                     body_text, body_html_raw, \
                     has_attachments, is_read, is_starred, is_draft, \
                     date, remote_version, is_deleted, deleted_at, created_at, updated_at \
                     FROM messages WHERE id = ?1";
                let row = conn
                    .query_row(sql, rusqlite::params![id], |row| {
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
                    .optional()?;
                if let Some(msg) = row {
                    results.push(msg);
                }
            }
            Ok(results)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get messages batch: {e}")))?;

    Ok(Json(messages))
}
