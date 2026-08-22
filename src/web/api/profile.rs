use axum::Json;
use serde_json::json;

pub async fn api_profile() -> Json<serde_json::Value> {
    Json(json!({ "profile": null }))
}
