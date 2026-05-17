use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{extract::State, Json};
use pebble_core::traits::SearchHit;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub has_attachment: Option<bool>,
    pub folder_id: Option<String>,
    pub limit: Option<usize>,
}

pub async fn search_messages(
    State(state): State<AppStateRef>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<Vec<SearchHit>>, ApiError> {
    let search = state.search.clone();
    let limit = body.limit.unwrap_or(50);

    // Determine if this is a simple search or advanced
    let is_advanced = body.from.is_some()
        || body.to.is_some()
        || body.subject.is_some()
        || body.date_from.is_some()
        || body.date_to.is_some()
        || body.has_attachment.is_some()
        || body.folder_id.is_some();

    let hits = if is_advanced {
        let text = body.query.clone();
        let from = body.from.clone();
        let to = body.to.clone();
        let subject = body.subject.clone();
        let date_from = body.date_from;
        let date_to = body.date_to;
        let has_attachment = body.has_attachment;
        let folder_id = body.folder_id.clone();

        tokio::task::spawn_blocking(move || {
            use pebble_search::AdvancedSearchParams;
            search.advanced_search(AdvancedSearchParams {
                text: text.as_deref(),
                from: from.as_deref(),
                to: to.as_deref(),
                subject: subject.as_deref(),
                date_from,
                date_to,
                has_attachment,
                folder_id: folder_id.as_deref(),
                limit,
            })
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Search task failed: {e}")))?
        .map_err(|e| ApiError::Internal(format!("Search failed: {e}")))?
    } else {
        let query_text = body.query.unwrap_or_default();
        if query_text.is_empty() {
            return Ok(Json(Vec::new()));
        }
        tokio::task::spawn_blocking(move || search.search(&query_text, limit))
            .await
            .map_err(|e| ApiError::Internal(format!("Search task failed: {e}")))?
            .map_err(|e| ApiError::Internal(format!("Search failed: {e}")))?
    };

    Ok(Json(hits))
}
