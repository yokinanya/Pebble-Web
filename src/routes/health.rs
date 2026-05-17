use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn pending_ops_summary() -> Json<Value> {
    Json(json!({
        "total": 0,
        "by_type": {}
    }))
}

pub async fn list_pending_ops() -> Json<Vec<Value>> {
    Json(vec![])
}
