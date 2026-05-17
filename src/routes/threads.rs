use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use pebble_core::{Message, ThreadSummary};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListThreadsParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn list_threads_by_folder(
    State(state): State<AppStateRef>,
    Path(folder_id): Path<String>,
    Query(params): Query<ListThreadsParams>,
) -> Result<Json<Vec<ThreadSummary>>, ApiError> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let store = state.store.clone();

    let threads = store
        .with_read_async(move |conn| {
            let mut stmt = conn.prepare(
                "WITH thread_participants AS (
                    SELECT thread_id,
                           GROUP_CONCAT(from_address, '||') AS participants
                    FROM (
                        SELECT DISTINCT m3.thread_id, m3.from_address
                        FROM messages m3
                        JOIN message_folders mf3 ON m3.id = mf3.message_id
                        WHERE mf3.folder_id = ?1
                          AND m3.is_deleted = 0
                          AND m3.thread_id IS NOT NULL
                    )
                    GROUP BY thread_id
                 )
                 SELECT
                    m.thread_id,
                    MAX(m.subject) as subject,
                    MAX(CASE WHEN m.date = max_date.md THEN m.snippet ELSE '' END) as snippet,
                    MAX(m.date) as last_date,
                    COUNT(*) as message_count,
                    SUM(CASE WHEN m.is_read = 0 THEN 1 ELSE 0 END) as unread_count,
                    MAX(m.is_starred) as is_starred,
                    COALESCE(tp.participants, '') as participants,
                    MAX(m.has_attachments) as has_attachments
                 FROM messages m
                 JOIN message_folders mf ON m.id = mf.message_id
                 JOIN (
                    SELECT m2.thread_id, MAX(m2.date) as md
                    FROM messages m2
                    JOIN message_folders mf2 ON m2.id = mf2.message_id
                    WHERE mf2.folder_id = ?1
                      AND m2.is_deleted = 0
                      AND m2.thread_id IS NOT NULL
                    GROUP BY m2.thread_id
                 ) max_date ON m.thread_id = max_date.thread_id
                 LEFT JOIN thread_participants tp ON m.thread_id = tp.thread_id
                 WHERE mf.folder_id = ?1 AND m.is_deleted = 0 AND m.thread_id IS NOT NULL
                 GROUP BY m.thread_id
                 ORDER BY last_date DESC
                 LIMIT ?2 OFFSET ?3",
            )?;

            let rows = stmt.query_map(rusqlite::params![folder_id, limit, offset], |row| {
                let participants_str: String = row.get(7)?;
                let participants: Vec<String> = participants_str
                    .split("||")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                let is_starred: i32 = row.get(6)?;
                let has_attachments: i32 = row.get(8)?;
                Ok(ThreadSummary {
                    thread_id: row.get(0)?,
                    subject: row.get(1)?,
                    snippet: row.get(2)?,
                    last_date: row.get(3)?,
                    message_count: row.get::<_, i64>(4)? as u32,
                    unread_count: row.get::<_, i64>(5)? as u32,
                    is_starred: is_starred != 0,
                    participants,
                    has_attachments: has_attachments != 0,
                })
            })?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to list threads: {e}")))?;

    Ok(Json(threads))
}

pub async fn get_thread_messages(
    State(state): State<AppStateRef>,
    Path(thread_id): Path<String>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let store = state.store.clone();

    let messages = store
        .with_read_async(move |conn| {
            let sql = "SELECT id, account_id, remote_id, message_id_header, in_reply_to, \
                 references_header, thread_id, subject, snippet, from_address, \
                 from_name, to_list, cc_list, bcc_list, \
                 body_text, body_html_raw, \
                 has_attachments, is_read, is_starred, is_draft, \
                 date, remote_version, is_deleted, deleted_at, created_at, updated_at \
                 FROM messages WHERE thread_id = ?1 AND is_deleted = 0 ORDER BY date ASC";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params![thread_id], |row| {
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
            })?;
            let mut messages = Vec::new();
            for row in rows {
                messages.push(row?);
            }
            Ok(messages)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get thread messages: {e}")))?;

    Ok(Json(messages))
}
