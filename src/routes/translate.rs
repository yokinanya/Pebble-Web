use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{extract::State, Json};
use pebble_translate::types::TranslateProviderConfig;
use pebble_translate::TranslateService;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub target_lang: String,
    pub source_lang: Option<String>,
}

#[derive(Serialize)]
pub struct TranslateResponse {
    pub translated: String,
}

pub async fn translate(
    State(state): State<AppStateRef>,
    Json(body): Json<TranslateRequest>,
) -> Result<Json<TranslateResponse>, ApiError> {
    let store = state.store.clone();

    // Load translation config from the store
    let config = store
        .get_translate_config()
        .map_err(|e| ApiError::Internal(format!("Failed to load translate config: {e}")))?;

    let config = config.ok_or_else(|| {
        ApiError::BadRequest("Translation is not configured".to_string())
    })?;

    if !config.is_enabled {
        return Err(ApiError::BadRequest(
            "Translation is currently disabled".to_string(),
        ));
    }

    // Parse the provider config from the stored JSON blob
    let provider_config: TranslateProviderConfig =
        serde_json::from_str(&config.config).map_err(|e| {
            ApiError::Internal(format!("Invalid translate provider config: {e}"))
        })?;

    let source_lang = body.source_lang.as_deref().unwrap_or("auto");

    let result = TranslateService::translate(&provider_config, &body.text, source_lang, &body.target_lang)
        .await
        .map_err(|e| ApiError::Internal(format!("Translation failed: {e}")))?;

    Ok(Json(TranslateResponse {
        translated: result.translated,
    }))
}
