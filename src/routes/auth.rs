use crate::auth::{create_token, verify_password};
use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    State(state): State<AppStateRef>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    if !verify_password(&body.password, &state.config.password_hash) {
        return Err(ApiError::Unauthorized("Invalid password".to_string()));
    }

    let token = create_token(&state.config.jwt_secret, 7)
        .map_err(|e| ApiError::Internal(format!("Token creation failed: {e}")))?;

    Ok(Json(LoginResponse { token }))
}
