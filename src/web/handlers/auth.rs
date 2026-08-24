use super::{
    csrf_rejected_response, rate_limit_retry_after, request_is_cross_site, unix_now, AppState,
};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;

pub(super) fn verify_telegram_init_data(init_data: &str, bot_token: &str) -> Option<i64> {
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

pub(super) fn verify_user_session(state: &AppState, headers: &HeaderMap) -> Option<i64> {
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

pub(super) fn create_admin_session(state: &AppState, user_id: i64) -> String {
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

pub(super) fn is_admin_session(state: &AppState, headers: &HeaderMap) -> bool {
    verify_admin_session(state, headers)
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
