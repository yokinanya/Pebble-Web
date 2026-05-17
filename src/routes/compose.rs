use crate::credentials::{decrypt_credentials, AccountCredentials};
use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{extract::State, Json};
use pebble_mail::{ConnectionSecurity, SmtpConfig};
use pebble_mail::smtp::SmtpSender;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ComposeRequest {
    pub account_id: String,
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub attachment_paths: Option<Vec<String>>,
}

pub async fn send_email(
    State(state): State<AppStateRef>,
    Json(body): Json<ComposeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get account and decrypt credentials
    let store = state.store.clone();
    let account_id = body.account_id.clone();

    let (email, sync_state_json) = store
        .with_read_async(move |conn| {
            let row = conn.query_row(
                "SELECT email, sync_state FROM accounts WHERE id = ?1",
                rusqlite::params![account_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )?;
            Ok(row)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get account: {e}")))?;

    let sync_state_str = sync_state_json
        .ok_or_else(|| ApiError::BadRequest("Account has no credentials configured".to_string()))?;

    // Extract encrypted credentials from sync_state
    let sync_state: serde_json::Value = serde_json::from_str(&sync_state_str)
        .map_err(|e| ApiError::Internal(format!("Invalid sync_state: {e}")))?;

    let encrypted_hex = sync_state["credentials"]
        .as_str()
        .ok_or_else(|| ApiError::BadRequest("No credentials found in account".to_string()))?;

    let creds = decrypt_credentials(&state.crypto, encrypted_hex)
        .map_err(|e| ApiError::Internal(format!("Failed to decrypt credentials: {e}")))?;

    let smtp_config = match &creds {
        AccountCredentials::Imap { smtp, .. } => SmtpConfig {
            host: smtp.host.clone(),
            port: smtp.port,
            username: smtp.username.clone(),
            password: smtp.password.clone(),
            security: match smtp.security.as_str() {
                "starttls" => ConnectionSecurity::StartTls,
                "plain" => ConnectionSecurity::Plain,
                _ => ConnectionSecurity::Tls,
            },
            proxy: None,
        },
    };

    let sender = SmtpSender::new(
        smtp_config.host,
        smtp_config.port,
        smtp_config.username,
        smtp_config.password,
        smtp_config.security,
        smtp_config.proxy,
    );

    let cc = body.cc.unwrap_or_default();
    let bcc = body.bcc.unwrap_or_default();
    let attachment_paths = body.attachment_paths.unwrap_or_default();

    sender
        .send(
            &email,
            &body.to,
            &cc,
            &bcc,
            &body.subject,
            &body.body_text,
            body.body_html.as_deref(),
            body.in_reply_to.as_deref(),
            &attachment_paths,
        )
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to send email: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
