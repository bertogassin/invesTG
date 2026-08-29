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
const EMAIL_USER_ID_BASE: i64 = 4_000_000_000_000_000_000;

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

fn append_user_session_cookie(response: &mut Response, session: &str) {
    let cookie = format!(
        "resursmap_user={session}; Path=/; {}; Max-Age=2592000",
        cookie_security_flags()
    );

    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

fn ensure_profile_public_id(
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
    db: &rusqlite::Connection,
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

    let transaction = db
        .transaction()
        .map_err(|_| "transaction_failed")?;

    transaction
        .execute(
            "INSERT OR IGNORE INTO users (
                id,
                created_at,
                updated_at,
                is_active
             )
             VALUES (?1, ?2, ?2, 1)",
            rusqlite::params![telegram_id, now],
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

    transaction
        .commit()
        .map_err(|_| "commit_failed")?;

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

fn create_user_session(
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

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes())
        .map_err(|_| "session_key_error")?;

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

pub async fn app_auth_page() -> Html<String> {
    let main_html = r####"
<div style="
    width:min(100%,520px);
    margin:0 auto;
    padding:18px;
">
    <section class="card"
             style="
                 display:block;
                 padding:26px 22px;
                 border-color:rgba(214,183,122,.24);
                 background:
                     radial-gradient(
                         circle at 85% 5%,
                         rgba(214,183,122,.12),
                         transparent 34%
                     ),
                     linear-gradient(
                         145deg,
                         rgba(255,255,255,.055),
                         rgba(255,255,255,.018)
                     );
                 box-shadow:
                     0 24px 70px rgba(0,0,0,.30),
                     inset 0 1px 0 rgba(255,255,255,.065);
             ">
        <div style="
            display:flex;
            align-items:center;
            gap:9px;
            margin-bottom:14px;
            color:var(--gold-light);
            font-size:12px;
            font-weight:800;
            letter-spacing:.14em;
        ">
            <span aria-hidden="true"
                  style="
                      width:7px;
                      height:7px;
                      border-radius:50%;
                      background:var(--gold);
                      box-shadow:0 0 16px rgba(214,183,122,.65);
                  ">
            </span>
            RESURSMAP
        </div>

        <h1 style="
            margin:0 0 10px;
            color:var(--text);
            font-size:clamp(30px,8vw,42px);
            line-height:1.04;
            letter-spacing:-.035em;
        ">
            Вход в аккаунт
        </h1>

        <p style="
            margin:0 0 24px;
            color:var(--muted);
            font-size:16px;
            line-height:1.58;
        ">
            Карта и поиск работают без регистрации.
            Вход нужен только для сообщений, избранного и публикаций.
        </p>

        <div id="telegram-section"
             style="margin-bottom:22px;">
            <button id="telegram-login-button"
                    type="button"
                    class="ui-button"
                    style="
                        width:100%;
                        min-height:52px;
                        padding:0 18px;
                        border:1px solid rgba(88,166,255,.38);
                        border-radius:14px;
                        color:#f5fbff;
                        background:
                            linear-gradient(
                                135deg,
                                rgba(42,140,255,.92),
                                rgba(26,102,210,.96)
                            );
                        box-shadow:
                            0 14px 34px rgba(42,140,255,.18),
                            inset 0 1px 0 rgba(255,255,255,.18);
                        font-size:16px;
                        font-weight:850;
                        cursor:pointer;
                    ">
                Войти через Telegram
            </button>

            <p id="telegram-status"
               class="ui-status"
               role="status"
               aria-live="polite"
               style="
                   min-height:22px;
                   margin:12px 0 0;
                   color:var(--muted);
                   font-size:14px;
                   line-height:1.5;
               ">
            </p>
        </div>

        <div style="
            display:flex;
            align-items:center;
            gap:12px;
            margin:0 0 22px;
            color:rgba(255,255,255,.34);
            font-size:12px;
            font-weight:700;
            letter-spacing:.12em;
            text-transform:uppercase;
        ">
            <span style="
                flex:1;
                height:1px;
                background:rgba(255,255,255,.08);
            "></span>
            или email
            <span style="
                flex:1;
                height:1px;
                background:rgba(255,255,255,.08);
            "></span>
        </div>

        <label for="email-input"
               style="
                   display:block;
                   margin-bottom:8px;
                   color:var(--text);
                   font-size:14px;
                   font-weight:750;
               ">
            Email
        </label>

        <input id="email-input"
               class="ui-input"
               name="email"
               type="email"
               autocomplete="email"
               inputmode="email"
               maxlength="254"
               placeholder="name@example.com"
               style="
                   width:100%;
                   min-height:52px;
                   padding:0 15px;
                   border:1px solid rgba(255,255,255,.13);
                   border-radius:14px;
                   color:var(--text);
                   background:rgba(6,8,11,.72);
                   box-shadow:
                       inset 0 1px 0 rgba(255,255,255,.025),
                       0 8px 24px rgba(0,0,0,.16);
                   font-size:16px;
                   caret-color:var(--gold-light);
               ">

        <button id="email-request-button"
                type="button"
                class="ui-button"
                style="
                    width:100%;
                    min-height:52px;
                    margin-top:12px;
                    padding:0 18px;
                    border:1px solid rgba(240,214,156,.42);
                    border-radius:14px;
                    color:#17130c;
                    background:
                        linear-gradient(
                            135deg,
                            var(--gold-light),
                            var(--gold)
                        );
                    box-shadow:
                        0 14px 34px rgba(214,183,122,.18),
                        inset 0 1px 0 rgba(255,255,255,.32);
                    font-size:16px;
                    font-weight:850;
                    cursor:pointer;
                ">
            Получить код
        </button>

        <div id="code-section"
             hidden
             style="
                 margin-top:22px;
                 padding-top:21px;
                 border-top:1px solid rgba(255,255,255,.08);
             ">
            <label for="code-input"
                   style="
                       display:block;
                       margin-bottom:8px;
                       color:var(--text);
                       font-size:14px;
                       font-weight:750;
                   ">
                Код из письма
            </label>

            <input id="code-input"
                   class="ui-input"
                   name="code"
                   type="text"
                   inputmode="numeric"
                   autocomplete="one-time-code"
                   pattern="[0-9]{6}"
                   minlength="6"
                   maxlength="6"
                   placeholder="000000"
                   style="
                       width:100%;
                       min-height:56px;
                       padding:0 15px;
                       border:1px solid rgba(255,255,255,.13);
                       border-radius:14px;
                       color:var(--text);
                       background:rgba(6,8,11,.72);
                       box-shadow:
                           inset 0 1px 0 rgba(255,255,255,.025),
                           0 8px 24px rgba(0,0,0,.16);
                       font-size:22px;
                       font-weight:750;
                       letter-spacing:.24em;
                       text-align:center;
                       caret-color:var(--gold-light);
                   ">

            <button id="email-verify-button"
                    type="button"
                    class="ui-button"
                    style="
                        width:100%;
                        min-height:52px;
                        margin-top:12px;
                        padding:0 18px;
                        border:1px solid rgba(240,214,156,.42);
                        border-radius:14px;
                        color:#17130c;
                        background:
                            linear-gradient(
                                135deg,
                                var(--gold-light),
                                var(--gold)
                            );
                        box-shadow:
                            0 14px 34px rgba(214,183,122,.18),
                            inset 0 1px 0 rgba(255,255,255,.32);
                        font-size:16px;
                        font-weight:850;
                        cursor:pointer;
                    ">
                Подтвердить и войти
            </button>
        </div>

        <p id="email-status"
           class="ui-status"
           role="status"
           aria-live="polite"
           style="
               min-height:22px;
               margin:15px 0 0;
               color:var(--muted);
               font-size:14px;
               line-height:1.5;
           ">
        </p>

        <p style="
            margin:18px 0 0;
            color:rgba(255,255,255,.40);
            font-size:12px;
            line-height:1.5;
            text-align:center;
        ">
            Код действует 10 минут
        </p>
    </section>

    <div style="
        margin-top:18px;
        text-align:center;
    ">
        <a href="/app"
           style="
               display:inline-block;
               padding:10px 14px;
               color:var(--muted);
               text-decoration:none;
               font-size:14px;
           ">
            ← Вернуться на карту
        </a>
    </div>
</div>
"####;

    let body_after = r####"
<script>
(function () {
    function authRedirectTarget() {
        const params = new URLSearchParams(window.location.search);
        const next = (params.get("next") || "").trim();

        if (
            next.startsWith("/app")
            && !next.startsWith("//")
            && !next.includes("://")
        ) {
            return next;
        }

        try {
            const referrer = new URL(document.referrer);

            if (
                referrer.origin === window.location.origin
                && referrer.pathname.startsWith("/app")
                && referrer.pathname !== "/app/auth"
            ) {
                return referrer.pathname + referrer.search;
            }
        } catch (_) {}

        return "/app";
    }

    const redirectTarget = authRedirectTarget();

    const emailInput =
        document.getElementById("email-input");

    const codeInput =
        document.getElementById("code-input");

    const codeSection =
        document.getElementById("code-section");

    const requestButton =
        document.getElementById("email-request-button");

    const verifyButton =
        document.getElementById("email-verify-button");

    const emailStatus =
        document.getElementById("email-status");

    const telegramButton =
        document.getElementById("telegram-login-button");

    const telegramStatus =
        document.getElementById("telegram-status");

    const telegramSection =
        document.getElementById("telegram-section");

    function setEmailStatus(message, isError) {
        emailStatus.textContent = message;
        emailStatus.style.color =
            isError ? "#ef6b72" : "var(--muted)";
    }

    function setTelegramStatus(message, isError) {
        telegramStatus.textContent = message;
        telegramStatus.style.color =
            isError ? "#ef6b72" : "var(--muted)";
    }

    function emailErrorMessage(error) {
        const messages = {
            invalid_email: "Проверьте правильность email.",
            rate_limited: "Слишком много попыток. Попробуйте позже.",
            mail_unavailable: "Отправка писем временно недоступна.",
            database_unavailable: "Сервис временно недоступен.",
            code_store_failed: "Не удалось сохранить код.",
            invalid_code: "Введите шестизначный код.",
            code_not_found: "Сначала запросите новый код.",
            code_used: "Этот код уже использован.",
            code_already_used: "Этот код уже использован.",
            code_expired: "Срок действия кода истёк.",
            wrong_code: "Код введён неверно.",
            too_many_attempts: "Слишком много неверных попыток.",
            user_create_failed: "Не удалось создать аккаунт.",
            identity_create_failed: "Не удалось создать способ входа.",
            profile_create_failed: "Не удалось создать профиль.",
            profile_update_failed: "Не удалось обновить профиль.",
            transaction_failed: "Не удалось начать операцию.",
            commit_failed: "Не удалось завершить вход.",
            invalid_telegram_data: "Не удалось проверить Telegram."
        };

        return messages[error] || "Не удалось выполнить запрос.";
    }

    function telegramErrorMessage(error) {
        const messages = {
            invalid_telegram_data: "Не удалось проверить Telegram.",
            rate_limited: "Слишком много попыток. Подождите немного.",
            database_unavailable: "Сервис временно недоступен.",
            user_create_failed: "Не удалось создать аккаунт.",
            identity_create_failed: "Не удалось создать способ входа.",
            profile_create_failed: "Не удалось создать профиль.",
            profile_update_failed: "Не удалось обновить профиль.",
            transaction_failed: "Не удалось начать операцию.",
            commit_failed: "Не удалось завершить вход."
        };

        return messages[error] || emailErrorMessage(error);
    }

    async function readJson(response) {
        try {
            return await response.json();
        } catch (_) {
            return {
                ok: false,
                error: "invalid_response"
            };
        }
    }

    function getTelegramWebApp() {
        return window.Telegram && window.Telegram.WebApp
            ? window.Telegram.WebApp
            : null;
    }

    async function loginWithTelegram(options) {
        const auto = options && options.auto;

        const tg = getTelegramWebApp();

        if (!tg || !tg.initData) {
            if (!auto) {
                setTelegramStatus(
                    "Откройте ResursMap через Telegram Mini App или используйте email.",
                    true
                );
            }
            return false;
        }

        tg.ready();

        if (telegramButton) {
            telegramButton.disabled = true;
        }

        setTelegramStatus(
            auto ? "Проверяем Telegram…" : "Входим…",
            false
        );

        try {
            const response = await fetch("/app/auth", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    init_data: tg.initData
                })
            });

            const data = await readJson(response);

            if (!response.ok || !data.ok) {
                setTelegramStatus(
                    telegramErrorMessage(data.error),
                    true
                );
                return false;
            }

            setTelegramStatus("✓ Вход выполнен", false);
            window.location.replace(redirectTarget);
            return true;
        } catch (_) {
            setTelegramStatus(
                "Ошибка соединения. Попробуйте ещё раз.",
                true
            );
            return false;
        } finally {
            if (telegramButton) {
                telegramButton.disabled = false;
            }
        }
    }

    async function requestEmailCode() {
        const email = emailInput.value.trim();

        if (!email) {
            setEmailStatus("Введите email.", true);
            emailInput.focus();
            return;
        }

        requestButton.disabled = true;
        setEmailStatus("Отправляем код…", false);

        try {
            const response = await fetch(
                "/app/auth/email/request",
                {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({ email })
                }
            );

            const data = await readJson(response);

            if (!response.ok || !data.ok) {
                setEmailStatus(
                    emailErrorMessage(data.error),
                    true
                );
                return;
            }

            codeSection.hidden = false;
            setEmailStatus(
                "Код отправлен. Проверьте входящие и папку «Спам».",
                false
            );
            codeInput.focus();
        } catch (_) {
            setEmailStatus(
                "Ошибка соединения. Попробуйте ещё раз.",
                true
            );
        } finally {
            requestButton.disabled = false;
        }
    }

    async function verifyEmailCode() {
        const email = emailInput.value.trim();
        const code = codeInput.value.trim();

        if (!email) {
            setEmailStatus("Введите email.", true);
            emailInput.focus();
            return;
        }

        if (!/^[0-9]{6}$/.test(code)) {
            setEmailStatus(
                "Введите шестизначный код.",
                true
            );
            codeInput.focus();
            return;
        }

        verifyButton.disabled = true;
        setEmailStatus("Проверяем код…", false);

        try {
            const response = await fetch(
                "/app/auth/email/verify",
                {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({ email, code })
                }
            );

            const data = await readJson(response);

            if (!response.ok || !data.ok) {
                setEmailStatus(
                    emailErrorMessage(data.error),
                    true
                );
                return;
            }

            setEmailStatus("✓ Вход выполнен", false);
            window.location.replace(redirectTarget);
        } catch (_) {
            setEmailStatus(
                "Ошибка соединения. Попробуйте ещё раз.",
                true
            );
        } finally {
            verifyButton.disabled = false;
        }
    }

    requestButton.addEventListener(
        "click",
        requestEmailCode
    );

    verifyButton.addEventListener(
        "click",
        verifyEmailCode
    );

    codeInput.addEventListener("input", function () {
        codeInput.value =
            codeInput.value.replace(/[^0-9]/g, "").slice(0, 6);
    });

    codeInput.addEventListener("keydown", function (event) {
        if (event.key === "Enter") {
            verifyEmailCode();
        }
    });

    emailInput.addEventListener("keydown", function (event) {
        if (event.key === "Enter") {
            requestEmailCode();
        }
    });

    if (telegramButton) {
        telegramButton.addEventListener(
            "click",
            function () {
                loginWithTelegram({ auto: false });
            }
        );
    }

    (async function () {
        const tg = getTelegramWebApp();

        if (tg && tg.initData) {
            await loginWithTelegram({ auto: true });
            return;
        }

        if (telegramSection) {
            setTelegramStatus(
                "Telegram-вход доступен в Mini App. Ниже — вход по email.",
                false
            );
        }

        emailInput.focus();
    })();
})();
</script>
"####;

    let head_extra = r####"<script src="https://telegram.org/js/telegram-web-app.js"></script>"####;

    Html(crate::web::templates::page_document(
        "Вход · ResursMap",
        head_extra,
        "",
        main_html,
        "",
        body_after,
    ))
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

    if let Ok(value) = HeaderValue::from_str(cookie) {
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

    let account_user_id = match provision_telegram_account(
        &db,
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
