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
    let jwt_secret = state.config.jwt_secret.clone();

    let public_routes = Router::new()
        .route("/api/v1/health", get(health::health))
        .route("/api/v1/auth/login", post(auth::login));

    let protected_routes = Router::new()
        .layer(middleware::from_fn(move |req, next| {
            crate::auth::auth_middleware(jwt_secret.clone(), req, next)
        }));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
        .with_state(state)
}
