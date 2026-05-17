# Pebble Web Phase 1: Skeleton Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a compilable, runnable Axum server that serves static files, protects routes with JWT auth, and loads configuration from environment variables — forming the foundation for all subsequent phases.

**Architecture:** Axum HTTP server with tower middleware for JWT auth. Config loaded from env vars. Copied crates from Pebble desktop provide store/mail/search/crypto capabilities. The `pebble-crypto` crate is modified to read DEK from env/file instead of OS keyring.

**Tech Stack:** Rust (Axum, tower, jsonwebtoken, argon2), SQLite (via pebble-store), Vite + React (frontend scaffold)

---

## File Structure

```
pebble-web/
├── Cargo.toml                      # workspace root
├── Cargo.lock
├── .gitignore
├── .env.example
├── Dockerfile
├── docker-compose.yml
├── crates/
│   ├── pebble-core/                # copied from Pebble
│   ├── pebble-store/               # copied from Pebble
│   ├── pebble-mail/                # copied from Pebble
│   ├── pebble-search/              # copied from Pebble
│   ├── pebble-translate/           # copied from Pebble
│   ├── pebble-crypto/              # copied + modified (keystore.rs → env/file)
│   └── pebble-oauth/               # copied from Pebble
├── src/
│   ├── main.rs                     # entry point: load config, init state, start server
│   ├── config.rs                   # Config struct, load from env
│   ├── auth.rs                     # password hashing, JWT creation/validation, middleware
│   ├── state.rs                    # AppState shared across handlers
│   ├── error.rs                    # API error types → JSON responses
│   └── routes/
│       ├── mod.rs                  # router assembly
│       ├── health.rs               # GET /api/v1/health
│       └── auth.rs                 # POST /api/v1/auth/login, /logout
└── frontend/
    ��── package.json
    ├── vite.config.ts
    ├── index.html
    ├── tsconfig.json
    └── src/
        ├── main.tsx
        ├── App.tsx
        └── api-client.ts           # axios instance with JWT interceptor
```

---

### Task 1: Copy Crates and Create Workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `src/main.rs` (placeholder)
- Create: `.gitignore`
- Copy: `crates/` (all 7 crates from Pebble)

- [ ] **Step 1: Copy crate directories from Pebble**

```bash
cp -r d:/project/Pebble/crates/pebble-core d:/project/Pebble-Web/crates/pebble-core
cp -r d:/project/Pebble/crates/pebble-store d:/project/Pebble-Web/crates/pebble-store
cp -r d:/project/Pebble/crates/pebble-mail d:/project/Pebble-Web/crates/pebble-mail
cp -r d:/project/Pebble/crates/pebble-search d:/project/Pebble-Web/crates/pebble-search
cp -r d:/project/Pebble/crates/pebble-translate d:/project/Pebble-Web/crates/pebble-translate
cp -r d:/project/Pebble/crates/pebble-crypto d:/project/Pebble-Web/crates/pebble-crypto
cp -r d:/project/Pebble/crates/pebble-oauth d:/project/Pebble-Web/crates/pebble-oauth
```

- [ ] **Step 2: Create .gitignore**

```gitignore
/target
node_modules/
frontend/dist/
.env
*.swp
*.swo
```

- [ ] **Step 3: Create workspace Cargo.toml**

```toml
[workspace]
members = [
    "crates/pebble-core",
    "crates/pebble-store",
    "crates/pebble-mail",
    "crates/pebble-search",
    "crates/pebble-translate",
    "crates/pebble-crypto",
    "crates/pebble-oauth",
]
resolver = "2"

[workspace.package]
name = "pebble-web"
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.39", features = ["bundled"] }
async-imap = { version = "0.10", default-features = false, features = ["runtime-tokio"] }
tokio-rustls = "0.26"
rustls = { version = "0.23", features = ["aws_lc_rs", "tls12"] }
webpki-roots = "0.26"
mail-parser = "0.9"
tantivy = "0.22"
ammonia = "4"
lol_html = "1"
lettre = { version = "0.11", default-features = false, features = ["tokio1-rustls-tls", "smtp-transport", "builder"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "socks"] }
tokio-socks = "0.5"
aes-gcm = "0.10"
rand = "0.8"
semver = "1"
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "cors", "trace"] }
jsonwebtoken = "9"
argon2 = "0.5"
```

- [ ] **Step 4: Create placeholder main.rs**

Create `src/main.rs`:

```rust
fn main() {
    println!("pebble-web starting...");
}
```

- [ ] **Step 5: Verify workspace compiles (crates only)**

```bash
cd d:/project/Pebble-Web && cargo check -p pebble-core -p pebble-store -p pebble-search -p pebble-translate
```

Expected: successful compilation of crates that have no OS-specific deps. `pebble-crypto` will fail due to `keyring` — that's fixed in the next task.

- [ ] **Step 6: Commit**

```bash
cd d:/project/Pebble-Web
git add -A
git commit -m "feat: init workspace with copied crates from Pebble desktop"
```

---

### Task 2: Adapt pebble-crypto to File/Env-Based Key Storage

**Files:**
- Modify: `crates/pebble-crypto/Cargo.toml`
- Rewrite: `crates/pebble-crypto/src/keystore.rs`
- Modify: `crates/pebble-crypto/src/lib.rs`

- [ ] **Step 1: Update Cargo.toml — remove keyring, add hex**

Replace `crates/pebble-crypto/Cargo.toml` with:

```toml
[package]
name = "pebble-crypto"
version = "0.1.0"
edition = "2021"

[dependencies]
pebble-core = { path = "../pebble-core" }
aes-gcm = { workspace = true }
rand = { workspace = true }
tracing = { workspace = true }
zeroize = { version = "1", features = ["zeroize_derive"] }
hex = "0.4"
```

- [ ] **Step 2: Rewrite keystore.rs to read DEK from env or file**

Replace `crates/pebble-crypto/src/keystore.rs` with:

```rust
use pebble_core::{PebbleError, Result};
use rand::RngCore;
use std::fs;
use std::path::Path;
use tracing::info;
use zeroize::Zeroizing;

pub struct KeyStore;

impl KeyStore {
    /// Load DEK from environment variable PEBBLE_ENCRYPTION_KEY (hex-encoded 32 bytes),
    /// or from a file at the given path. If neither exists, generate and save to file.
    pub fn get_or_create_dek(key_file_path: Option<&Path>) -> Result<Zeroizing<[u8; 32]>> {
        // Priority 1: environment variable
        if let Ok(hex_key) = std::env::var("PEBBLE_ENCRYPTION_KEY") {
            let bytes = hex::decode(hex_key.trim())
                .map_err(|e| PebbleError::Auth(format!("Invalid PEBBLE_ENCRYPTION_KEY hex: {e}")))?;
            if bytes.len() != 32 {
                return Err(PebbleError::Auth(format!(
                    "PEBBLE_ENCRYPTION_KEY must be 32 bytes, got {}",
                    bytes.len()
                )));
            }
            let mut key = Zeroizing::new([0u8; 32]);
            key.copy_from_slice(&bytes);
            return Ok(key);
        }

        // Priority 2: key file
        let key_path = key_file_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new("/data/encryption.key").to_path_buf());

        if key_path.exists() {
            let hex_key = fs::read_to_string(&key_path)
                .map_err(|e| PebbleError::Auth(format!("Failed to read key file: {e}")))?;
            let bytes = hex::decode(hex_key.trim())
                .map_err(|e| PebbleError::Auth(format!("Invalid key file hex: {e}")))?;
            if bytes.len() != 32 {
                return Err(PebbleError::Auth(format!(
                    "Key file must contain 32 bytes, got {}",
                    bytes.len()
                )));
            }
            let mut key = Zeroizing::new([0u8; 32]);
            key.copy_from_slice(&bytes);
            return Ok(key);
        }

        // Generate new key and save to file
        info!("No DEK found, generating new one at {:?}", key_path);
        let mut key = Zeroizing::new([0u8; 32]);
        rand::thread_rng().fill_bytes(&mut *key);

        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PebbleError::Auth(format!("Failed to create key dir: {e}")))?;
        }
        fs::write(&key_path, hex::encode(&*key))
            .map_err(|e| PebbleError::Auth(format!("Failed to write key file: {e}")))?;

        Ok(key)
    }
}
```

- [ ] **Step 3: Update lib.rs to accept key file path**

Replace `crates/pebble-crypto/src/lib.rs` with:

```rust
pub mod aes;
pub mod keystore;

use pebble_core::Result;
use std::path::Path;
use zeroize::Zeroizing;

pub struct CryptoService {
    dek: Zeroizing<[u8; 32]>,
}

impl CryptoService {
    /// Initialize by loading (or creating) the DEK from env or file.
    pub fn init(key_file_path: Option<&Path>) -> Result<Self> {
        let dek = keystore::KeyStore::get_or_create_dek(key_file_path)?;
        Ok(Self { dek })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        aes::encrypt(&self.dek, plaintext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        aes::decrypt(&self.dek, ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_crypto_service_init_with_env() {
        let key = [0xABu8; 32];
        env::set_var("PEBBLE_ENCRYPTION_KEY", hex::encode(key));
        let service = CryptoService::init(None).unwrap();
        let encrypted = service.encrypt(b"hello").unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"hello");
        env::remove_var("PEBBLE_ENCRYPTION_KEY");
    }

    #[test]
    fn test_crypto_service_init_generates_file() {
        env::remove_var("PEBBLE_ENCRYPTION_KEY");
        let tmp = TempDir::new().unwrap();
        let key_path = tmp.path().join("encryption.key");
        let service = CryptoService::init(Some(&key_path)).unwrap();
        assert!(key_path.exists());
        let encrypted = service.encrypt(b"test").unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"test");
    }
}
```

- [ ] **Step 4: Add tempfile dev-dependency**

Add to `crates/pebble-crypto/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 5: Verify pebble-crypto compiles and tests pass**

```bash
cd d:/project/Pebble-Web && cargo test -p pebble-crypto
```

Expected: all tests pass (including the two new integration tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pebble-crypto/
git commit -m "feat(crypto): replace OS keyring with env/file-based key storage"
```

---

### Task 3: Config Module

**Files:**
- Create: `src/config.rs`
- Create: `.env.example`

- [ ] **Step 1: Create .env.example**

```env
# Required
PEBBLE_PASSWORD=changeme
PEBBLE_JWT_SECRET=generate-a-random-string-here

# Optional
PEBBLE_DATA_DIR=/data
PEBBLE_PORT=8080
PEBBLE_SYNC_INTERVAL=300
PEBBLE_ENCRYPTION_KEY=
```

- [ ] **Step 2: Create src/config.rs**

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub password_hash: String,
    pub jwt_secret: String,
    pub sync_interval_secs: u64,
    pub encryption_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let password = std::env::var("PEBBLE_PASSWORD")
            .map_err(|_| "PEBBLE_PASSWORD env var is required")?;

        let jwt_secret = std::env::var("PEBBLE_JWT_SECRET")
            .map_err(|_| "PEBBLE_JWT_SECRET env var is required")?;

        let port = std::env::var("PEBBLE_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid PEBBLE_PORT: {e}"))?;

        let data_dir = PathBuf::from(
            std::env::var("PEBBLE_DATA_DIR").unwrap_or_else(|_| "/data".to_string()),
        );

        let sync_interval_secs = std::env::var("PEBBLE_SYNC_INTERVAL")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid PEBBLE_SYNC_INTERVAL: {e}"))?;

        let encryption_key = std::env::var("PEBBLE_ENCRYPTION_KEY").ok();

        let password_hash = crate::auth::hash_password(&password)
            .map_err(|e| format!("Failed to hash password: {e}"))?;

        Ok(Self {
            port,
            data_dir,
            password_hash,
            jwt_secret,
            sync_interval_secs,
            encryption_key,
        })
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("pebble.db")
    }

    pub fn index_dir(&self) -> PathBuf {
        self.data_dir.join("index")
    }

    pub fn attachments_dir(&self) -> PathBuf {
        self.data_dir.join("attachments")
    }

    pub fn key_file_path(&self) -> PathBuf {
        self.data_dir.join("encryption.key")
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/config.rs .env.example
git commit -m "feat: add config module with env var loading"
```

---

### Task 4: Auth Module (Password Hashing + JWT)

**Files:**
- Create: `src/auth.rs`

- [ ] **Step 1: Create src/auth.rs**

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn create_token(secret: &str, expiry_days: u64) -> Result<String, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        iat: now,
        exp: now + (expiry_days * 24 * 60 * 60) as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let state = request
        .extensions()
        .get::<crate::state::AppStateRef>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    validate_token(token, &state.config.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}
```

- [ ] **Step 2: Commit**

```bash
git add src/auth.rs
git commit -m "feat: add auth module with argon2 hashing and JWT"
```

---

### Task 5: Error Module and AppState

**Files:**
- Create: `src/error.rs`
- Create: `src/state.rs`

- [ ] **Step 1: Create src/error.rs**

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

- [ ] **Step 2: Create src/state.rs**

```rust
use crate::config::Config;
use pebble_crypto::CryptoService;
use pebble_search::TantivySearch;
use pebble_store::Store;
use std::path::PathBuf;
use std::sync::Arc;

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub store: Arc<Store>,
    pub search: Arc<TantivySearch>,
    pub crypto: Arc<CryptoService>,
    pub attachments_dir: PathBuf,
}

impl AppState {
    pub fn init(config: Config) -> Result<Self, String> {
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| format!("Failed to create data dir: {e}"))?;
        std::fs::create_dir_all(config.attachments_dir())
            .map_err(|e| format!("Failed to create attachments dir: {e}"))?;

        let store = Store::open(&config.db_path())
            .map_err(|e| format!("Failed to open store: {e}"))?;

        let search = TantivySearch::open_or_create(&config.index_dir())
            .map_err(|e| format!("Failed to open search index: {e}"))?;

        let key_file = config.key_file_path();
        let crypto = CryptoService::init(Some(&key_file))
            .map_err(|e| format!("Failed to init crypto: {e}"))?;

        let attachments_dir = config.attachments_dir();

        Ok(Self {
            config,
            store: Arc::new(store),
            search: Arc::new(search),
            crypto: Arc::new(crypto),
            attachments_dir,
        })
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/error.rs src/state.rs
git commit -m "feat: add API error types and AppState initialization"
```

---

### Task 6: Routes — Health and Auth

**Files:**
- Create: `src/routes/mod.rs`
- Create: `src/routes/health.rs`
- Create: `src/routes/auth.rs`

- [ ] **Step 1: Create src/routes/health.rs**

```rust
use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

- [ ] **Step 2: Create src/routes/auth.rs**

```rust
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
```

- [ ] **Step 3: Create src/routes/mod.rs**

```rust
pub mod auth;
pub mod health;

use crate::state::AppStateRef;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

pub fn build_router(state: AppStateRef, static_dir: &str) -> Router {
    let public_routes = Router::new()
        .route("/api/v1/health", get(health::health))
        .route("/api/v1/auth/login", post(auth::login));

    let protected_routes = Router::new()
        // Future protected routes go here
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware_with_state,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
        .with_state(state)
}

async fn auth_middleware_with_state(
    State(state): axum::extract::State<AppStateRef>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    request.extensions_mut().insert(state.clone());
    crate::auth::auth_middleware(request, next).await
}
```

- [ ] **Step 4: Commit**

```bash
git add src/routes/
git commit -m "feat: add health check and login routes"
```

---

### Task 7: Main Entry Point

**Files:**
- Modify: `Cargo.toml` (add root package)
- Rewrite: `src/main.rs`

- [ ] **Step 1: Add root package to workspace Cargo.toml**

Add at the end of the workspace `Cargo.toml`:

```toml
[package]
name = "pebble-web"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
jsonwebtoken = { workspace = true }
argon2 = { workspace = true }
pebble-core = { path = "crates/pebble-core" }
pebble-store = { path = "crates/pebble-store" }
pebble-search = { path = "crates/pebble-search" }
pebble-crypto = { path = "crates/pebble-crypto" }
```

Also add `"."` to the workspace members list.

- [ ] **Step 2: Write src/main.rs**

```rust
mod auth;
mod config;
mod error;
mod routes;
mod state;

use crate::config::Config;
use crate::state::{AppState, AppStateRef};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("pebble_web=info".parse().unwrap()))
        .init();

    let config = Config::from_env().expect("Failed to load config");
    let port = config.port;

    let state = AppState::init(config).expect("Failed to initialize app state");
    let state: AppStateRef = Arc::new(state);

    let static_dir = std::env::var("PEBBLE_STATIC_DIR")
        .unwrap_or_else(|_| "/usr/local/share/pebble-web/static".to_string());

    let app = routes::build_router(state, &static_dir);

    let addr = format!("0.0.0.0:{port}");
    info!("Pebble Web listening on {addr}");
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
```

- [ ] **Step 3: Verify full workspace compiles**

```bash
cd d:/project/Pebble-Web && cargo check
```

Expected: successful compilation.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: add Axum server entry point with auth and static serving"
```

---

### Task 8: Frontend Scaffold

**Files:**
- Create: `frontend/package.json`
- Create: `frontend/vite.config.ts`
- Create: `frontend/tsconfig.json`
- Create: `frontend/index.html`
- Create: `frontend/src/main.tsx`
- Create: `frontend/src/App.tsx`
- Create: `frontend/src/api-client.ts`

- [ ] **Step 1: Create frontend/package.json**

```json
{
  "name": "pebble-web-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "axios": "^1.7.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0"
  },
  "devDependencies": {
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }
}
```

- [ ] **Step 2: Create frontend/vite.config.ts**

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
    },
  },
});
```

- [ ] **Step 3: Create frontend/tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

- [ ] **Step 4: Create frontend/index.html**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Pebble Web</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 5: Create frontend/src/main.tsx**

```tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

- [ ] **Step 6: Create frontend/src/App.tsx**

```tsx
function App() {
  return (
    <div>
      <h1>Pebble Web</h1>
      <p>Email client is loading...</p>
    </div>
  );
}

export default App;
```

- [ ] **Step 7: Create frontend/src/api-client.ts**

```typescript
import axios from 'axios';

const api = axios.create({
  baseURL: '/api/v1',
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('pebble_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('pebble_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export default api;

export async function login(password: string): Promise<string> {
  const { data } = await api.post('/auth/login', { password });
  localStorage.setItem('pebble_token', data.token);
  return data.token;
}

export function logout(): void {
  localStorage.removeItem('pebble_token');
  window.location.href = '/login';
}

export function isAuthenticated(): boolean {
  return !!localStorage.getItem('pebble_token');
}
```

- [ ] **Step 8: Install frontend dependencies and verify build**

```bash
cd d:/project/Pebble-Web/frontend && npm install && npm run build
```

Expected: `dist/` directory created with built assets.

- [ ] **Step 9: Commit**

```bash
cd d:/project/Pebble-Web
git add frontend/
git commit -m "feat: add React frontend scaffold with api-client"
```

---

### Task 9: Dockerfile and docker-compose.yml

**Files:**
- Create: `Dockerfile`
- Create: `docker-compose.yml`

- [ ] **Step 1: Create Dockerfile**

```dockerfile
# Stage 1: Build React frontend
FROM node:20-alpine AS frontend
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# Stage 2: Build Rust backend
FROM rust:1.80-alpine AS backend
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY src/ src/
RUN cargo build --release

# Stage 3: Final minimal image
FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=backend /app/target/release/pebble-web /usr/local/bin/
COPY --from=frontend /app/dist /usr/local/share/pebble-web/static
EXPOSE 8080
VOLUME /data
ENV PEBBLE_DATA_DIR=/data
ENV PEBBLE_STATIC_DIR=/usr/local/share/pebble-web/static
CMD ["pebble-web"]
```

- [ ] **Step 2: Create docker-compose.yml**

```yaml
services:
  pebble-web:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - pebble-data:/data
    environment:
      - PEBBLE_PASSWORD=changeme
      - PEBBLE_JWT_SECRET=change-this-to-a-random-string
      - PEBBLE_DATA_DIR=/data
      - PEBBLE_PORT=8080
      - PEBBLE_SYNC_INTERVAL=300
    restart: unless-stopped

volumes:
  pebble-data:
```

- [ ] **Step 3: Commit**

```bash
git add Dockerfile docker-compose.yml
git commit -m "feat: add Dockerfile and docker-compose for deployment"
```

---

### Task 10: Final Verification and Push

- [ ] **Step 1: Full cargo check**

```bash
cd d:/project/Pebble-Web && cargo check
```

Expected: success.

- [ ] **Step 2: Run all tests**

```bash
cd d:/project/Pebble-Web && cargo test
```

Expected: all tests pass (pebble-crypto tests + any existing crate tests).

- [ ] **Step 3: Verify frontend builds**

```bash
cd d:/project/Pebble-Web/frontend && npm run build
```

Expected: `dist/` created successfully.

- [ ] **Step 4: Push to remote**

```bash
cd d:/project/Pebble-Web && git push -u origin master
```
