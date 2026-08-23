use axum::{
    extract::{Form, Path, Query, State},
    response::Html,
    Json,
};
use serde::{Deserialize};
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

pub async fn app_cat(
    State(state): State<AppState>,
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
) -> Html<String> {
    let db = state.db.lock().await;

    let resources: Vec<(i64, String, String, String, String, f64, i64, i64, i64)> = db
        .prepare(
            "SELECT id, title, description, contact, address, rating, votes, is_verified, is_premium
             FROM resources
             WHERE continent_index = ?1
               AND country_index = ?2
               AND city_index = ?3
               AND category = ?4
               AND is_active = 1
             ORDER BY is_verified DESC, rating DESC, votes DESC, id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(
                rusqlite::params![ci, si, zi, k],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                    ))
                },
            )?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);


    // Premium → проверенные → рейтинг → голоса
    let mut resources = resources;
    resources.sort_by(|a, b| {
        b.8.cmp(&a.8)
            .then_with(|| b.7.cmp(&a.7))
            .then_with(|| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| b.6.cmp(&a.6))
    });

    Html(templates::render_category(
        ci,
        si,
        zi,
        &k,
        resources,
    ))
}


// ============================================================
// ДОБАВЛЕНИЕ РЕСУРСА
// ============================================================

#[derive(Debug, Deserialize)]
pub struct AddResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
}


// ============================================================
// ПРОФИЛЬ РЕСУРСА
// ============================================================
pub async fn resource_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Html<String> {
    let db = state.db.lock().await;

    let resource = db.query_row(
        "SELECT title, description, contact, address,
                rating, votes, is_premium, is_verified,
                category, created_at
         FROM resources
         WHERE id = ?1 AND is_active = 1",
        rusqlite::params![id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i64>(9)?,
            ))
        },
    ).ok();

    drop(db);

    match resource {
        Some((
            title,
            description,
            contact,
            address,
            rating,
            votes,
            premium,
            verified,
            category,
            created_at,
        )) => Html(templates::render_resource_profile(
            id,
            &title,
            &description,
            &contact,
            &address,
            rating,
            votes,
            premium,
            verified,
            &category,
            created_at,
        )),

        None => Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ресурс не найден · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
<section class="hero">
    <div class="eyebrow">⚠ ResursMap</div>
    <h1>Ресурс не найден</h1>
    <p>Этот ресурс больше недоступен или был удалён.</p>
    <a class="card" href="/app" style="text-decoration:none;margin-top:24px;">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Вернуться на карту</div>
            <div class="card-meta">Открыть ResursMap</div>
        </div>
        <div class="card-arrow">›</div>
    </a>
</section>
</main>
</body>
</html>"#,
            templates::base_style(),
            templates::icon("map"),
        )),
    }
}

pub async fn add_resource_page(
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
) -> Html<String> {
    Html(templates::render_add_resource(ci, si, zi, &k))
}

pub async fn add_resource(
    State(state): State<AppState>,
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
    Form(form): Form<AddResourceForm>,
) -> Html<String> {
    let db = state.db.lock().await;

    let result = db.execute(
        "INSERT INTO resources
        (client_id, continent_index, country_index, city_index,
         category, title, description, contact, address,
         rating, votes, is_premium, is_verified, is_active,
         created_at, updated_at)
        VALUES
        ('', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
         0, 0, 0, 0, 1, strftime('%s','now'), strftime('%s','now'))",
        rusqlite::params![
            ci,
            si,
            zi,
            k,
            form.title.trim(),
            form.description.trim(),
            form.contact.trim(),
            form.address.trim(),
        ],
    );

    drop(db);

    match result {
        Ok(_) => Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ресурс добавлен · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
    <section class="hero">
        <div class="eyebrow">✓ Готово</div>
        <h1>Ресурс добавлен</h1>
        <p>Спасибо! Ресурс появился в категории и ожидает проверки.</p>

        <a class="card"
           href="/app/{}/{}/{}/cat/{}"
           style="text-decoration:none;margin-top:24px;">
            <div class="card-icon">{}</div>
            <div class="card-content">
                <div class="card-title">Вернуться к ресурсам</div>
                <div class="card-meta">Открыть категорию</div>
            </div>
            <div class="card-arrow">›</div>
        </a>
    </section>
</main>
</body>
</html>"#,
            templates::base_style(),
            ci,
            si,
            zi,
            k,
            templates::icon("map"),
        )),
        Err(_) => Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ошибка · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
    <section class="hero">
        <div class="eyebrow">⚠ Ошибка</div>
        <h1>Не удалось добавить ресурс</h1>
        <p>Попробуйте ещё раз.</p>
        <a href="/app/{}/{}/{}/cat/{}">Назад</a>
    </section>
</main>
</body>
</html>"#,
            templates::base_style(),
            ci,
            si,
            zi,
            k,
        )),
    }
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
