pub mod accounts;
pub mod attachments;
pub mod auth;
pub mod compose;
pub mod folders;
pub mod health;
pub mod labels;
pub mod messages;
pub mod search;
pub mod sync;
pub mod threads;
pub mod translate;

use crate::state::AppStateRef;
use crate::ws;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

pub fn build_router(state: AppStateRef, static_dir: &str) -> Router {
    let jwt_secret = state.config.jwt_secret.clone();

    let public_routes = Router::new()
        .route("/api/v1/health", get(health::health))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/ws", get(ws::ws_handler));

    let protected_routes = Router::new()
        // Accounts
        .route("/api/v1/accounts", get(accounts::list_accounts))
        .route("/api/v1/accounts", post(accounts::create_account))
        .route("/api/v1/accounts/{account_id}", put(accounts::update_account).delete(accounts::delete_account))
        .route("/api/v1/accounts/{account_id}/test-connection", post(accounts::test_account_connection))
        .route("/api/v1/test-imap-connection", post(accounts::test_imap_connection))
        // Folders
        .route("/api/v1/accounts/{account_id}/folders", get(folders::list_folders))
        .route("/api/v1/accounts/{account_id}/folder-unread-counts", get(folders::get_folder_unread_counts))
        // Messages
        .route("/api/v1/folders/{folder_id}/messages", get(messages::list_messages_by_folder))
        .route("/api/v1/messages/{message_id}", get(messages::get_message))
        .route("/api/v1/messages/{message_id}/with-html", post(messages::get_message_with_html))
        .route("/api/v1/messages/{message_id}/render", post(messages::render_html))
        .route("/api/v1/messages/{message_id}/flags", put(messages::update_message_flags))
        .route("/api/v1/messages/{message_id}/move", post(messages::move_message))
        .route("/api/v1/messages/{message_id}/archive", post(messages::archive_message))
        .route("/api/v1/messages/{message_id}/restore", post(messages::restore_message))
        .route("/api/v1/messages/{message_id}", delete(messages::delete_message))
        .route("/api/v1/accounts/{account_id}/starred", get(messages::list_starred_messages))
        .route("/api/v1/accounts/{account_id}/empty-trash", post(messages::empty_trash))
        // Batch operations
        .route("/api/v1/messages/batch", post(messages::get_messages_batch))
        .route("/api/v1/messages/batch/archive", post(messages::batch_archive))
        .route("/api/v1/messages/batch/delete", post(messages::batch_delete))
        .route("/api/v1/messages/batch/mark-read", post(messages::batch_mark_read))
        .route("/api/v1/messages/batch/star", post(messages::batch_star))
        // Threads
        .route("/api/v1/folders/{folder_id}/threads", get(threads::list_threads_by_folder))
        .route("/api/v1/threads/{thread_id}/messages", get(threads::get_thread_messages))
        // Search
        .route("/api/v1/search", post(search::search_messages))
        // Attachments
        .route("/api/v1/messages/{message_id}/attachments", get(attachments::list_attachments))
        .route("/api/v1/attachments/{attachment_id}/download", get(attachments::download_attachment))
        // Compose
        .route("/api/v1/compose", post(compose::send_email))
        .route("/api/v1/compose/attachment", post(compose::stage_attachment))
        // Translate
        .route("/api/v1/translate", post(translate::translate))
        .route("/api/v1/translate/config", get(translate::get_translate_config).post(translate::save_translate_config))
        .route("/api/v1/translate/test", post(translate::test_translate_connection))
        // Labels
        .route("/api/v1/labels", get(labels::list_labels))
        .route("/api/v1/labels", post(labels::create_label))
        .route("/api/v1/labels/{id}", delete(labels::delete_label))
        .route("/api/v1/messages/{id}/labels", post(labels::add_label_to_message))
        .route("/api/v1/messages/{id}/labels/{label_id}", delete(labels::remove_label_from_message))
        // Pending Ops
        .route("/api/v1/pending-ops/summary", get(health::pending_ops_summary))
        .route("/api/v1/pending-ops", get(health::list_pending_ops))
        // Sync
        .route("/api/v1/sync/trigger", post(sync::trigger_sync))
        .layer(middleware::from_fn(move |req, next| {
            crate::auth::auth_middleware(jwt_secret.clone(), req, next)
        }));

    let index_path = format!("{static_dir}/index.html");
    let spa_fallback = ServeDir::new(static_dir)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(index_path));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback_service(spa_fallback)
        .with_state(state)
}
