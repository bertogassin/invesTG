use axum::Json;
use serde_json::json;

pub async fn api_vote() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}
