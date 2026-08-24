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

pub async fn admin_login_page() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Admin Login · ResursMap</title>
<script src="https://telegram.org/js/telegram-web-app.js"></script>
</head>

<body style="
    font-family:system-ui;
    background:#0b0e12;
    color:#f5f5f5;
    padding:28px;
">

<div style="max-width:520px;margin:auto;">
    <h1>ResursMap Admin</h1>
    <p id="status">Проверяем Telegram…</p>
</div>

<script>
(async function () {
    const status = document.getElementById("status");

    try {
        const tg = window.Telegram && window.Telegram.WebApp
            ? window.Telegram.WebApp
            : null;

        if (!tg || !tg.initData) {
            status.textContent =
                "Откройте эту страницу через Telegram Mini App.";
            return;
        }

        tg.ready();

        const response = await fetch("/app/admin/login", {
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
            status.textContent = "Доступ запрещён.";
            return;
        }

        status.textContent = "✓ Вход выполнен";

        window.location.replace("/app/admin/resources");
    } catch (_) {
        status.textContent = "Ошибка авторизации.";
    }
})();
</script>

</body>
</html>"#
            .to_string(),
    )
}

pub async fn admin_login(
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
        Some(id) if id == state.admin_telegram_id => id,

        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "ok": false,
                    "error": "forbidden"
                })),
            )
                .into_response();
        }
    };

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "admin_auth", 10, 600).await
    {
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

    let session = create_admin_session(&state, user_id);

    let cookie = format!(
        "resursmap_admin={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=43200",
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

pub async fn app_me(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_me(
                false, 0, "", "", "", 0, 0, 0, 0, 0, 0, 0, 0, false, "", 0,
            ));
        }
    };

    let client_id = format!("tg:{}", user_id);

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let profile: Option<(String, String, String, i64, String, i64)> = db
        .query_row(
            "SELECT
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until
             FROM profiles
             WHERE client_id = ?1",
            rusqlite::params![&client_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .ok();

    let (username, first_name, last_name, open_contact, intent_text, intent_until) = profile
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                0,
                String::new(),
                0,
            )
        });

    let resources_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let approved_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1
               AND moderation_status = 'approved'",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pending_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1
               AND moderation_status = 'pending'",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let rejected_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1
               AND moderation_status = 'rejected'",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let favorites_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM favorites
             WHERE user_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let unread_notifications_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM user_notifications
             WHERE user_id = ?1
               AND is_read = 0",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pending_contact_requests_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM contact_requests
             WHERE receiver_user_id = ?1
               AND status = 'pending'",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let unread_messages_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM messages m
             JOIN conversations c
               ON c.id = m.conversation_id
             WHERE (c.user1_id = ?1 OR c.user2_id = ?1)
               AND m.sender_user_id <> ?1
               AND m.is_read = 0",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    drop(db);

    Html(templates::render_me(
        true,
        user_id,
        &username,
        &first_name,
        &last_name,
        resources_count,
        approved_count,
        pending_count,
        rejected_count,
        favorites_count,
        unread_notifications_count,
        pending_contact_requests_count,
        unread_messages_count,
        open_contact == 1,
        &intent_text,
        intent_until,
    ))
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
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resources: Vec<(i64, String, String, String, String, f64, i64, i64, i64)> = db
        .prepare(
            "SELECT id, title, description, contact, address, rating, votes, is_verified, is_premium
             FROM resources
             WHERE continent_index = ?1
               AND country_index = ?2
               AND city_index = ?3
               AND category = ?4
               AND is_active = 1
               AND moderation_status = 'approved'
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

    Html(templates::render_category(ci, si, zi, &k, resources))
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

pub async fn public_user_profile(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let public_id = public_id.trim();

    if public_id.is_empty()
        || public_id.len() > 64
        || !public_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Html(templates::render_public_user_not_found());
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let profile: Option<(String, String, String, String, i64, String, i64)> = db
        .query_row(
            "SELECT
                client_id,
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until
             FROM profiles
             WHERE public_id = ?1
             LIMIT 1",
            rusqlite::params![public_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .ok();

    let (client_id, username, first_name, last_name, open_contact, intent_text, intent_until) =
        match profile {
            Some(profile) => profile,

            None => {
                drop(db);

                return Html(templates::render_public_user_not_found());
            }
        };

    // Старые web:-профили публичными не делаем.
    if !client_id.starts_with("tg:") {
        drop(db);

        return Html(templates::render_public_user_not_found());
    }

    let resources: Vec<(i64, String, String, String, f64, i64, i64, i64)> = db
        .prepare(
            "SELECT
                id,
                title,
                category,
                description,
                rating,
                votes,
                is_verified,
                is_premium
             FROM resources
             WHERE client_id = ?1
               AND is_active = 1
               AND moderation_status = 'approved'
             ORDER BY
                is_premium DESC,
                is_verified DESC,
                rating DESC,
                votes DESC,
                id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![&client_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    // Определяем, кто сейчас смотрит публичный профиль.
    let viewer_user_id = verify_user_session(&state, &headers);

    // Если между текущим пользователем и владельцем профиля
    // уже существует conversation, передаём ID владельца
    // в шаблон, чтобы вместо повторного запроса показать чат.
    let chat_user_id: Option<i64> = viewer_user_id.and_then(|viewer_id| {
        let profile_user_id = client_id
            .strip_prefix("tg:")
            .and_then(|value| value.parse::<i64>().ok())?;

        if viewer_id <= 0 || profile_user_id <= 0 || viewer_id == profile_user_id {
            return None;
        }

        let (user1_id, user2_id) = if viewer_id < profile_user_id {
            (viewer_id, profile_user_id)
        } else {
            (profile_user_id, viewer_id)
        };

        let exists: Option<i64> = db
            .query_row(
                "SELECT id
                     FROM conversations
                     WHERE user1_id = ?1
                       AND user2_id = ?2
                     LIMIT 1",
                rusqlite::params![user1_id, user2_id,],
                |row| row.get(0),
            )
            .ok();

        exists.map(|_| profile_user_id)
    });

    drop(db);

    // Просроченный статус публично не показываем.
    let visible_intent = if intent_until > 0 && intent_until < unix_now() {
        String::new()
    } else {
        intent_text
    };

    Html(templates::render_public_user_profile(
        public_id,
        &username,
        &first_name,
        &last_name,
        open_contact == 1,
        &visible_intent,
        chat_user_id,
        resources,
    ))
}

// ============================================================
// ПРОФИЛЬ РЕСУРСА
// ============================================================
pub async fn resource_profile(State(state): State<AppState>, Path(id): Path<i64>) -> Html<String> {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resource = db
        .query_row(
            "SELECT
                r.title,
                r.description,
                r.contact,
                r.address,
                r.rating,
                r.votes,
                r.is_premium,
                r.is_verified,
                r.category,
                r.created_at,
                COALESCE(p.public_id, '')
         FROM resources r
         LEFT JOIN profiles p
           ON p.client_id = r.client_id
         WHERE r.id = ?1
           AND r.is_active = 1
           AND r.moderation_status = 'approved'",
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
                    row.get::<_, String>(10)?,
                ))
            },
        )
        .ok();

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
            owner_public_id,
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
            &owner_public_id,
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
    headers: HeaderMap,
    Form(form): Form<AddResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    let category = k.trim();
    let title = form.title.trim();
    let description = form.description.trim();
    let contact = form.contact.trim();
    let address = form.address.trim();
    let init_data = form.init_data.trim();

    if !input_text_is_valid(category, 1, 100) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Некорректная категория.</p>".to_string()),
        )
            .into_response();
    }

    if !input_text_is_valid(title, 1, 120) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Название должно содержать от 1 до 120 символов.</p>".to_string()),
        )
            .into_response();
    }

    if !input_text_is_valid(description, 1, 1000) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Описание должно содержать от 1 до 1000 символов.</p>".to_string()),
        )
            .into_response();
    }

    if !input_text_is_valid(contact, 0, 120) {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>400</h1><p>Контакт слишком длинный или содержит недопустимые символы.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    if !input_text_is_valid(address, 0, 250) {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>400</h1><p>Адрес слишком длинный или содержит недопустимые символы.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    // Telegram initData обычно намного меньше.
    // Ограничение защищает endpoint от бессмысленно огромного значения.
    if !input_text_is_valid(init_data, 0, 8192) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Некорректные данные Telegram.</p>".to_string()),
        )
            .into_response();
    }

    let category_url = urlencoding::encode(category);

    let owner_client_id = if let Some(user_id) = verify_user_session(&state, &headers) {
        format!("tg:{}", user_id)
    } else if !init_data.is_empty() {
        match verify_telegram_init_data(init_data, &state.bot_token) {
            Some(user_id) => format!("tg:{}", user_id),
            None => String::new(),
        }
    } else {
        String::new()
    };

    if owner_client_id.is_empty() {
        return Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Требуется Telegram · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
<section class="hero">
<div class="eyebrow">⚠ Авторизация</div>
<h1>Не удалось подтвердить пользователя</h1>
<p>Откройте ResursMap через Telegram и попробуйте добавить ресурс снова.</p>
<a class="card" href="/app" style="text-decoration:none;margin-top:20px;">
<div class="card-content">
<div class="card-title">Вернуться на карту</div>
</div>
<div class="card-arrow">›</div>
</a>
</section>
</main>
</body>
</html>"#,
            templates::base_style(),
        ))
        .into_response();
    }

    let owner_user_id = owner_client_id
        .strip_prefix("tg:")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);

    if owner_user_id <= 0 {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<h1>401</h1><p>Не удалось определить пользователя.</p>".to_string()),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, owner_user_id, "resource_add", 10, 3600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Html(
                "<h1>429</h1><p>Слишком много добавлений ресурсов. Попробуйте позже.</p>"
                    .to_string(),
            ),
        )
            .into_response();
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

    let result = db.execute(
        "INSERT INTO resources
        (client_id, continent_index, country_index, city_index,
         category, title, description, contact, address,
         rating, votes, is_premium, is_verified, is_active,
         moderation_status, rejection_reason,
         created_at, updated_at)
        VALUES
        (?9, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
         0, 0, 0, 0, 1,
         'pending', '',
         strftime('%s','now'), strftime('%s','now'))",
        rusqlite::params![
            ci,
            si,
            zi,
            category,
            title,
            description,
            contact,
            address,
            owner_client_id,
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
            category_url,
            templates::icon("map"),
        ))
        .into_response(),
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
            category_url,
        ))
        .into_response(),
    }
}

// ============================================================
// МОИ РЕСУРСЫ
// ============================================================

pub async fn notifications_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_notifications(vec![], false));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let notifications: Vec<(i64, Option<i64>, String, String, String, i64, i64)> = db
        .prepare(
            "SELECT
                id,
                resource_id,
                kind,
                title,
                message,
                is_read,
                created_at
             FROM user_notifications
             WHERE user_id = ?1
             ORDER BY
                is_read ASC,
                created_at DESC,
                id DESC
             LIMIT 100",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![user_id], |row| {
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

    let _ = db.execute(
        "UPDATE user_notifications
         SET is_read = 1
         WHERE user_id = ?1
           AND is_read = 0",
        rusqlite::params![user_id],
    );

    drop(db);

    Html(templates::render_notifications(notifications, true))
}

// ============================================================
// CONTACT REQUESTS
// ============================================================

pub async fn contact_requests_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_contact_requests(vec![], false));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let requests: Vec<(i64, i64, String, String, String, String, String, i64, i64)> = db
        .prepare(
            "SELECT
                cr.id,
                cr.sender_user_id,
                cr.message,
                cr.status,
                COALESCE(p.public_id, ''),
                COALESCE(p.username, ''),
                COALESCE(p.first_name, ''),
                cr.created_at,
                CASE
                    WHEN cr.status = 'pending' THEN 0
                    WHEN cr.status = 'accepted' THEN 1
                    ELSE 2
                END
             FROM contact_requests cr
             LEFT JOIN profiles p
               ON p.client_id = ('tg:' || cr.sender_user_id)
             WHERE cr.receiver_user_id = ?1
             ORDER BY
                CASE cr.status
                    WHEN 'pending' THEN 0
                    WHEN 'accepted' THEN 1
                    ELSE 2
                END,
                cr.updated_at DESC,
                cr.id DESC
             LIMIT 100",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![user_id], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    Html(templates::render_contact_requests(requests, true))
}

pub async fn accept_contact_request(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (StatusCode::UNAUTHORIZED, "Требуется вход через Telegram").into_response();
        }
    };

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "contact_decision", 30, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много действий с запросами. Попробуйте позже.",
        )
            .into_response();
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

    let request: Option<i64> = db
        .query_row(
            "SELECT sender_user_id
             FROM contact_requests
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id],
            |row| row.get(0),
        )
        .ok();

    let sender_user_id = match request {
        Some(id) => id,

        None => {
            drop(db);

            return (StatusCode::NOT_FOUND, "Запрос не найден").into_response();
        }
    };

    let (user1_id, user2_id) = if sender_user_id < user_id {
        (sender_user_id, user_id)
    } else {
        (user_id, sender_user_id)
    };

    let transaction_result = (|| -> rusqlite::Result<usize> {
        let tx = db.unchecked_transaction()?;

        let changed = tx.execute(
            "UPDATE contact_requests
             SET status = 'accepted',
                 updated_at = strftime('%s','now')
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id],
        )?;

        if changed == 1 {
            tx.execute(
                "INSERT INTO conversations (
                    user1_id,
                    user2_id,
                    created_at,
                    updated_at
                 )
                 VALUES (
                    ?1,
                    ?2,
                    strftime('%s','now'),
                    strftime('%s','now')
                 )
                 ON CONFLICT(user1_id, user2_id)
                 DO UPDATE SET
                    updated_at = strftime('%s','now')",
                rusqlite::params![user1_id, user2_id],
            )?;
        }

        tx.commit()?;
        Ok(changed)
    })();

    let changed = match transaction_result {
        Ok(changed) => changed,
        Err(err) => {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка принятия запроса: {}", err),
            )
                .into_response();
        }
    };

    if changed == 1 {
        let _ = db.execute(
            "INSERT INTO user_notifications (
                user_id,
                resource_id,
                kind,
                title,
                message,
                is_read,
                created_at
             )
             VALUES (
                ?1,
                NULL,
                'contact_accepted',
                'Запрос принят',
                'Ваш запрос на связь принят. Теперь можно начать общение в ResursMap.',
                0,
                strftime('%s','now')
             )",
            rusqlite::params![sender_user_id],
        );
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/contact-requests")],
    )
        .into_response()
}

pub async fn reject_contact_request(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (StatusCode::UNAUTHORIZED, "Требуется вход через Telegram").into_response();
        }
    };

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "contact_decision", 30, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много действий с запросами. Попробуйте позже.",
        )
            .into_response();
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

    let request: Option<i64> = db
        .query_row(
            "SELECT sender_user_id
             FROM contact_requests
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id],
            |row| row.get(0),
        )
        .ok();

    let sender_user_id = match request {
        Some(id) => id,

        None => {
            drop(db);

            return (StatusCode::NOT_FOUND, "Запрос не найден").into_response();
        }
    };

    let changed = db
        .execute(
            "UPDATE contact_requests
             SET status = 'rejected',
                 updated_at = strftime('%s','now')
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id,],
        )
        .unwrap_or(0);

    if changed == 1 {
        let _ = db.execute(
            "INSERT INTO user_notifications (
                user_id,
                resource_id,
                kind,
                title,
                message,
                is_read,
                created_at
             )
             VALUES (
                ?1,
                NULL,
                'contact_rejected',
                'Запрос отклонён',
                'Пользователь отклонил запрос на связь.',
                0,
                strftime('%s','now')
             )",
            rusqlite::params![sender_user_id,],
        );
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/contact-requests")],
    )
        .into_response()
}

// ============================================================
// TASK 7.22G-C — MESSAGES LIST
// ============================================================

pub async fn messages_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_messages(false, vec![]));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let conversations: Vec<(i64, i64, String, String, String, String, i64, i64)> = db
        .prepare(
            "SELECT
                c.id,

                CASE
                    WHEN c.user1_id = ?1
                    THEN c.user2_id
                    ELSE c.user1_id
                END AS other_user_id,

                COALESCE(p.username, ''),
                COALESCE(p.first_name, ''),
                COALESCE(p.last_name, ''),

                COALESCE((
                    SELECT m.message
                    FROM messages m
                    WHERE m.conversation_id = c.id
                    ORDER BY
                        m.created_at DESC,
                        m.id DESC
                    LIMIT 1
                ), ''),

                (
                    SELECT COUNT(*)
                    FROM messages m
                    WHERE m.conversation_id = c.id
                      AND m.sender_user_id <> ?1
                      AND m.is_read = 0
                ) AS unread_count,

                c.updated_at

             FROM conversations c

             LEFT JOIN profiles p
               ON p.client_id = (
                    'tg:' ||
                    CASE
                        WHEN c.user1_id = ?1
                        THEN c.user2_id
                        ELSE c.user1_id
                    END
               )

             WHERE c.user1_id = ?1
                OR c.user2_id = ?1

             ORDER BY
                c.updated_at DESC,
                c.id DESC

             LIMIT 200",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![user_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let _ = db.execute(
        "UPDATE user_notifications
         SET is_read = 1
         WHERE user_id = ?1
           AND kind = 'chat_message'
           AND is_read = 0",
        rusqlite::params![user_id],
    );

    drop(db);

    Html(templates::render_messages(true, conversations))
}

// ============================================================
// TASK 7.22F-E — INTERNAL CHAT
// ============================================================

#[derive(Debug, Deserialize)]
pub struct ChatMessageForm {
    pub message: String,
}

pub async fn chat_page(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return Html(templates::render_chat(false, 0, "", "", "", vec![]));
        }
    };

    if other_user_id <= 0 || other_user_id == user_id {
        return Html(templates::render_chat(
            true,
            other_user_id,
            "",
            "",
            "",
            vec![],
        ));
    }

    let (user1_id, user2_id) = if user_id < other_user_id {
        (user_id, other_user_id)
    } else {
        (other_user_id, user_id)
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let conversation_id: Option<i64> = db
        .query_row(
            "SELECT id
             FROM conversations
             WHERE user1_id = ?1
               AND user2_id = ?2
             LIMIT 1",
            rusqlite::params![user1_id, user2_id,],
            |row| row.get(0),
        )
        .ok();

    let conversation_id = match conversation_id {
        Some(id) => id,

        None => {
            drop(db);

            return Html(templates::render_chat(
                true,
                other_user_id,
                "",
                "",
                "",
                vec![],
            ));
        }
    };

    let other_profile: Option<(String, String, String)> = db
        .query_row(
            "SELECT
                COALESCE(username, ''),
                COALESCE(first_name, ''),
                COALESCE(last_name, '')
             FROM profiles
             WHERE client_id = ?1
             LIMIT 1",
            rusqlite::params![format!("tg:{}", other_user_id)],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let (other_username, other_first_name, other_last_name) =
        other_profile.unwrap_or_else(|| (String::new(), String::new(), String::new()));

    let messages: Vec<(i64, i64, String, i64, i64)> = db
        .prepare(
            "SELECT
                id,
                sender_user_id,
                message,
                is_read,
                created_at
             FROM messages
             WHERE conversation_id = ?1
             ORDER BY
                created_at ASC,
                id ASC
             LIMIT 500",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![conversation_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let _ = db.execute(
        "UPDATE messages
         SET is_read = 1
         WHERE conversation_id = ?1
           AND sender_user_id = ?2
           AND is_read = 0",
        rusqlite::params![conversation_id, other_user_id,],
    );

    drop(db);

    Html(templates::render_chat(
        true,
        other_user_id,
        &other_username,
        &other_first_name,
        &other_last_name,
        messages,
    ))
}

pub async fn send_chat_message(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<ChatMessageForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (StatusCode::UNAUTHORIZED, "Требуется вход через Telegram").into_response();
        }
    };

    if other_user_id <= 0 || other_user_id == user_id {
        return (StatusCode::BAD_REQUEST, "Некорректный пользователь").into_response();
    }

    let message = form.message.trim();

    if message.is_empty() {
        return (
            StatusCode::SEE_OTHER,
            [(
                header::LOCATION,
                format!("/app/chat/{}#chat-end", other_user_id),
            )],
        )
            .into_response();
    }

    if !input_text_is_valid(message, 1, 2000) {
        return (
            StatusCode::BAD_REQUEST,
            "Сообщение слишком длинное или содержит недопустимые символы",
        )
            .into_response();
    }

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "chat_send", 30, 60).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много сообщений. Попробуйте немного позже.",
        )
            .into_response();
    }

    let (user1_id, user2_id) = if user_id < other_user_id {
        (user_id, other_user_id)
    } else {
        (other_user_id, user_id)
    };

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

    let conversation_id: Option<i64> = db
        .query_row(
            "SELECT id
             FROM conversations
             WHERE user1_id = ?1
               AND user2_id = ?2
             LIMIT 1",
            rusqlite::params![user1_id, user2_id,],
            |row| row.get(0),
        )
        .ok();

    let conversation_id = match conversation_id {
        Some(id) => id,

        None => {
            drop(db);

            return (StatusCode::FORBIDDEN, "Чат между пользователями не открыт").into_response();
        }
    };

    let transaction_result = (|| -> rusqlite::Result<usize> {
        let tx = db.unchecked_transaction()?;

        let inserted = tx.execute(
            "INSERT INTO messages (
                conversation_id,
                sender_user_id,
                message,
                is_read,
                created_at
             )
             VALUES (
                ?1,
                ?2,
                ?3,
                0,
                strftime('%s','now')
             )",
            rusqlite::params![conversation_id, user_id, message],
        )?;

        if inserted == 1 {
            tx.execute(
                "UPDATE conversations
                 SET updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![conversation_id],
            )?;
        }

        tx.commit()?;
        Ok(inserted)
    })();

    let inserted = match transaction_result {
        Ok(inserted) => inserted,
        Err(err) => {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка сохранения сообщения: {}", err),
            )
                .into_response();
        }
    };

    if inserted == 1 {
        let existing_chat_notification = db
            .execute(
                "UPDATE user_notifications
                 SET created_at = strftime('%s','now')
                 WHERE user_id = ?1
                   AND kind = 'chat_message'
                   AND is_read = 0",
                rusqlite::params![other_user_id],
            )
            .unwrap_or(0);

        if existing_chat_notification == 0 {
            let _ = db.execute(
                "INSERT INTO user_notifications (
                    user_id,
                    resource_id,
                    kind,
                    title,
                    message,
                    is_read,
                    created_at
                 )
                 VALUES (
                    ?1,
                    NULL,
                    'chat_message',
                    'Новое сообщение',
                    'У вас новое сообщение в ResursMap.',
                    0,
                    strftime('%s','now')
                 )",
                rusqlite::params![other_user_id],
            );
        }
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(
            header::LOCATION,
            format!("/app/chat/{}#chat-end", other_user_id),
        )],
    )
        .into_response()
}

pub async fn favorites_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return Html(templates::render_favorites(vec![], false));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resources: Vec<(i64, String, String, String, String, f64, i64, i64, i64)> = db
        .prepare(
            "SELECT
                r.id,
                r.title,
                r.category,
                r.description,
                r.address,
                r.rating,
                r.votes,
                r.is_verified,
                r.is_premium
             FROM favorites f
             JOIN resources r
               ON r.id = f.resource_id
             WHERE f.user_id = ?1
               AND r.is_active = 1
               AND r.moderation_status = 'approved'
             ORDER BY
                r.is_premium DESC,
                r.is_verified DESC,
                f.created_at DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![user_id], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    Html(templates::render_favorites(resources, true))
}

pub async fn my_resources(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let client_id = match verify_user_session(&state, &headers) {
        Some(user_id) => format!("tg:{}", user_id),
        None => String::new(),
    };

    if client_id.is_empty() {
        return Html(templates::render_my_resources("", vec![]));
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resources: Vec<(
        i64,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        String,
        String,
        i64,
    )> = db
        .prepare(
            "SELECT
                id,
                title,
                category,
                description,
                rating,
                votes,
                is_verified,
                is_premium,
                moderation_status,
                rejection_reason,
                is_active
             FROM resources
             WHERE client_id = ?1
             ORDER BY is_active DESC,
                      is_premium DESC,
                      updated_at DESC,
                      id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![&client_id], |row| {
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
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    Html(templates::render_my_resources(&client_id, resources))
}

#[derive(Debug, Deserialize)]
pub struct EditResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
}

pub async fn edit_resource_page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Html<String> {
    let client_id = match verify_user_session(&state, &headers) {
        Some(user_id) => format!("tg:{}", user_id),
        None => String::new(),
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resource = db
        .query_row(
            "SELECT title, description, contact, address, category, client_id
         FROM resources
         WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .ok();

    drop(db);

    match resource {
        Some((title, description, contact, address, category, owner))
            if !client_id.is_empty() && owner == client_id =>
        {
            Html(templates::render_edit_resource(
                id,
                &title,
                &description,
                &contact,
                &address,
                &category,
            ))
        }

        _ => Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Нет доступа · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
<section class="hero">
<div class="eyebrow">⚠ Доступ</div>
<h1>Редактирование недоступно</h1>
<p>Этот ресурс не принадлежит текущему пользователю.</p>
<a class="card" href="/app/me" style="text-decoration:none;margin-top:20px;">
<div class="card-content">
<div class="card-title">Вернуться в профиль</div>
</div>
<div class="card-arrow">›</div>
</a>
</section>
</main>
</body>
</html>"#,
            templates::base_style(),
        )),
    }
}

pub async fn edit_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<EditResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    let title = form.title.trim();
    let description = form.description.trim();
    let contact = form.contact.trim();
    let address = form.address.trim();

    if !input_text_is_valid(title, 1, 120)
        || !input_text_is_valid(description, 1, 1000)
        || !input_text_is_valid(contact, 0, 120)
        || !input_text_is_valid(address, 0, 250)
    {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Проверьте длину и содержимое полей ресурса.</p>".to_string()),
        )
            .into_response();
    }

    let owner_client_id = match verify_user_session(&state, &headers) {
        Some(user_id) => format!("tg:{}", user_id),
        None => String::new(),
    };

    if owner_client_id.is_empty() {
        return Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Нет доступа · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
<section class="hero">
<div class="eyebrow">⚠ Доступ</div>
<h1>Не удалось подтвердить владельца</h1>
<p>Откройте ResursMap через Telegram и попробуйте снова.</p>
</section>
</main>
</body>
</html>"#,
            templates::base_style(),
        ))
        .into_response();
    }

    let owner_user_id = owner_client_id
        .strip_prefix("tg:")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);

    if owner_user_id <= 0 {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<h1>401</h1><p>Не удалось определить пользователя.</p>".to_string()),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, owner_user_id, "resource_edit", 30, 3600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Html(
                "<h1>429</h1><p>Слишком много изменений ресурсов. Попробуйте позже.</p>"
                    .to_string(),
            ),
        )
            .into_response();
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

    let changed = db
        .execute(
            "UPDATE resources
         SET title = ?1,
             description = ?2,
             contact = ?3,
             address = ?4,
             is_verified = 0,
             is_active = 1,
             moderation_status = 'pending',
             rejection_reason = '',
             updated_at = strftime('%s','now')
         WHERE id = ?5
           AND client_id = ?6",
            rusqlite::params![title, description, contact, address, id, &owner_client_id,],
        )
        .unwrap_or(0);

    drop(db);

    if changed == 1 {
        Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Изменения сохранены · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
<section class="hero">
<div class="eyebrow">✓ Готово</div>
<h1>Изменения сохранены</h1>
<p>После изменения ресурс снова ожидает проверки.</p>
<a class="card" href="/app/my-resources" style="text-decoration:none;margin-top:20px;">
<div class="card-content">
<div class="card-title">Вернуться в мои ресурсы</div>
</div>
<div class="card-arrow">›</div>
</a>
</section>
</main>
</body>
</html>"#,
            templates::base_style(),
        ))
        .into_response()
    } else {
        Html(format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Нет доступа · ResursMap</title>
<style>{}</style>
</head>
<body>
<main class="page">
<section class="hero">
<div class="eyebrow">⚠ Ошибка</div>
<h1>Не удалось сохранить</h1>
<p>Проверьте владельца ресурса.</p>
</section>
</main>
</body>
</html>"#,
            templates::base_style(),
        ))
        .into_response()
    }
}

pub async fn admin_reports(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    if !is_admin_session(&state, &headers) {
        return Html(
            r#"<h1>403</h1>
<p>Доступ запрещён.</p>
<a href="/app/admin/login">Войти через Telegram</a>"#
                .to_string(),
        );
    }

    let key_query = String::new();

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let rows: Vec<(
        i64,
        i64,
        i64,
        String,
        String,
        i64,
        String,
        String,
        String,
        i64,
    )> = db
        .prepare(
            "SELECT
                rr.id,
                rr.reporter_user_id,
                rr.resource_id,
                rr.reason,
                rr.status,
                rr.created_at,
                COALESCE(r.title, ''),
                COALESCE(r.category, ''),
                COALESCE(r.moderation_status, ''),
                COALESCE(r.is_active, 0)
             FROM resource_reports rr
             LEFT JOIN resources r
               ON r.id = rr.resource_id
             ORDER BY
                CASE rr.status
                    WHEN 'pending' THEN 0
                    ELSE 1
                END,
                rr.created_at DESC,
                rr.id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map([], |row| {
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
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    let pending_count = rows.iter().filter(|r| r.4 == "pending").count();

    let closed_count = rows.iter().filter(|r| r.4 == "closed").count();

    let mut cards = String::new();

    for (
        report_id,
        reporter_user_id,
        resource_id,
        reason,
        report_status,
        created_at,
        title,
        category,
        moderation_status,
        is_active,
    ) in rows
    {
        let safe_reason = templates::escape_html(&reason);
        let safe_title = templates::escape_html(&title);
        let safe_category = templates::escape_html(&category);
        let safe_moderation_status = templates::escape_html(&moderation_status);

        let report_badge = if report_status == "pending" {
            r#"<span style="color:#d97706;font-weight:800;">● Ожидает</span>"#
        } else {
            r#"<span style="color:#16a34a;font-weight:800;">✓ Закрыта</span>"#
        };

        let resource_status = if is_active == 1 {
            format!(
                r#"<span style="color:#16a34a;">● Активен · {}</span>"#,
                safe_moderation_status
            )
        } else {
            format!(
                r#"<span style="color:#dc2626;">● Скрыт · {}</span>"#,
                safe_moderation_status
            )
        };

        let close_button = if report_status == "pending" {
            format!(
                r#"
<form method="post"
      action="/app/admin/report/{report_id}/close{key_query}">
    <button type="submit"
            style="
                width:100%;
                min-height:44px;
                border-radius:12px;
                border:1px solid rgba(22,163,74,.35);
                background:rgba(22,163,74,.10);
                color:inherit;
                font-weight:800;
                cursor:pointer;
            ">
        ✓ Закрыть жалобу
    </button>
</form>
"#,
                report_id = report_id,
                key_query = key_query,
            )
        } else {
            String::new()
        };

        cards.push_str(&format!(
            r#"
<article style="
    border:1px solid rgba(214,183,122,.22);
    border-radius:20px;
    padding:18px;
    margin-bottom:16px;
    background:rgba(255,255,255,.035);
">

    <div style="
        display:flex;
        justify-content:space-between;
        gap:12px;
        flex-wrap:wrap;
        align-items:flex-start;
    ">
        <div>
            <div style="
                font-size:11px;
                color:#8f96a3;
                margin-bottom:6px;
            ">
                Жалоба #{report_id} · ресурс #{resource_id}
            </div>

            <h2 style="margin:0 0 6px;font-size:20px;">
                {title}
            </h2>

            <div style="
                color:#9ca3af;
                font-size:13px;
            ">
                {category}
            </div>
        </div>

        <div style="
            display:flex;
            gap:10px;
            flex-wrap:wrap;
            font-size:12px;
        ">
            {report_badge}
            {resource_status}
        </div>
    </div>

    <div style="
        margin-top:16px;
        padding:14px;
        border-radius:14px;
        background:rgba(217,119,6,.07);
        border:1px solid rgba(217,119,6,.20);
        line-height:1.5;
    ">
        <strong>Причина:</strong><br>
        {reason}
    </div>

    <div style="
        margin-top:10px;
        font-size:11px;
        color:var(--muted);
    ">
        Reporter Telegram ID: {reporter_user_id}
        · created_at: {created_at}
    </div>

    <div style="
        display:grid;
        grid-template-columns:repeat(auto-fit,minmax(145px,1fr));
        gap:9px;
        margin-top:18px;
    ">

        {close_button}

        <form method="post"
              action="/app/admin/report/{report_id}/hide-resource{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.30);
                        background:rgba(220,38,38,.07);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                Скрыть ресурс
            </button>
        </form>

        <form method="post"
              action="/app/admin/report/{report_id}/reject-resource{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.40);
                        background:rgba(220,38,38,.12);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                ✕ Отклонить ресурс
            </button>
        </form>

        <a href="/app/resource/{resource_id}"
           target="_blank"
           style="
               min-height:42px;
               border-radius:12px;
               border:1px solid rgba(255,255,255,.12);
               display:flex;
               align-items:center;
               justify-content:center;
               text-decoration:none;
               color:inherit;
               font-weight:800;
           ">
            Открыть ресурс
        </a>

    </div>

</article>
"#,
            report_id = report_id,
            resource_id = resource_id,
            title = safe_title,
            category = safe_category,
            report_badge = report_badge,
            resource_status = resource_status,
            reason = safe_reason,
            reporter_user_id = reporter_user_id,
            created_at = created_at,
            close_button = close_button,
            key_query = key_query,
        ));
    }

    if cards.is_empty() {
        cards = r#"
<div class="card" style="display:block;">
    <div class="card-content">
        <div class="card-title">Жалоб пока нет</div>
        <div class="card-meta">Очередь модерации пуста.</div>
    </div>
</div>
"#
        .to_string();
    }

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Жалобы · ResursMap</title>
<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">REPORT CENTER</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {shield}
        Администрирование
    </div>

    <h1>Жалобы</h1>

    <p>
        Проверка жалоб пользователей на опубликованные ресурсы.
    </p>

</section>

<div style="
    display:flex;
    gap:8px;
    flex-wrap:wrap;
    margin-bottom:20px;
">

    <a href="/app/admin/resources{key_query}"
       style="
           padding:9px 12px;
           border-radius:999px;
           text-decoration:none;
           border:1px solid rgba(255,255,255,.14);
           color:inherit;
           font-weight:700;
       ">
        Ресурсы
    </a>

    <a href="/app/admin/reports{key_query}"
       style="
           padding:9px 12px;
           border-radius:999px;
           text-decoration:none;
           border:1px solid rgba(217,119,6,.38);
           background:rgba(217,119,6,.08);
           color:inherit;
           font-weight:800;
       ">
        Жалобы
    </a>

</div>

<div style="
    display:grid;
    grid-template-columns:repeat(2,minmax(0,1fr));
    gap:10px;
    margin-bottom:24px;
">

    <div class="card">
        <div class="card-content">
            <div class="card-title">{pending_count}</div>
            <div class="card-meta">Ожидают</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{closed_count}</div>
            <div class="card-meta">Закрыты</div>
        </div>
    </div>

</div>

<section>
    {cards}
</section>

</main>

</body>
</html>"#,
        style = templates::base_style(),
        logo = templates::icon("map"),
        shield = templates::icon("user"),
        pending_count = pending_count,
        closed_count = closed_count,
        cards = cards,
        key_query = key_query,
    ))
}

pub async fn admin_close_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_admin_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
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

    let _ = db.execute(
        "UPDATE resource_reports
         SET status = 'closed',
             updated_at = strftime('%s','now')
         WHERE id = ?1",
        rusqlite::params![id],
    );

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/admin/reports")],
    )
        .into_response()
}

pub async fn admin_hide_reported_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_admin_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
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

    let resource_id: Option<i64> = db
        .query_row(
            "SELECT resource_id
             FROM resource_reports
             WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    if let Some(resource_id) = resource_id {
        let transaction_result = (|| -> rusqlite::Result<()> {
            let tx = db.unchecked_transaction()?;

            tx.execute(
                "UPDATE resources
                 SET is_active = 0,
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![resource_id],
            )?;

            tx.execute(
                "UPDATE resource_reports
                 SET status = 'closed',
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![id],
            )?;

            tx.commit()
        })();

        if let Err(err) = transaction_result {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка обработки жалобы: {}", err),
            )
                .into_response();
        }
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/admin/reports")],
    )
        .into_response()
}

pub async fn admin_reject_reported_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_admin_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
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

    let report: Option<(i64, String)> = db
        .query_row(
            "SELECT resource_id, reason
             FROM resource_reports
             WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    if let Some((resource_id, reason)) = report {
        let rejection_reason = format!("Жалоба пользователя: {}", reason);

        let transaction_result = (|| -> rusqlite::Result<()> {
            let tx = db.unchecked_transaction()?;

            tx.execute(
                "UPDATE resources
                 SET moderation_status = 'rejected',
                     rejection_reason = ?2,
                     is_verified = 0,
                     is_active = 0,
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![resource_id, rejection_reason],
            )?;

            tx.execute(
                "UPDATE resource_reports
                 SET status = 'closed',
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![id],
            )?;

            tx.commit()
        })();

        if let Err(err) = transaction_result {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка отклонения ресурса по жалобе: {}", err),
            )
                .into_response();
        }
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/admin/reports")],
    )
        .into_response()
}

pub async fn admin_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<BTreeMap<String, String>>,
) -> Html<String> {
    let filter = params.get("filter").map(|s| s.as_str()).unwrap_or("all");

    let q = params.get("q").map(|s| s.trim()).unwrap_or("");

    if q.chars().count() > 100 || q.chars().any(|c| c.is_control()) {
        return Html(
            "<h1>400</h1><p>Поисковый запрос должен содержать не более 100 символов.</p>"
                .to_string(),
        );
    }

    if !is_admin_session(&state, &headers) {
        return Html(
            r#"<h1>403</h1>
<p>Доступ запрещён.</p>
<a href="/app/admin/login">Войти через Telegram</a>"#
                .to_string(),
        );
    }

    let key_query = String::new();
    let key_join = "?";

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let filter_clause = match filter {
        "pending" => "moderation_status = 'pending'",
        "verified" => "moderation_status = 'approved'",
        "rejected" => "moderation_status = 'rejected'",
        "premium" => "is_premium = 1",
        "hidden" => "is_active = 0",
        _ => "1 = 1",
    };

    let search_clause = if q.is_empty() {
        "1 = 1"
    } else {
        "(LOWER(title) LIKE LOWER(?1)
          OR LOWER(category) LIKE LOWER(?1)
          OR LOWER(description) LIKE LOWER(?1))"
    };

    let sql = format!(
        "SELECT
            id,
            title,
            category,
            description,
            rating,
            votes,
            is_verified,
            is_premium,
            is_active,
            moderation_status,
            rejection_reason
         FROM resources
         WHERE {}
           AND {}
         ORDER BY
            is_verified ASC,
            is_active DESC,
            is_premium DESC,
            id DESC",
        filter_clause, search_clause
    );

    let rows: Vec<(
        i64,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        i64,
        String,
        String,
    )> = if q.is_empty() {
        db.prepare(&sql)
            .and_then(|mut stmt| {
                stmt.query_map([], |row| {
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
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default()
    } else {
        let pattern = format!("%{}%", q);

        db.prepare(&sql)
            .and_then(|mut stmt| {
                stmt.query_map(rusqlite::params![pattern], |row| {
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
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default()
    };

    let pending_reports_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resource_reports
             WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    drop(db);

    let result_count = rows.len();

    let pending = rows.iter().filter(|r| r.9 == "pending").count();

    let verified_count = rows.iter().filter(|r| r.9 == "approved").count();

    let rejected_count = rows.iter().filter(|r| r.9 == "rejected").count();

    let premium_count = rows.iter().filter(|r| r.7 == 1).count();

    let mut cards = String::new();

    for (
        id,
        title,
        category,
        description,
        rating,
        votes,
        _verified,
        premium,
        active,
        moderation_status,
        rejection_reason,
    ) in rows
    {
        let safe_title = templates::escape_html(&title);
        let safe_category = templates::escape_html(&category);
        let safe_description = templates::escape_html(&description);
        let safe_rejection_reason = templates::escape_html(&rejection_reason);

        let moderation_badge = match moderation_status.as_str() {
            "approved" => r#"<span style="color:#16a34a;font-weight:800;">✓ Одобрен</span>"#,
            "rejected" => r#"<span style="color:#dc2626;font-weight:800;">✕ Отклонён</span>"#,
            _ => r#"<span style="color:#d97706;font-weight:800;">● Ожидает проверки</span>"#,
        };

        let rejection_html =
            if moderation_status == "rejected" && !rejection_reason.trim().is_empty() {
                format!(
                    r#"<div style="
                        margin-top:12px;
                        padding:11px 13px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.22);
                        background:rgba(220,38,38,.07);
                        color:#dc2626;
                        font-size:13px;
                        line-height:1.45;
                    "><strong>Причина отказа:</strong> {}</div>"#,
                    safe_rejection_reason
                )
            } else {
                String::new()
            };

        let premium_badge = if premium == 1 {
            r#"<span style="color:#b88932;font-weight:800;">★ PREMIUM</span>"#
        } else {
            ""
        };

        let active_badge = if active == 1 {
            r#"<span style="color:#16a34a;">● Активен</span>"#
        } else {
            r#"<span style="color:#dc2626;">● Скрыт</span>"#
        };

        let premium_label = if premium == 1 {
            "Premium OFF"
        } else {
            "★ Premium ON"
        };

        let active_label = if active == 1 {
            "Скрыть"
        } else {
            "Вернуть"
        };

        cards.push_str(&format!(
            r#"
<article style="
    border:1px solid rgba(214,183,122,.22);
    border-radius:20px;
    padding:18px;
    margin:0 0 16px;
    background:rgba(255,255,255,.035);
    box-shadow:0 10px 30px rgba(0,0,0,.12);
">

    <div style="
        display:flex;
        justify-content:space-between;
        gap:14px;
        align-items:flex-start;
        flex-wrap:wrap;
    ">
        <div>
            <div style="font-size:11px;color:#8f96a3;margin-bottom:6px;">
                #{id} · {category}
            </div>

            <h2 style="margin:0 0 8px;font-size:20px;">
                {title}
            </h2>

            <div style="color:#9ca3af;line-height:1.5;max-width:620px;">
                {description}
            </div>
        </div>

        <div style="font-weight:800;white-space:nowrap;">
            ⭐ {rating:.1} · {votes}
        </div>
    </div>

    <div style="
        display:flex;
        gap:12px;
        flex-wrap:wrap;
        margin-top:14px;
        font-size:12px;
    ">
        {moderation_badge}
        {premium_badge}
        {active_badge}
    </div>

    {rejection_html}

    <div style="
        display:grid;
        grid-template-columns:repeat(auto-fit,minmax(145px,1fr));
        gap:9px;
        margin-top:18px;
    ">

        <form method="post"
              action="/app/admin/resource/{id}/approve{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(22,163,74,.35);
                        background:rgba(22,163,74,.10);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                ✓ Одобрить
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/reject{key_query}"
              style="
                  grid-column:1/-1;
                  display:grid;
                  grid-template-columns:minmax(0,1fr) auto;
                  gap:9px;
              ">

            <input
                type="text"
                name="reason"
                required
                maxlength="500"
                placeholder="Причина отклонения"
                style="
                    min-width:0;
                    min-height:44px;
                    box-sizing:border-box;
                    padding:0 13px;
                    border-radius:12px;
                    border:1px solid rgba(220,38,38,.30);
                    background:rgba(255,255,255,.04);
                    color:inherit;
                    font-size:14px;
                ">

            <button type="submit"
                    style="
                        min-height:44px;
                        padding:0 16px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.38);
                        background:rgba(220,38,38,.10);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                ✕ Отклонить
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/toggle-premium{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(214,183,122,.42);
                        background:rgba(214,183,122,.10);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                {premium_label}
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/toggle-active{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.28);
                        background:rgba(220,38,38,.07);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                {active_label}
            </button>
        </form>

        <a href="/app/resource/{id}"
           target="_blank"
           style="
               min-height:42px;
               border-radius:12px;
               border:1px solid rgba(255,255,255,.12);
               display:flex;
               align-items:center;
               justify-content:center;
               text-decoration:none;
               color:inherit;
               font-weight:800;
           ">
            Открыть ресурс
        </a>

    </div>
</article>
"#,
            id = id,
            title = safe_title,
            category = safe_category,
            description = safe_description,
            rating = rating,
            votes = votes,
            moderation_badge = moderation_badge,
            rejection_html = rejection_html,
            premium_badge = premium_badge,
            active_badge = active_badge,
            premium_label = premium_label,
            active_label = active_label,
            key_query = key_query,
        ));
    }

    let safe_q = templates::escape_html(q);
    let safe_filter = templates::escape_html(filter);

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Moderation · ResursMap</title>
<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">MODERATION CENTER</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {shield}
        Администрирование
    </div>

    <h1>Модерация ресурсов</h1>

    <p>
        Проверка, Premium и видимость ресурсов.
    </p>

</section>

<form method="get"
      action="/app/admin/resources"
      style="display:flex;gap:8px;margin-bottom:16px;">

    <input type="hidden" name="filter" value="{filter}">

    <input
        type="search"
        name="q"
        value="{q}"
        placeholder="Поиск по названию, категории, описанию..."
        style="
            flex:1;
            min-width:0;
            padding:13px 14px;
            border-radius:14px;
            border:1px solid rgba(255,255,255,.14);
            background:rgba(255,255,255,.04);
            color:inherit;
            font-size:15px;
        "
    >

    <button type="submit"
            style="
                min-width:92px;
                border-radius:14px;
                border:1px solid rgba(214,183,122,.35);
                background:rgba(214,183,122,.10);
                color:inherit;
                font-weight:800;
                cursor:pointer;
            ">
        Найти
    </button>

</form>

<div style="
    display:flex;
    gap:8px;
    flex-wrap:wrap;
    margin-bottom:20px;
">

    <a href="/app/admin/reports{key_query}"
       style="
           padding:9px 12px;
           border-radius:999px;
           text-decoration:none;
           border:1px solid rgba(220,38,38,.38);
           background:rgba(220,38,38,.08);
           color:inherit;
           font-weight:800;
       ">
        🚩 Жалобы ({pending_reports_count})
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=all"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(255,255,255,.14);color:inherit;font-weight:700;">
        Все
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=pending"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(217,119,6,.35);color:inherit;font-weight:700;">
        Ожидают
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=verified"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(22,163,74,.35);color:inherit;font-weight:700;">
        Проверены
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=rejected"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(220,38,38,.35);color:inherit;font-weight:700;">
        Отклонённые
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=premium"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(214,183,122,.45);color:inherit;font-weight:700;">
        Premium
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=hidden"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(220,38,38,.32);color:inherit;font-weight:700;">
        Скрытые
    </a>

</div>

<div style="
    display:grid;
    grid-template-columns:repeat(auto-fit,minmax(140px,1fr));
    gap:10px;
    margin-bottom:24px;
">

    <div class="card">
        <div class="card-content">
            <div class="card-title">{pending}</div>
            <div class="card-meta">Ожидают проверки</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{verified_count}</div>
            <div class="card-meta">Проверены</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{rejected_count}</div>
            <div class="card-meta">Отклонены</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{premium_count}</div>
            <div class="card-meta">Premium</div>
        </div>
    </div>

</div>

<section style="margin-bottom:20px;">

    <div style="
        display:flex;
        align-items:center;
        justify-content:space-between;
        gap:12px;
        flex-wrap:wrap;
        margin-bottom:12px;
    ">
        <strong>Найдено: {result_count}</strong>
        <span style="font-size:12px;color:var(--muted);">
            Массовые действия применяются к текущей выборке
        </span>
    </div>

    <div style="
        display:grid;
        grid-template-columns:repeat(auto-fit,minmax(150px,1fr));
        gap:8px;
    ">

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=verify">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(22,163,74,.35);
                           background:rgba(22,163,74,.10);
                           color:inherit;font-weight:800;cursor:pointer;">
                ✓ Одобрить найденные
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=unverify">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(217,119,6,.35);
                           background:rgba(217,119,6,.08);
                           color:inherit;font-weight:800;cursor:pointer;">
                Снять проверку
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=premium">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(214,183,122,.42);
                           background:rgba(214,183,122,.10);
                           color:inherit;font-weight:800;cursor:pointer;">
                ★ Premium ON
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=hide">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(220,38,38,.30);
                           background:rgba(220,38,38,.07);
                           color:inherit;font-weight:800;cursor:pointer;">
                Скрыть найденные
            </button>
        </form>

    </div>
</section>

<section>
{cards}
</section>

</main>

</body>
</html>"#,
        style = templates::base_style(),
        logo = templates::icon("map"),
        shield = templates::icon("user"),
        pending = pending,
        pending_reports_count = pending_reports_count,
        verified_count = verified_count,
        rejected_count = rejected_count,
        premium_count = premium_count,
        cards = cards,
        key_query = key_query,
        key_join = key_join,
        filter = safe_filter,
        q = safe_q,
        result_count = result_count,
    ))
}

pub async fn admin_bulk_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<BTreeMap<String, String>>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    let filter = params.get("filter").map(|s| s.as_str()).unwrap_or("all");
    let q = params.get("q").map(|s| s.trim()).unwrap_or("");
    let action = params.get("action").map(|s| s.as_str()).unwrap_or("");

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

    if !is_admin_session(&state, &headers) {
        return (
            StatusCode::FORBIDDEN,
            Html("<h1>403</h1><p>Доступ запрещён</p>".to_string()),
        )
            .into_response();
    }

    let filter_clause = match filter {
        "pending" => "is_verified = 0 AND is_active = 1",
        "verified" => "is_verified = 1",
        "premium" => "is_premium = 1",
        "hidden" => "is_active = 0",
        _ => "1 = 1",
    };

    let action_sql = match action {
        "verify" => "is_verified = 1",
        "unverify" => "is_verified = 0",
        "premium" => "is_premium = 1",
        "hide" => "is_active = 0",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html("<h1>400</h1><p>Неизвестное действие</p>".to_string()),
            )
                .into_response();
        }
    };

    let search_clause = if q.is_empty() {
        "1 = 1"
    } else {
        "(LOWER(title) LIKE LOWER(?1)
          OR LOWER(category) LIKE LOWER(?1)
          OR LOWER(description) LIKE LOWER(?1))"
    };

    let sql = format!(
        "UPDATE resources
         SET {},
             updated_at = strftime('%s','now')
         WHERE {}
           AND {}",
        action_sql, filter_clause, search_clause
    );

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

    let changed = if q.is_empty() {
        db.execute(&sql, []).unwrap_or(0)
    } else {
        let pattern = format!("%{}%", q);

        db.execute(&sql, rusqlite::params![pattern]).unwrap_or(0)
    };

    drop(db);

    let filter_url = urlencoding::encode(filter);
    let q_url = urlencoding::encode(q);

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Готово · ResursMap</title>
<meta http-equiv="refresh" content="1;url=/app/admin/resources?filter={filter_url}&amp;q={q_url}">
</head>
<body style="font-family:system-ui;padding:30px;">
<h2>Изменено ресурсов: {changed}</h2>
<p>Возвращаемся в модерацию…</p>
</body>
</html>"#,
        filter_url = filter_url,
        q_url = q_url,
        changed = changed,
    ))
    .into_response()
}

async fn admin_toggle_field(
    state: &AppState,
    headers: &HeaderMap,
    id: i64,
    field: &str,
) -> Response {
    if !is_admin_session(state, headers) {
        return (StatusCode::FORBIDDEN, Html("<h1>403</h1>".to_string())).into_response();
    }

    let allowed = ["is_verified", "is_premium", "is_active"];

    if !allowed.contains(&field) {
        return (StatusCode::BAD_REQUEST, Html("<h1>400</h1>".to_string())).into_response();
    }

    let sql = format!(
        "UPDATE resources
         SET {0} = CASE WHEN {0}=1 THEN 0 ELSE 1 END,
             updated_at = strftime('%s','now')
         WHERE id=?1",
        field
    );

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
    let _ = db.execute(&sql, rusqlite::params![id]);
    drop(db);

    Html(r#"<meta http-equiv="refresh" content="0;url=/app/admin/resources">"#.to_string())
        .into_response()
}

pub async fn admin_toggle_verified(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    admin_toggle_field(&state, &headers, id, "is_verified").await
}

pub async fn admin_toggle_premium(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    admin_toggle_field(&state, &headers, id, "is_premium").await
}

pub async fn admin_toggle_active(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    admin_toggle_field(&state, &headers, id, "is_active").await
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

pub async fn api_report_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ReportResourcePayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let reason = payload.reason.trim();

    if !input_text_is_valid(reason, 3, 500) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_reason"
            })),
        )
            .into_response();
    }

    let resource_exists: bool = {
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

        db.query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM resources
                WHERE id = ?1
                  AND is_active = 1
                  AND moderation_status = 'approved'
            )",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .unwrap_or(false)
    };

    if !resource_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "resource_not_found"
            })),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "resource_report", 6, 3600).await
    {
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

    let result = db.execute(
        "INSERT INTO resource_reports (
            reporter_user_id,
            resource_id,
            reason,
            status,
            created_at,
            updated_at
        )
        VALUES (
            ?1,
            ?2,
            ?3,
            'pending',
            strftime('%s','now'),
            strftime('%s','now')
        )

        ON CONFLICT(reporter_user_id, resource_id)
        DO UPDATE SET
            reason = excluded.reason,
            status = 'pending',
            updated_at = strftime('%s','now')",
        rusqlite::params![user_id, id, reason,],
    );

    drop(db);

    match result {
        Ok(_) => Json(json!({
            "ok": true,
            "status": "pending"
        }))
        .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "database_error",
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn api_favorite_toggle(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "favorite_toggle", 60, 60).await
    {
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

    let allowed: bool = db
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM resources
                WHERE id = ?1
                  AND is_active = 1
                  AND moderation_status = 'approved'
            )",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !allowed {
        drop(db);

        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "resource_not_found"
            })),
        )
            .into_response();
    }

    let exists: bool = db
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM favorites
                WHERE user_id = ?1
                  AND resource_id = ?2
            )",
            rusqlite::params![user_id, id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    let favorite = if exists {
        let _ = db.execute(
            "DELETE FROM favorites
             WHERE user_id = ?1
               AND resource_id = ?2",
            rusqlite::params![user_id, id],
        );

        false
    } else {
        let _ = db.execute(
            "INSERT OR IGNORE INTO favorites
             (user_id, resource_id, created_at)
             VALUES (?1, ?2, strftime('%s','now'))",
            rusqlite::params![user_id, id],
        );

        true
    };

    drop(db);

    Json(json!({
        "ok": true,
        "favorite": favorite
    }))
    .into_response()
}

pub async fn api_favorite_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return Json(json!({
                "ok": true,
                "favorite": false,
                "authenticated": false
            }))
            .into_response();
        }
    };

    let favorite: bool = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db
            .query_row(
                "SELECT EXISTS(
                    SELECT 1
                    FROM favorites
                    WHERE user_id = ?1
                      AND resource_id = ?2
                )",
                rusqlite::params![user_id, id],
                |row| row.get(0),
            )
            .unwrap_or(false),
        Err(_) => false,
    };

    Json(json!({
        "ok": true,
        "favorite": favorite,
        "authenticated": true
    }))
    .into_response()
}

pub async fn api_resource_vote(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let score = payload.get("score").and_then(|v| v.as_i64()).unwrap_or(0);

    if !(1..=5).contains(&score) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_vote"
            })),
        )
            .into_response();
    }

    let client_id = format!("tg:{}", user_id);

    let allowed: bool = {
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

        db.query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM resources
                WHERE id = ?1
                  AND is_active = 1
                  AND moderation_status = 'approved'
            )",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .unwrap_or(false)
    };

    if !allowed {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "resource_not_found"
            })),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "resource_rating", 60, 60).await
    {
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

    let transaction_result = (|| -> rusqlite::Result<(f64, i64)> {
        let tx = db.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO resource_votes
                (resource_id, client_id, score, updated_at)
             VALUES
                (?1, ?2, ?3, strftime('%s','now'))
             ON CONFLICT(resource_id, client_id)
             DO UPDATE SET
                score = excluded.score,
                updated_at = excluded.updated_at",
            rusqlite::params![id, client_id, score],
        )?;

        let stats: (f64, i64) = tx.query_row(
            "SELECT
                COALESCE(AVG(score), 0),
                COUNT(*)
             FROM resource_votes
             WHERE resource_id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        tx.execute(
            "UPDATE resources
             SET rating = ?1,
                 votes = ?2,
                 updated_at = strftime('%s','now')
             WHERE id = ?3",
            rusqlite::params![stats.0, stats.1, id],
        )?;

        tx.commit()?;
        Ok(stats)
    })();

    let stats = match transaction_result {
        Ok(stats) => stats,

        Err(err) => {
            drop(db);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": "vote_write_failed",
                    "message": err.to_string()
                })),
            )
                .into_response();
        }
    };

    drop(db);

    Json(json!({
        "ok": true,
        "resource_id": id,
        "rating": stats.0,
        "votes": stats.1
    }))
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub struct ContactRequestPayload {
    pub public_id: String,
    pub message: String,
}

pub async fn api_contact_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ContactRequestPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let sender_user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let public_id = payload.public_id.trim();
    let message = payload.message.trim();

    if public_id.is_empty()
        || public_id.len() > 64
        || !public_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_public_id"
            })),
        )
            .into_response();
    }

    if !input_text_is_valid(message, 2, 500) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_message"
            })),
        )
            .into_response();
    }

    let receiver_user_id: Option<i64> = {
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

        db.query_row(
            "SELECT CAST(
                substr(client_id, 4)
                AS INTEGER
            )
             FROM profiles
             WHERE public_id = ?1
               AND client_id LIKE 'tg:%'
             LIMIT 1",
            rusqlite::params![public_id],
            |row| row.get(0),
        )
        .ok()
    };

    let receiver_user_id = match receiver_user_id {
        Some(id) if id > 0 => id,

        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "ok": false,
                    "error": "user_not_found"
                })),
            )
                .into_response();
        }
    };

    if sender_user_id == receiver_user_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "cannot_contact_self"
            })),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, sender_user_id, "contact_request", 6, 600).await
    {
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

    let existing_status: Option<String> = db
        .query_row(
            "SELECT status
             FROM contact_requests
             WHERE sender_user_id = ?1
               AND receiver_user_id = ?2
             LIMIT 1",
            rusqlite::params![sender_user_id, receiver_user_id,],
            |row| row.get(0),
        )
        .ok();

    if let Some(status) = existing_status.as_deref() {
        if status == "pending" {
            drop(db);

            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "ok": false,
                    "error": "request_already_pending"
                })),
            )
                .into_response();
        }

        if status == "accepted" {
            drop(db);

            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "ok": false,
                    "error": "already_connected"
                })),
            )
                .into_response();
        }
    }

    let result = db.execute(
        "INSERT INTO contact_requests (
            sender_user_id,
            receiver_user_id,
            message,
            status,
            created_at,
            updated_at
         )
         VALUES (
            ?1,
            ?2,
            ?3,
            'pending',
            strftime('%s','now'),
            strftime('%s','now')
         )

         ON CONFLICT(sender_user_id, receiver_user_id)
         DO UPDATE SET
            message = excluded.message,
            status = 'pending',
            updated_at = strftime('%s','now')",
        rusqlite::params![sender_user_id, receiver_user_id, message,],
    );

    drop(db);

    match result {
        Ok(_) => Json(json!({
            "ok": true,
            "status": "pending"
        }))
        .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "database_error",
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn api_profile_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let client_id = format!("tg:{}", user_id);

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

    let profile: Option<(String, String, String, i64, String, i64)> = db
        .query_row(
            "SELECT
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until
             FROM profiles
             WHERE client_id = ?1",
            rusqlite::params![&client_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .ok();

    drop(db);

    let (username, first_name, last_name, open_contact, intent_text, intent_until) = profile
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                0,
                String::new(),
                0,
            )
        });

    Json(json!({
        "ok": true,
        "username": username,
        "first_name": first_name,
        "last_name": last_name,
        "open_contact": open_contact == 1,
        "intent_text": intent_text,
        "intent_until": intent_until
    }))
    .into_response()
}

pub async fn api_profile_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let client_id = format!("tg:{}", user_id);

    let open_contact = payload
        .get("open_contact")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let intent_text = payload
        .get("intent_text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if !input_text_is_valid(intent_text, 0, 300) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_intent"
            })),
        )
            .into_response();
    }

    let duration_days = payload
        .get("duration_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let allowed_days = [0_i64, 1, 3, 7, 30];

    if !allowed_days.contains(&duration_days) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_duration"
            })),
        )
            .into_response();
    }

    let intent_until = if intent_text.is_empty() {
        0
    } else if duration_days == 0 {
        0
    } else {
        unix_now() + duration_days * 86_400
    };

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "profile_update", 30, 600).await
    {
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

    let result = db.execute(
        "INSERT INTO profiles (
            client_id,
            open_contact,
            intent_text,
            intent_until,
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
            open_contact = excluded.open_contact,
            intent_text = excluded.intent_text,
            intent_until = excluded.intent_until,
            updated_at = strftime('%s','now')",
        rusqlite::params![
            &client_id,
            if open_contact { 1 } else { 0 },
            intent_text,
            intent_until,
        ],
    );

    drop(db);

    match result {
        Ok(_) => Json(json!({
            "ok": true,
            "open_contact": open_contact,
            "intent_text": intent_text,
            "intent_until": intent_until
        }))
        .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "database_error",
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn api_open_count(State(state): State<AppState>) -> Json<serde_json::Value> {
    let count: i64 = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db
            .query_row(
                "SELECT COUNT(*)
                 FROM profiles
                 WHERE open_contact = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0),
        Err(_) => 0,
    };

    Json(json!({
        "count": count
    }))
}

// ============================================================
// TASK 7.3B — MODERATION APPROVE / REJECT
// ============================================================

#[derive(serde::Deserialize)]
pub struct RejectResourceForm {
    pub reason: String,
}

pub async fn admin_approve_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_admin_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
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

    let result = db.execute(
        "UPDATE resources
         SET moderation_status = 'approved',
             rejection_reason = '',
             is_verified = 1,
             is_active = 1,
             updated_at = strftime('%s','now')
         WHERE id = ?1",
        rusqlite::params![id],
    );

    if result.is_ok() {
        let owner: Option<(String, String)> = db
            .query_row(
                "SELECT client_id, title
                 FROM resources
                 WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((client_id, resource_title)) = owner {
            if let Some(user_id) = telegram_owner_user_id(&client_id) {
                let _ = db.execute(
                    "INSERT INTO user_notifications (
                        user_id,
                        resource_id,
                        kind,
                        title,
                        message,
                        is_read,
                        created_at
                     )
                     VALUES (
                        ?1,
                        ?2,
                        'resource_approved',
                        'Ресурс одобрен',
                        ?3,
                        0,
                        strftime('%s','now')
                     )",
                    rusqlite::params![
                        user_id,
                        id,
                        format!(
                            "Ваш ресурс «{}» прошёл модерацию и опубликован.",
                            resource_title
                        ),
                    ],
                );
            }
        }
    }

    drop(db);

    match result {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            [(header::LOCATION, "/app/admin/resources?filter=pending")],
        )
            .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка одобрения ресурса: {}", err),
        )
            .into_response(),
    }
}

pub async fn admin_reject_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<RejectResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_admin_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
    }

    let reason = form.reason.trim();

    if !input_text_is_valid(reason, 1, 500) {
        return (
            StatusCode::BAD_REQUEST,
            "Причина отклонения должна содержать от 1 до 500 символов",
        )
            .into_response();
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

    let result = db.execute(
        "UPDATE resources
         SET moderation_status = 'rejected',
             rejection_reason = ?2,
             is_verified = 0,
             is_active = 0,
             updated_at = strftime('%s','now')
         WHERE id = ?1",
        rusqlite::params![id, reason],
    );

    if result.is_ok() {
        let owner: Option<(String, String)> = db
            .query_row(
                "SELECT client_id, title
                 FROM resources
                 WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((client_id, resource_title)) = owner {
            if let Some(user_id) = telegram_owner_user_id(&client_id) {
                let message = format!("Ресурс «{}» отклонён. Причина: {}", resource_title, reason);

                let _ = db.execute(
                    "INSERT INTO user_notifications (
                        user_id,
                        resource_id,
                        kind,
                        title,
                        message,
                        is_read,
                        created_at
                     )
                     VALUES (
                        ?1,
                        ?2,
                        'resource_rejected',
                        'Ресурс требует исправления',
                        ?3,
                        0,
                        strftime('%s','now')
                     )",
                    rusqlite::params![user_id, id, message,],
                );
            }
        }
    }

    drop(db);

    match result {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            [(header::LOCATION, "/app/admin/resources?filter=pending")],
        )
            .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка отклонения ресурса: {}", err),
        )
            .into_response(),
    }
}
