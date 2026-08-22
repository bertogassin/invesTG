use axum::{
    extract::{Path, Query, State},
    response::Html,
    Json,
};
use serde_json::json;
use std::collections::BTreeMap;
use crate::state::app_state::AppState;
use super::templates;

// ------------------------------------------------------------
// HTML-страницы
// ------------------------------------------------------------
pub async fn home() -> Html<String> {
    Html("<h1>ResursMap</h1><p>Сервер работает.</p>".to_string())
}

pub async fn app_root() -> Html<String> {
    Html(templates::render_continents())
}

pub async fn app_search(Query(params): Query<BTreeMap<String, String>>) -> Html<String> {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    Html(templates::render_search(q))
}

pub async fn app_me() -> Html<String> {
    Html(templates::render_me())
}

pub async fn app_continent(Path(ci): Path<usize>) -> Html<String> {
    Html(templates::render_continent(ci))
}

pub async fn app_country(Path((ci, si)): Path<(usize, usize)>) -> Html<String> {
    Html(templates::render_country(ci, si))
}

pub async fn app_city(Path((ci, si, zi)): Path<(usize, usize, usize)>) -> Html<String> {
    Html(templates::render_city(ci, si, zi))
}

pub async fn app_cat(Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>) -> Html<String> {
    Html(templates::render_category(ci, si, zi, &k))
}

pub async fn health() -> &'static str {
    "ok"
}

// ------------------------------------------------------------
// API-обработчики (реальная логика)
// ------------------------------------------------------------
pub async fn api_vote(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let client_id = payload.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
    let city = payload.get("city").and_then(|v| v.as_str()).unwrap_or("");
    let category = payload.get("category").and_then(|v| v.as_str()).unwrap_or("");
    let mark = payload.get("mark").and_then(|v| v.as_str()).unwrap_or("");

    let _ = db.execute(
        "INSERT OR REPLACE INTO votes (client_id, city, category, mark, updated_at) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))",
        rusqlite::params![client_id, city, category, mark],
    );

    Json(json!({ "ok": true }))
}

pub async fn api_points(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let client_id = params.get("client_id").map(|s| s.as_str()).unwrap_or("");

    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM votes WHERE client_id = ?1",
        rusqlite::params![client_id],
        |row| row.get(0),
    ).unwrap_or(0);

    Json(json!({ "points": count }))
}

pub async fn api_my(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let client_id = params.get("client_id").map(|s| s.as_str()).unwrap_or("");

    let rows = db.prepare(
        "SELECT city, category, mark FROM votes WHERE client_id = ?1"
    ).unwrap().query_map(rusqlite::params![client_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap_or(vec![]);

    Json(json!({ "votes": rows }))
}

pub async fn api_stats(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let city = params.get("city").map(|s| s.as_str()).unwrap_or("");

    let stats: Vec<(String, i64)> = db.prepare(
        "SELECT category, COUNT(*) FROM votes WHERE city = ?1 GROUP BY category"
    ).unwrap().query_map(rusqlite::params![city], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap_or(vec![]);

    Json(json!({ "stats": stats }))
}

pub async fn api_profile_get(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let client_id = params.get("client_id").map(|s| s.as_str()).unwrap_or("");

    let profile: Option<(String, i64, String, i64)> = db.query_row(
        "SELECT username, open_contact, intent_text, intent_until FROM profiles WHERE client_id = ?1",
        rusqlite::params![client_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).ok();

    if let Some((username, open_contact, intent_text, intent_until)) = profile {
        Json(json!({
            "username": username,
            "open_contact": open_contact == 1,
            "intent_text": intent_text,
            "intent_until": intent_until
        }))
    } else {
        Json(json!({ "username": "", "open_contact": false, "intent_text": "", "intent_until": 0 }))
    }
}

pub async fn api_profile_set(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let client_id = payload.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
    let username = payload.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let open_contact = payload.get("open_contact").and_then(|v| v.as_bool()).unwrap_or(false);
    let intent_text = payload.get("intent_text").and_then(|v| v.as_str()).unwrap_or("");
    let intent_until = payload.get("intent_until").and_then(|v| v.as_i64()).unwrap_or(0);

    let _ = db.execute(
        "INSERT OR REPLACE INTO profiles (client_id, username, open_contact, intent_text, intent_until, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s','now'))",
        rusqlite::params![client_id, username, open_contact, intent_text, intent_until],
    );

    Json(json!({ "ok": true }))
}

pub async fn api_open_count(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().await;
    let city = params.get("city").map(|s| s.as_str()).unwrap_or("");

    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM profiles WHERE open_contact = 1 AND city = ?1",
        rusqlite::params![city],
        |row| row.get(0),
    ).unwrap_or(0);

    Json(json!({ "count": count }))
}

pub async fn webhook_handler(
    State(_state): State<AppState>,
    Query(_params): Query<serde_json::Value>,
) -> String {
    "OK".to_string()
}
