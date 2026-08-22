use axum::Json;
use serde_json::json;

pub async fn api_stats() -> Json<serde_json::Value> {
    Json(json!({ "stats": {} }))
}
