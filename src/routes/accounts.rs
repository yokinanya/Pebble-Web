use crate::credentials::{encrypt_credentials, AccountCredentials};
use crate::error::ApiError;
use crate::state::AppStateRef;
use axum::{
    extract::{Path, State},
    Json,
};
use pebble_core::{new_id, now_timestamp, Account, ProviderType};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub color: Option<String>,
    pub provider: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Account> for AccountResponse {
    fn from(a: Account) -> Self {
        let provider = match a.provider {
            ProviderType::Imap => "imap",
            ProviderType::Gmail => "gmail",
            ProviderType::Outlook => "outlook",
        };
        Self {
            id: a.id,
            email: a.email,
            display_name: a.display_name,
            color: a.color,
            provider: provider.to_string(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub email: String,
    pub display_name: String,
    pub color: Option<String>,
    pub credentials: AccountCredentials,
}

pub async fn list_accounts(
    State(state): State<AppStateRef>,
) -> Result<Json<Vec<AccountResponse>>, ApiError> {
    let store = state.store.clone();
    let accounts = store
        .with_read_async(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, email, display_name, color, provider, created_at, updated_at
                 FROM accounts ORDER BY created_at ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                let provider_str: String = row.get(4)?;
                let provider = match provider_str.as_str() {
                    "gmail" => ProviderType::Gmail,
                    "outlook" => ProviderType::Outlook,
                    _ => ProviderType::Imap,
                };
                Ok(Account {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    display_name: row.get(2)?,
                    color: row.get(3)?,
                    provider,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?;
            let mut accounts = Vec::new();
            for row in rows {
                accounts.push(row?);
            }
            Ok(accounts)
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to list accounts: {e}")))?;

    let response: Vec<AccountResponse> = accounts.into_iter().map(AccountResponse::from).collect();
    Ok(Json(response))
}

pub async fn create_account(
    State(state): State<AppStateRef>,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    let encrypted = encrypt_credentials(&state.crypto, &body.credentials)
        .map_err(|e| ApiError::Internal(format!("Encryption failed: {e}")))?;

    let now = now_timestamp();
    let account = Account {
        id: new_id(),
        email: body.email,
        display_name: body.display_name,
        color: body.color,
        provider: ProviderType::Imap,
        created_at: now,
        updated_at: now,
    };

    let account_clone = account.clone();
    let store = state.store.clone();
    store
        .with_write_async(move |conn| {
            conn.execute(
                "INSERT INTO accounts (id, email, display_name, color, provider, created_at, updated_at, sync_state)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    account_clone.id,
                    account_clone.email,
                    account_clone.display_name,
                    account_clone.color.as_deref(),
                    "imap",
                    account_clone.created_at,
                    account_clone.updated_at,
                    format!(r#"{{"credentials":"{}"}}"#, encrypted),
                ],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create account: {e}")))?;

    Ok(Json(AccountResponse::from(account)))
}

pub async fn delete_account(
    State(state): State<AppStateRef>,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.store.clone();
    store
        .with_write_async(move |conn| {
            conn.execute(
                "DELETE FROM accounts WHERE id = ?1",
                rusqlite::params![account_id],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete account: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
