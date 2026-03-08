pub mod admin;
pub mod notes;
pub mod shares;
pub mod todos;

use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
