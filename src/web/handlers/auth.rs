use super::common::{
    csrf_rejected_response, rate_limit_retry_after, request_is_cross_site, unix_now,
};
use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

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

const EMAIL_CODE_TTL_SECONDS: i64 = 600;
const EMAIL_CODE_MAX_ATTEMPTS: i64 = 5;

// Email IDs live in a reserved positive range so they never collide
// with existing/future Telegram numeric IDs while the compatibility
// migration is active.
const EMAIL_USER_ID_BASE: i64 = 4_000_000_000_000_000_000;

#[derive(Debug, Deserialize)]
pub struct EmailCodeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailCodeVerifyRequest {
    pub email: String,
    pub code: String,
}

fn normalize_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_lowercase();

    if email.is_empty() || email.len() > 254 || email.chars().any(char::is_whitespace) {
        return None;
    }

    let (local, domain) = email.split_once('@')?;

    if local.is_empty()
        || domain.is_empty()
        || local.len() > 64
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
    {
        return None;
    }

    if !domain.contains('.') {
        return None;
    }

    Some(email)
}

fn email_rate_limit_id(state: &AppState, email: &str) -> i64 {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(b"email-rate-limit:");
    mac.update(email.as_bytes());

    let digest = mac.finalize().into_bytes();

    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);

    i64::from_be_bytes(bytes) & i64::MAX
}

fn generate_email_code(state: &AppState, email: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let payload = format!("email-code:{}:{}:{}", email, unix_now(), nanos);

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(payload.as_bytes());

    let digest = mac.finalize().into_bytes();

    let value = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]) % 1_000_000;

    format!("{value:06}")
}

fn hash_email_code(state: &AppState, email: &str, code: &str, expires_at: i64) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let payload = format!("email-login-code:{}:{}:{}", email, code, expires_at);

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(payload.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}

async fn send_email_code(email: &str, code: &str) -> Result<(), String> {
    let api_key = std::env::var("RESEND_API_KEY")
        .map_err(|_| "RESEND_API_KEY is not configured".to_string())?;

    let from = std::env::var("RESURSMAP_MAIL_FROM")
        .unwrap_or_else(|_| "ResursMap <noreply@resursmap.de>".to_string());

    let client = reqwest::Client::new();

    let response = client
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&json!({
            "from": from,
            "to": [email],
            "subject": "Код входа в ResursMap",
            "html": format!(
                "<div style=\"font-family:Arial,sans-serif;max-width:520px;margin:auto;padding:32px\">\
                    <h2 style=\"margin:0 0 18px\">ResursMap</h2>\
                    <p>Ваш код входа:</p>\
                    <div style=\"font-size:34px;font-weight:800;letter-spacing:8px;margin:24px 0\">{}</div>\
                    <p>Код действует 10 минут.</p>\
                    <p style=\"color:#777;font-size:13px\">Если вы не запрашивали вход, просто проигнорируйте это письмо.</p>\
                </div>",
                code
            )
        }))
        .send()
        .await
        .map_err(|_| "mail_transport_error".to_string())?;

    if !response.status().is_success() {
        return Err(format!("mail_provider_status_{}", response.status()));
    }

    Ok(())
}

pub async fn email_auth_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailCodeRequest>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let email = match normalize_email(&payload.email) {
        Some(email) => email,

        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "invalid_email"
                })),
            )
                .into_response();
        }
    };

    let rate_id = email_rate_limit_id(&state, &email);

    // Не больше 5 запросов кода за 10 минут на один email.
    if let Some(retry_after) =
        rate_limit_retry_after(&state, rate_id, "email_code_request", 5, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "ok": false,
                "error": "rate_limited",
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    let code = generate_email_code(&state, &email);
    let expires_at = unix_now() + EMAIL_CODE_TTL_SECONDS;
    let code_hash = hash_email_code(&state, &email, &code, expires_at);

    // Сначала отправляем письмо. Если почтовый сервис не работает,
    // не создаём бесполезный код в БД.
    if let Err(error) = send_email_code(&email, &code).await {
        eprintln!("email auth send failed: {error}");

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": "mail_unavailable"
            })),
        )
            .into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,

        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": "database_unavailable"
                })),
            )
                .into_response();
        }
    };

    // Все предыдущие неиспользованные коды для адреса инвалидируем.
    let _ = db.execute(
        "UPDATE email_login_codes
         SET consumed_at = ?2
         WHERE email = ?1
           AND consumed_at = 0",
        rusqlite::params![&email, unix_now()],
    );

    let result = db.execute(
        "INSERT INTO email_login_codes (
            email,
            code_hash,
            expires_at,
            attempts,
            consumed_at,
            created_at
         )
         VALUES (?1, ?2, ?3, 0, 0, ?4)",
        rusqlite::params![&email, &code_hash, expires_at, unix_now(),],
    );

    if result.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "code_store_failed"
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "expires_in": EMAIL_CODE_TTL_SECONDS
        })),
    )
        .into_response()
}

pub async fn email_auth_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailCodeVerifyRequest>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let email = match normalize_email(&payload.email) {
        Some(email) => email,

        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "invalid_email"
                })),
            )
                .into_response();
        }
    };

    let code = payload.code.trim();

    if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_code"
            })),
        )
            .into_response();
    }

    let rate_id = email_rate_limit_id(&state, &email);

    if let Some(retry_after) =
        rate_limit_retry_after(&state, rate_id, "email_code_verify", 15, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "ok": false,
                "error": "rate_limited",
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    let mut db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,

        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": "database_unavailable"
                })),
            )
                .into_response();
        }
    };

    let row: Option<(i64, String, i64, i64, i64)> = db
        .query_row(
            "SELECT
                id,
                code_hash,
                expires_at,
                attempts,
                consumed_at
             FROM email_login_codes
             WHERE email = ?1
             ORDER BY id DESC
             LIMIT 1",
            rusqlite::params![&email],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .ok();

    let (code_id, expected_hash, expires_at, attempts, consumed_at) = match row {
        Some(row) => row,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "code_not_found"
                })),
            )
                .into_response();
        }
    };

    if consumed_at != 0 {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "code_used"
            })),
        )
            .into_response();
    }

    if expires_at < unix_now() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "code_expired"
            })),
        )
            .into_response();
    }

    if attempts >= EMAIL_CODE_MAX_ATTEMPTS {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "ok": false,
                "error": "too_many_attempts"
            })),
        )
            .into_response();
    }

    let actual_hash = hash_email_code(&state, &email, code, expires_at);

    if actual_hash != expected_hash {
        let _ = db.execute(
            "UPDATE email_login_codes
             SET attempts = attempts + 1
             WHERE id = ?1",
            rusqlite::params![code_id],
        );

        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "wrong_code"
            })),
        )
            .into_response();
    }

    let transaction = match db.transaction() {
        Ok(transaction) => transaction,

        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": "transaction_failed"
                })),
            )
                .into_response();
        }
    };

    let existing_user_id: Option<i64> = transaction
        .query_row(
            "SELECT user_id
             FROM auth_identities
             WHERE provider = 'email'
               AND email = ?1
             LIMIT 1",
            rusqlite::params![&email],
            |row| row.get(0),
        )
        .ok();

    let user_id = if let Some(user_id) = existing_user_id {
        user_id
    } else {
        let next_id: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(id), ?1 - 1) + 1
                 FROM users
                 WHERE id >= ?1",
                rusqlite::params![EMAIL_USER_ID_BASE],
                |row| row.get(0),
            )
            .unwrap_or(EMAIL_USER_ID_BASE);

        if transaction
            .execute(
                "INSERT INTO users (
                    id,
                    created_at,
                    updated_at,
                    is_active
                 )
                 VALUES (?1, ?2, ?2, 1)",
                rusqlite::params![next_id, unix_now()],
            )
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": "user_create_failed"
                })),
            )
                .into_response();
        }

        if transaction
            .execute(
                "INSERT INTO auth_identities (
                    user_id,
                    provider,
                    provider_subject,
                    email,
                    verified_at,
                    created_at,
                    updated_at
                 )
                 VALUES (
                    ?1,
                    'email',
                    ?2,
                    ?2,
                    ?3,
                    ?3,
                    ?3
                 )",
                rusqlite::params![next_id, &email, unix_now()],
            )
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": "identity_create_failed"
                })),
            )
                .into_response();
        }

        let client_id = format!("user:{next_id}");

        if transaction
            .execute(
                "INSERT OR IGNORE INTO profiles (
                    client_id,
                    user_id,
                    updated_at
                 )
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![&client_id, next_id, unix_now(),],
            )
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": "profile_create_failed"
                })),
            )
                .into_response();
        }

        next_id
    };

    if transaction
        .execute(
            "UPDATE email_login_codes
             SET consumed_at = ?2
             WHERE id = ?1
               AND consumed_at = 0",
            rusqlite::params![code_id, unix_now()],
        )
        .ok()
        != Some(1)
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "code_already_used"
            })),
        )
            .into_response();
    }

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "commit_failed"
            })),
        )
            .into_response();
    }

    let session = create_user_session(&state, user_id);

    let cookie = format!(
        "resursmap_user={session}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=2592000"
    );

    let mut response = (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "user_id": user_id
        })),
    )
        .into_response();

    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }

    response
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

    if mac.verify_slice(&signature).is_err() {
        return None;
    }

    let db = crate::db::pool::get_connection(&state.db_pool).ok()?;

    let is_active: i64 = db
        .query_row(
            "SELECT is_active
             FROM users
             WHERE id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .ok()?;

    (is_active == 1).then_some(user_id)
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
    let head_extra = r#"<script src="https://telegram.org/js/telegram-web-app.js"></script>"#;

    let main_html = r#"
<div style="max-width:520px;margin:auto;">
    <h2>RESURSMAP</h2>
    <p id="status">Подключаем Telegram…</p>
</div>
"#;

    let body_after = r#"
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
"#;

    Html(crate::web::templates::page_document(
        "ResursMap",
        head_extra,
        "",
        main_html,
        "",
        body_after,
    ))
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
