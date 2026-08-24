mod favorites;
pub use favorites::*;

mod notifications;
pub use notifications::*;

mod contacts;
pub use contacts::*;

mod chat;
pub use chat::*;

mod profiles;
pub use profiles::*;

mod resources;
pub use resources::*;

mod admin;
pub use admin::*;

use super::templates;
use crate::state::app_state::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_telegram_init_data(init_data: &str, bot_token: &str) -> Option<i64> {
    let mut hash_from_telegram = String::new();
    let mut pairs: Vec<(String, String)> = Vec::new();

    for part in init_data.split('&') {
        let mut iter = part.splitn(2, '=');

        let key = iter.next().unwrap_or("");
        let value = iter.next().unwrap_or("");

        let key = urlencoding::decode(key).ok()?.to_string();
        let value = urlencoding::decode(value).ok()?.to_string();

        if key == "hash" {
            hash_from_telegram = value;
        } else {
            pairs.push((key, value));
        }
    }

    if hash_from_telegram.is_empty() {
        return None;
    }

    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let data_check_string = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    type HmacSha256 = Hmac<Sha256>;

    let mut secret_mac = HmacSha256::new_from_slice(b"WebAppData").ok()?;
    secret_mac.update(bot_token.as_bytes());

    let secret_key = secret_mac.finalize().into_bytes();

    let mut check_mac = HmacSha256::new_from_slice(&secret_key).ok()?;
    check_mac.update(data_check_string.as_bytes());

    let calculated_hash = hex::encode(check_mac.finalize().into_bytes());

    if calculated_hash != hash_from_telegram {
        return None;
    }

    // Telegram initData нельзя принимать бесконечно долго.
    // Проверяем auth_date после успешной проверки подписи.
    //
    // Разрешаем максимум 10 минут с момента выдачи.
    // Небольшой запас в будущее (60 секунд) нужен на возможную
    // рассинхронизацию часов между Telegram и сервером.
    let auth_date: i64 = pairs
        .iter()
        .find(|(k, _)| k == "auth_date")
        .and_then(|(_, v)| v.parse::<i64>().ok())?;

    let now = unix_now();

    if auth_date <= 0 || auth_date > now + 60 || now.saturating_sub(auth_date) > 600 {
        return None;
    }

    let user_json = pairs.iter().find(|(k, _)| k == "user").map(|(_, v)| v)?;

    let user: serde_json::Value = serde_json::from_str(user_json).ok()?;

    user.get("id")?.as_i64()
}

fn telegram_profile_from_init_data(init_data: &str) -> (String, String, String) {
    let params: Vec<(String, String)> = url::form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();

    let user_json = params
        .iter()
        .find(|(key, _)| key == "user")
        .map(|(_, value)| value.as_str())
        .unwrap_or("");

    let user: serde_json::Value = match serde_json::from_str(user_json) {
        Ok(value) => value,
        Err(_) => {
            return (String::new(), String::new(), String::new());
        }
    };

    let username = user
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let first_name = user
        .get("first_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let last_name = user
        .get("last_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    (username, first_name, last_name)
}

fn telegram_owner_user_id(client_id: &str) -> Option<i64> {
    client_id
        .strip_prefix("tg:")
        .and_then(|value| value.parse::<i64>().ok())
}

fn input_text_is_valid(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let length = value.chars().count();

    if length < min_chars || length > max_chars {
        return false;
    }

    // Запрещаем управляющие символы, которые не нужны
    // обычному пользовательскому тексту.
    //
    // Перевод строки / CR / TAB разрешаем:
    // они нужны описаниям, сообщениям и статусам.
    !value
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
}

fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

async fn rate_limit_retry_after(
    state: &AppState,
    user_id: i64,
    action: &str,
    max_requests: usize,
    window_seconds: i64,
) -> Option<u64> {
    let now = unix_now();
    let cutoff = now.saturating_sub(window_seconds);
    let key = format!("{}:{}", action, user_id);

    let mut limits = state.rate_limits.lock().await;
    let events = limits.entry(key).or_default();

    // Sliding window:
    // забываем только запросы, вышедшие за текущее окно.
    while let Some(timestamp) = events.front().copied() {
        if timestamp <= cutoff {
            events.pop_front();
        } else {
            break;
        }
    }

    if events.len() >= max_requests {
        let oldest = events.front().copied().unwrap_or(now);

        let retry_after = oldest
            .saturating_add(window_seconds)
            .saturating_sub(now)
            .max(1);

        return Some(retry_after as u64);
    }

    events.push_back(now);

    None
}

fn request_is_cross_site(headers: &HeaderMap) -> bool {
    // Современные браузеры прямо сообщают происхождение запроса.
    // Если они говорят cross-site — такой state-changing запрос
    // с cookie-сессией нам не нужен.
    if let Some(value) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        if value.eq_ignore_ascii_case("cross-site") {
            return true;
        }
    }

    // Дополнительная проверка Origin.
    //
    // Заголовок может отсутствовать у curl, некоторых WebView и
    // старых клиентов — отсутствие Origin само по себе НЕ блокируем.
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let origin = origin.trim_end_matches('/');

        if origin != "https://resursmap.de"
            && origin != "https://www.resursmap.de"
            && origin != "http://127.0.0.1:3000"
        {
            return true;
        }
    }

    false
}

fn csrf_rejected_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "ok": false,
            "error": "cross_site_request_rejected"
        })),
    )
        .into_response()
}

fn create_user_session(state: &AppState, user_id: i64) -> String {
    type HmacSha256 = Hmac<Sha256>;

    // Пользовательская сессия на 30 дней.
    let expires = unix_now() + 2_592_000;
    let payload = format!("user:{}:{}", user_id, expires);

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(payload.as_bytes());

    let signature = hex::encode(mac.finalize().into_bytes());

    format!("{}:{}:{}", user_id, expires, signature)
}

fn verify_user_session(state: &AppState, headers: &HeaderMap) -> Option<i64> {
    type HmacSha256 = Hmac<Sha256>;

    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    let session = cookie_header
        .split(';')
        .map(|v| v.trim())
        .find_map(|v| v.strip_prefix("resursmap_user="))?;

    let mut parts = session.split(':');

    let user_id: i64 = parts.next()?.parse().ok()?;
    let expires: i64 = parts.next()?.parse().ok()?;
    let signature_hex = parts.next()?;

    if parts.next().is_some() || expires < unix_now() {
        return None;
    }

    let signature = hex::decode(signature_hex).ok()?;

    let payload = format!("user:{}:{}", user_id, expires);

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).ok()?;

    mac.update(payload.as_bytes());

    if mac.verify_slice(&signature).is_ok() {
        Some(user_id)
    } else {
        None
    }
}

fn create_admin_session(state: &AppState, user_id: i64) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let expires = unix_now() + 43_200;
    let payload = format!("{}:{}", user_id, expires);

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(payload.as_bytes());

    let signature = hex::encode(mac.finalize().into_bytes());

    format!("{}:{}", payload, signature)
}

fn verify_admin_session(state: &AppState, headers: &HeaderMap) -> bool {
    type HmacSha256 = Hmac<Sha256>;

    let cookie_header = match headers.get(header::COOKIE) {
        Some(v) => match v.to_str() {
            Ok(v) => v,
            Err(_) => return false,
        },
        None => return false,
    };

    let session = cookie_header
        .split(';')
        .map(|v| v.trim())
        .find_map(|v| v.strip_prefix("resursmap_admin="));

    let session = match session {
        Some(v) => v,
        None => return false,
    };

    let mut parts = session.split(':');

    let user_id: i64 = match parts.next().and_then(|v| v.parse().ok()) {
        Some(v) => v,
        None => return false,
    };

    let expires: i64 = match parts.next().and_then(|v| v.parse().ok()) {
        Some(v) => v,
        None => return false,
    };

    let signature_hex = match parts.next() {
        Some(v) => v,
        None => return false,
    };

    if parts.next().is_some() || user_id != state.admin_telegram_id || expires < unix_now() {
        return false;
    }

    let signature = match hex::decode(signature_hex) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let payload = format!("{}:{}", user_id, expires);

    let mut mac = match HmacSha256::new_from_slice(state.admin_key.as_bytes()) {
        Ok(v) => v,
        Err(_) => return false,
    };

    mac.update(payload.as_bytes());

    mac.verify_slice(&signature).is_ok()
}

fn is_admin_session(state: &AppState, headers: &HeaderMap) -> bool {
    verify_admin_session(state, headers)
}

// ------------------------------------------------------------
// HTML-страницы
// ------------------------------------------------------------
pub async fn home() -> Html<String> {
    Html("<h1>ResursMap</h1><p>Сервер работает.</p>".to_string())
}

pub async fn app_auth_page() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>ResursMap</title>
<script src="https://telegram.org/js/telegram-web-app.js"></script>
</head>

<body style="
    margin:0;
    padding:28px;
    background:#080a0d;
    color:#f5f5f5;
    font-family:system-ui;
">

<div style="max-width:520px;margin:auto;">
    <h2>RESURSMAP</h2>
    <p id="status">Подключаем Telegram…</p>
</div>

<script>
(async function () {
    const status = document.getElementById("status");

    try {
        const tg =
            window.Telegram && window.Telegram.WebApp
                ? window.Telegram.WebApp
                : null;

        if (!tg || !tg.initData) {
            status.textContent =
                "Telegram-сессия недоступна.";
            return;
        }

        tg.ready();

        const response = await fetch("/app/auth", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                init_data: tg.initData
            })
        });

        const data = await response.json();

        if (!data.ok) {
            status.textContent =
                "Не удалось подтвердить Telegram.";
            return;
        }

        status.textContent = "✓ Готово";

        window.location.replace("/app");
    } catch (_) {
        status.textContent =
            "Ошибка подключения Telegram.";
    }
})();
</script>

</body>
</html>"#
            .to_string(),
    )
}

pub async fn app_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }
    let init_data = payload
        .get("init_data")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let user_id = match verify_telegram_init_data(init_data, &state.bot_token) {
        Some(id) => id,

        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "ok": false,
                    "error": "invalid_telegram_data"
                })),
            )
                .into_response();
        }
    };

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "app_auth", 30, 600).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(json!({
                "ok": false,
                "error": "rate_limited",
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // TASK 7.16B PROFILE SYNC
    let (telegram_username, telegram_first_name, telegram_last_name) =
        telegram_profile_from_init_data(init_data);

    let client_id = format!("tg:{}", user_id);

    {
        let db = crate::db::pool::get_connection(&state.db_pool).ok();

        if let Some(db) = db {
            let _ = db.execute(
                "INSERT INTO profiles (
                client_id,
                username,
                first_name,
                last_name,
                updated_at
             )
             VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                strftime('%s','now')
             )
             ON CONFLICT(client_id)
             DO UPDATE SET
                username = excluded.username,
                first_name = excluded.first_name,
                last_name = excluded.last_name,
                updated_at = strftime('%s','now')",
                rusqlite::params![
                    &client_id,
                    &telegram_username,
                    &telegram_first_name,
                    &telegram_last_name,
                ],
            );
        }
    }

    let session = create_user_session(&state, user_id);

    let cookie = format!(
        "resursmap_user={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=2592000",
        session
    );

    let mut response = Json(json!({
        "ok": true
    }))
    .into_response();

    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }

    response
}

pub async fn app_root() -> Html<String> {
    Html(templates::render_continents())
}

pub async fn app_search(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Response {
    let q = params.get("q").map(|s| s.trim()).unwrap_or("");

    if q.chars().count() > 100 || q.chars().any(|c| c.is_control()) {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>400</h1><p>Поисковый запрос должен содержать не более 100 символов.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    let mut resources: Vec<(
        i64,
        String,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        usize,
        usize,
        usize,
    )> = Vec::new();

    let mut people: Vec<(String, String, String, String, i64, String, i64)> = Vec::new();

    if !q.is_empty() {
        let query_lower = q.to_lowercase();

        let mut location_matches: Vec<(usize, usize, usize)> = Vec::new();

        let world_data = templates::world();

        for (ci, (_, countries)) in world_data.iter().enumerate() {
            for (si, (country, cities)) in countries.iter().enumerate() {
                let country_match = country.to_lowercase().contains(&query_lower);

                for (zi, city) in cities.iter().enumerate() {
                    let city_match = city.to_lowercase().contains(&query_lower);

                    if city_match || country_match {
                        location_matches.push((ci, si, zi));
                    }
                }
            }
        }

        let db = match crate::db::pool::get_connection(&state.db_pool) {
            Ok(db) => db,
            Err(_) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "База данных временно недоступна",
                )
                    .into_response();
            }
        };

        // FTS5 query:
        // каждое слово ищем как prefix, чтобы запрос
        // "elect" находил "electrician".
        //
        // Спецсимволы FTS не передаём напрямую:
        // оставляем только буквы/цифры и собираем безопасный MATCH.
        let fts_terms: Vec<String> = q
            .split_whitespace()
            .filter_map(|term| {
                let clean: String = term.chars().filter(|c| c.is_alphanumeric()).collect();

                if clean.is_empty() {
                    None
                } else {
                    Some(format!("{}*", clean))
                }
            })
            .collect();

        let fts_query = fts_terms.join(" ");

        let mut sql = String::from(
            "SELECT
                r.id,
                r.title,
                r.category,
                r.description,
                r.address,
                r.rating,
                r.votes,
                r.is_verified,
                r.is_premium,
                r.continent_index,
                r.country_index,
                r.city_index
             FROM resources_fts f
             JOIN resources r
               ON r.id = f.rowid
             WHERE r.is_active = 1
               AND r.moderation_status = 'approved'
               AND resources_fts MATCH ?1",
        );

        let mut values: Vec<rusqlite::types::Value> = Vec::new();

        values.push(fts_query.clone().into());

        if !location_matches.is_empty() {
            sql.push_str(" AND (");

            for (index, _) in location_matches.iter().enumerate() {
                if index > 0 {
                    sql.push_str(" OR ");
                }

                let base = 2 + index * 3;

                sql.push_str(&format!(
                    "(
                        r.continent_index = ?{}
                        AND r.country_index = ?{}
                        AND r.city_index = ?{}
                    )",
                    base,
                    base + 1,
                    base + 2
                ));
            }

            sql.push(')');
        }

        sql.push_str(
            " ORDER BY
                r.is_premium DESC,
                r.is_verified DESC,
                r.rating DESC,
                r.votes DESC,
                r.id DESC
              LIMIT 100",
        );

        for (ci, si, zi) in location_matches {
            values.push((ci as i64).into());
            values.push((si as i64).into());
            values.push((zi as i64).into());
        }

        if !fts_terms.is_empty() {
            resources = db
                .prepare(&sql)
                .and_then(|mut stmt| {
                    stmt.query_map(rusqlite::params_from_iter(values.iter()), |row| {
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
                            row.get(9)?,
                            row.get(10)?,
                            row.get(11)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()
                })
                .unwrap_or_default();
        }

        if !fts_terms.is_empty() {
            people = db
                .prepare(
                    "SELECT
                        p.public_id,
                        p.username,
                        p.first_name,
                        p.last_name,
                        p.open_contact,
                        p.intent_text,
                        p.intent_until
                     FROM profiles_fts
                     JOIN profiles p
                       ON p.rowid = profiles_fts.rowid
                     WHERE profiles_fts MATCH ?1
                       AND p.client_id LIKE 'tg:%'
                       AND p.public_id <> ''
                     ORDER BY
                        CASE
                            WHEN p.intent_text <> ''
                             AND (
                                  p.intent_until = 0
                                  OR p.intent_until >= strftime('%s','now')
                             )
                            THEN 0
                            ELSE 1
                        END,
                        p.updated_at DESC
                     LIMIT 50",
                )
                .and_then(|mut stmt| {
                    stmt.query_map(rusqlite::params![&fts_query], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()
                })
                .unwrap_or_default();
        }

        drop(db);
    }

    Html(templates::render_search(q, resources, people)).into_response()
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

// ============================================================
// ДОБАВЛЕНИЕ РЕСУРСА
// ============================================================

#[derive(Debug, Deserialize)]
pub struct AddResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
    #[serde(default)]
    pub init_data: String,
}

// ============================================================
// PUBLIC USER PROFILE
// ============================================================

// ============================================================
// ПРОФИЛЬ РЕСУРСА
// ============================================================
// ============================================================
// МОИ РЕСУРСЫ
// ============================================================

// ============================================================
// CONTACT REQUESTS
// ============================================================

// ============================================================
// TASK 7.22G-C — MESSAGES LIST
// ============================================================

// ============================================================
// TASK 7.22F-E — INTERNAL CHAT
// ============================================================

#[derive(Debug, Deserialize)]
pub struct ChatMessageForm {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct EditResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
}

pub async fn health() -> &'static str {
    "ok"
}

// ------------------------------------------------------------
// API-обработчики (реальная логика)
// ------------------------------------------------------------
#[derive(Debug, serde::Deserialize)]
pub struct ReportResourcePayload {
    pub reason: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ContactRequestPayload {
    pub public_id: String,
    pub message: String,
}

// ============================================================
// TASK 7.3B — MODERATION APPROVE / REJECT
// ============================================================

#[derive(serde::Deserialize)]
pub struct RejectResourceForm {
    pub reason: String,
}
