use super::common::{
    csrf_rejected_response, rate_limit_retry_after, request_is_cross_site, unix_now,
};
use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Form, Json,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
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
pub(super) const EMAIL_USER_ID_BASE: i64 = 4_000_000_000_000_000_000;

const USER_SESSION_TTL_SECONDS: i64 = 2_592_000;
static USER_SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Deserialize)]
pub struct EmailCodeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailCodeVerifyRequest {
    pub email: String,
    pub code: String,
}

pub(super) fn normalize_email(raw: &str) -> Option<String> {
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

pub(super) fn auth_redirect_target(raw: Option<&str>) -> String {
    let next = raw.unwrap_or("").trim();

    if next.starts_with("/app") && !next.starts_with("//") && !next.contains("://") {
        return next.to_string();
    }

    "/app".to_string()
}

pub(super) fn email_rate_limit_id(state: &AppState, email: &str) -> i64 {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(b"email-rate-limit:");
    mac.update(email.as_bytes());

    let digest = mac.finalize().into_bytes();

    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);

    i64::from_be_bytes(bytes) & i64::MAX
}

fn generate_email_code() -> String {
    let mut bytes = [0u8; 4];
    getrandom::getrandom(&mut bytes).expect("secure random");

    let value = u32::from_be_bytes(bytes) % 1_000_000;

    format!("{value:06}")
}

fn cookie_security_flags() -> &'static str {
    match std::env::var("RESURSMAP_COOKIE_SECURE").as_deref() {
        Ok("0") | Ok("false") | Ok("False") => "HttpOnly; SameSite=Lax",
        _ => "HttpOnly; Secure; SameSite=Lax",
    }
}

pub(super) fn append_user_session_cookie(response: &mut Response, session: &str) {
    let cookie = format!(
        "resursmap_user={session}; Path=/; {}; Max-Age=2592000",
        cookie_security_flags()
    );

    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

pub(super) fn ensure_profile_public_id(
    transaction: &rusqlite::Transaction<'_>,
    user_id: i64,
) -> Result<(), &'static str> {
    transaction
        .execute(
            "UPDATE profiles
             SET public_id = lower(hex(randomblob(16)))
             WHERE user_id = ?1
               AND (public_id IS NULL OR public_id = '')",
            rusqlite::params![user_id],
        )
        .map_err(|_| "profile_update_failed")?;

    Ok(())
}

fn provision_telegram_account(
    db: &mut rusqlite::Connection,
    telegram_id: i64,
    username: &str,
    first_name: &str,
    last_name: &str,
) -> Result<i64, &'static str> {
    if telegram_id <= 0 {
        return Err("invalid_telegram_id");
    }

    let now = unix_now();
    let client_id = format!("tg:{telegram_id}");
    let subject = telegram_id.to_string();

    let transaction = db.transaction().map_err(|_| "transaction_failed")?;

    transaction
        .execute(
            "INSERT OR IGNORE INTO users (
                id,
                created_at,
                updated_at,
                is_active,
                telegram_id
             )
             VALUES (?1, ?2, ?2, 1, ?1)",
            rusqlite::params![telegram_id, now],
        )
        .map_err(|_| "user_create_failed")?;

    transaction
        .execute(
            "UPDATE users
             SET telegram_id = ?2,
                 updated_at = ?3
             WHERE id = ?1",
            rusqlite::params![telegram_id, telegram_id, now],
        )
        .map_err(|_| "user_create_failed")?;

    transaction
        .execute(
            "INSERT OR IGNORE INTO auth_identities (
                user_id,
                provider,
                provider_subject,
                verified_at,
                created_at,
                updated_at
             )
             VALUES (?1, 'telegram', ?2, ?3, ?3, ?3)",
            rusqlite::params![telegram_id, &subject, now],
        )
        .map_err(|_| "identity_create_failed")?;

    transaction
        .execute(
            "INSERT INTO profiles (
                client_id,
                user_id,
                username,
                first_name,
                last_name,
                public_id,
                updated_at
             )
             VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                lower(hex(randomblob(16))),
                ?6
             )
             ON CONFLICT(client_id) DO UPDATE SET
                user_id = excluded.user_id,
                username = excluded.username,
                first_name = excluded.first_name,
                last_name = excluded.last_name,
                updated_at = excluded.updated_at,
                public_id = CASE
                    WHEN profiles.public_id IS NULL OR profiles.public_id = ''
                    THEN lower(hex(randomblob(16)))
                    ELSE profiles.public_id
                END",
            rusqlite::params![
                &client_id,
                telegram_id,
                username,
                first_name,
                last_name,
                now,
            ],
        )
        .map_err(|_| "profile_create_failed")?;

    ensure_profile_public_id(&transaction, telegram_id)?;

    transaction.commit().map_err(|_| "commit_failed")?;

    Ok(telegram_id)
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

    let code = generate_email_code();
    let expires_at = unix_now() + EMAIL_CODE_TTL_SECONDS;
    let code_hash = hash_email_code(&state, &email, &code, expires_at);

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

    if let Err(error) = send_email_code(&email, &code).await {
        eprintln!("email auth send failed: {error}");

        let _ = db.execute(
            "UPDATE email_login_codes
             SET consumed_at = ?2
             WHERE email = ?1
               AND consumed_at = 0",
            rusqlite::params![&email, unix_now()],
        );

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": "mail_unavailable"
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
                    is_active,
                    telegram_id
                 )
                 VALUES (?1, ?2, ?2, 1, NULL)",
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
                    public_id,
                    updated_at
                 )
                 VALUES (
                    ?1,
                    ?2,
                    lower(hex(randomblob(16))),
                    ?3
                 )",
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

    if ensure_profile_public_id(&transaction, user_id).is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "profile_update_failed"
            })),
        )
            .into_response();
    }

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

    let session = match create_user_session(&state, user_id, &headers) {
        Ok(session) => session,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": error
                })),
            )
                .into_response();
        }
    };

    let mut response = (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "user_id": user_id
        })),
    )
        .into_response();

    append_user_session_cookie(&mut response, &session);

    response
}

fn hash_user_session_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub(super) fn current_session_public_id(headers: &HeaderMap) -> Option<String> {
    let token = extract_user_session_token(headers)?;
    Some(token.split_once('.')?.0.to_string())
}

fn extract_user_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    let token = cookie_header
        .split(';')
        .map(|value| value.trim())
        .find_map(|value| value.strip_prefix("resursmap_user="))?;

    let (public_id, secret) = token.split_once('.')?;

    if public_id.len() != 32
        || secret.len() != 64
        || !public_id.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !secret.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }

    Some(token.to_string())
}

fn revoke_user_session(state: &AppState, headers: &HeaderMap) {
    let Some(token) = extract_user_session_token(headers) else {
        return;
    };

    let Some((public_id, _)) = token.split_once('.') else {
        return;
    };

    let Ok(db) = crate::db::pool::get_connection(&state.db_pool) else {
        return;
    };

    let _ = db.execute(
        "UPDATE user_sessions
         SET revoked_at = ?2
         WHERE session_public_id = ?1
           AND revoked_at IS NULL",
        rusqlite::params![public_id, unix_now()],
    );
}

pub(super) fn create_user_session(
    state: &AppState,
    user_id: i64,
    headers: &HeaderMap,
) -> Result<String, &'static str> {
    type HmacSha256 = Hmac<Sha256>;

    let now = unix_now();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system_time_error")?
        .as_nanos();

    let sequence = USER_SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let payload = format!("user-v1:{user_id}:{now}:{nanos}:{sequence}");

    let mut mac =
        HmacSha256::new_from_slice(state.admin_key.as_bytes()).map_err(|_| "session_key_error")?;

    mac.update(payload.as_bytes());

    let secret = hex::encode(mac.finalize().into_bytes());
    let public_digest = Sha256::digest(format!("{payload}:{secret}").as_bytes());
    let public_id = hex::encode(public_digest)[..32].to_string();
    let token = format!("{public_id}.{secret}");
    let session_hash = hash_user_session_token(&token);
    let expires_at = now + USER_SESSION_TTL_SECONDS;

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .unwrap_or("")
        .chars()
        .take(64)
        .collect::<String>();

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .chars()
        .take(255)
        .collect::<String>();

    let db = crate::db::pool::get_connection(&state.db_pool).map_err(|_| "database_unavailable")?;

    db.execute(
        "INSERT INTO user_sessions (
            session_public_id,
            user_id,
            session_hash,
            ip_address,
            user_agent,
            created_at,
            last_seen_at,
            expires_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
        rusqlite::params![
            public_id,
            user_id,
            session_hash,
            ip_address,
            user_agent,
            now,
            expires_at
        ],
    )
    .map_err(|_| "session_store_failed")?;

    let _ = db.execute(
        "DELETE FROM user_sessions
         WHERE expires_at <= ?1
            OR (
                revoked_at IS NOT NULL
                AND revoked_at <= ?1 - 86400
            )",
        rusqlite::params![now],
    );

    Ok(token)
}

pub(super) fn verify_user_session(state: &AppState, headers: &HeaderMap) -> Option<i64> {
    let token = extract_user_session_token(headers)?;
    let (public_id, _) = token.split_once('.')?;
    let session_hash = hash_user_session_token(&token);
    let now = unix_now();

    let db = crate::db::pool::get_connection(&state.db_pool).ok()?;

    let user_id: i64 = db
        .query_row(
            "SELECT user_id
             FROM user_sessions
             WHERE session_public_id = ?1
               AND session_hash = ?2
               AND revoked_at IS NULL
               AND expires_at > ?3",
            rusqlite::params![public_id, session_hash, now],
            |row| row.get(0),
        )
        .ok()?;

    let is_active: i64 = db
        .query_row(
            "SELECT is_active
             FROM users
             WHERE id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .ok()?;

    if is_active != 1 {
        return None;
    }

    let _ = db.execute(
        "UPDATE user_sessions
         SET last_seen_at = ?2
         WHERE session_public_id = ?1",
        rusqlite::params![public_id, now],
    );

    Some(user_id)
}

#[derive(Debug, Clone)]
pub(super) struct AuthenticatedUser {
    pub user_id: i64,
    pub client_id: String,
}

pub(super) fn verify_authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<AuthenticatedUser> {
    let user_id = verify_user_session(state, headers)?;
    let db = crate::db::pool::get_connection(&state.db_pool).ok()?;

    let client_id: String = db
        .query_row(
            "SELECT client_id
             FROM profiles
             WHERE user_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .ok()?;

    if client_id.is_empty() {
        return None;
    }

    Some(AuthenticatedUser { user_id, client_id })
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

#[derive(Deserialize)]
pub struct RevokeSessionForm {
    pub session_public_id: String,
}

pub async fn app_revoke_other_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return (
                StatusCode::SEE_OTHER,
                [(header::LOCATION, "/login?next=/app/me")],
            )
                .into_response();
        }
    };

    let current_public_id = current_session_public_id(&headers).unwrap_or_default();
    let now = unix_now();

    if let Ok(db) = crate::db::pool::get_connection(&state.db_pool) {
        let _ = db.execute(
            "UPDATE user_sessions
             SET revoked_at = ?3
             WHERE user_id = ?1
               AND session_public_id <> ?2
               AND revoked_at IS NULL",
            rusqlite::params![user_id, current_public_id, now],
        );
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/me?sessions=revoked")],
    )
        .into_response()
}

pub async fn app_revoke_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RevokeSessionForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return (
                StatusCode::SEE_OTHER,
                [(header::LOCATION, "/login?next=/app/me")],
            )
                .into_response();
        }
    };

    let session_public_id = form.session_public_id.trim();

    if session_public_id.len() != 32
        || !session_public_id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/app/me")]).into_response();
    }

    if current_session_public_id(&headers).as_deref() == Some(session_public_id) {
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/app/me")]).into_response();
    }

    let now = unix_now();

    if let Ok(db) = crate::db::pool::get_connection(&state.db_pool) {
        let _ = db.execute(
            "UPDATE user_sessions
             SET revoked_at = ?3
             WHERE user_id = ?1
               AND session_public_id = ?2
               AND revoked_at IS NULL",
            rusqlite::params![user_id, session_public_id, now],
        );
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/me?sessions=revoked")],
    )
        .into_response()
}

pub async fn app_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    revoke_user_session(&state, &headers);

    let mut response = (StatusCode::SEE_OTHER, [(header::LOCATION, "/app")]).into_response();

    let cookie = format!(
        "resursmap_user=; Path=/; {}; Max-Age=0",
        cookie_security_flags()
    );

    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }

    response
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

    // TASK 7.16B PROFILE SYNC + unified account model
    let (telegram_username, telegram_first_name, telegram_last_name) =
        telegram_profile_from_init_data(init_data);

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

    let account_user_id = match provision_telegram_account(
        &mut db,
        user_id,
        &telegram_username,
        &telegram_first_name,
        &telegram_last_name,
    ) {
        Ok(account_user_id) => account_user_id,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": error
                })),
            )
                .into_response();
        }
    };

    let session = match create_user_session(&state, account_user_id, &headers) {
        Ok(session) => session,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": error
                })),
            )
                .into_response();
        }
    };

    let mut response = Json(json!({
        "ok": true,
        "user_id": account_user_id
    }))
    .into_response();

    append_user_session_cookie(&mut response, &session);

    response
}
