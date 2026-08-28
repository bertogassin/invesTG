use super::admin_access::{
    load_admin_context, record_denied_access, valid_admin_session_public_id, AdminPermission,
};
use super::auth::verify_authenticated_user;
use super::common::request_is_cross_site;
use crate::state::app_state::AppState;
use crate::web::templates::{render_admin_security, AdminSecurityData};
use axum::{
    extract::{Form, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const CODE_TTL_SECONDS: i64 = 600;
const MAX_ATTEMPTS: i64 = 5;
const STEP_UP_VALID_SECONDS: i64 = 900;

static CHALLENGE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Deserialize)]
pub struct VerifyStepUpForm {
    code: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct SecurityQuery {
    sent: Option<i64>,
    verified: Option<i64>,
    error: Option<i64>,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn context_and_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(super::admin_access::AdminContext, String), Box<Response>> {
    let authenticated = verify_authenticated_user(state, headers).ok_or_else(|| {
        Box::new(
            (
                StatusCode::UNAUTHORIZED,
                "Требуется вход в аккаунт ResursMap",
            )
                .into_response(),
        )
    })?;

    let context = load_admin_context(state, authenticated.user_id).ok_or_else(|| {
        record_denied_access(
            state,
            authenticated.user_id,
            "admin_security_access_denied",
            "Нет активного административного назначения",
        );

        Box::new((StatusCode::NOT_FOUND, "404").into_response())
    })?;

    if !context.is_owner() || !context.has_permission(AdminPermission::GlobalSettingsManage) {
        record_denied_access(
            state,
            authenticated.user_id,
            "admin_security_permission_denied",
            "Недостаточно прав для Owner step-up",
        );

        return Err(Box::new(
            (StatusCode::FORBIDDEN, "Доступ запрещён").into_response(),
        ));
    }

    let session_public_id =
        valid_admin_session_public_id(state, headers, context.user_id, context.assignment_id)
            .ok_or_else(|| {
                let mut response = StatusCode::SEE_OTHER.into_response();

                response
                    .headers_mut()
                    .insert(header::LOCATION, HeaderValue::from_static("/app/center"));

                Box::new(response)
            })?;

    Ok((context, session_public_id))
}

fn owner_email(state: &AppState, user_id: i64) -> Option<String> {
    state
        .db_pool
        .get()
        .ok()?
        .query_row(
            "SELECT email
         FROM auth_identities
         WHERE user_id = ?1
           AND provider = 'email'
           AND email <> ''
           AND verified_at > 0
         ORDER BY id
         LIMIT 1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .ok()
}

fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return "Скрытый адрес".to_string();
    };

    let first = local.chars().next().unwrap_or('*');

    format!("{first}***@{domain}")
}

fn generate_code(state: &AppState, user_id: i64, session_public_id: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let sequence = CHALLENGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);

    let payload = format!(
        "step-up:{user_id}:{session_public_id}:{}:{nanos}:{sequence}",
        unix_now()
    );

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(payload.as_bytes());

    let digest = mac.finalize().into_bytes();

    let value = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]) % 1_000_000;

    format!("{value:06}")
}

fn code_hash(
    state: &AppState,
    user_id: i64,
    session_public_id: &str,
    code: &str,
    expires_at: i64,
) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let payload = format!("owner-step-up:{user_id}:{session_public_id}:{code}:{expires_at}");

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(payload.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}

fn destination_hash(state: &AppState, email: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");

    mac.update(b"owner-step-up-email:");
    mac.update(email.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}

async fn send_code(email: &str, code: &str) -> Result<(), String> {
    let api_key = std::env::var("RESEND_API_KEY").map_err(|_| "missing_resend_key".to_string())?;

    let from = std::env::var("RESURSMAP_MAIL_FROM")
        .unwrap_or_else(|_| "ResursMap <noreply@resursmap.de>".to_string());

    let response = reqwest::Client::new()
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "from": from,
            "to": [email],
            "subject": "Защищённая Owner-сессия ResursMap",
            "html": format!(
                "<div style=\"font-family:Arial,sans-serif;max-width:520px;margin:auto;padding:32px\">\
                 <h2>ResursMap · Global Owner</h2>\
                 <p>Код подтверждения административной сессии:</p>\
                 <div style=\"font-size:34px;font-weight:800;letter-spacing:8px;margin:24px 0\">{code}</div>\
                 <p>Код действует 10 минут и только в текущей сессии.</p>\
                 <p style=\"color:#777;font-size:13px\">Если вы не запрашивали код, проверьте безопасность аккаунта.</p>\
                 </div>"
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

fn secure_response(html: String) -> Response {
    let mut response = Html(html).into_response();

    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );
    response
        .headers_mut()
        .insert("x-frame-options", HeaderValue::from_static("DENY"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             style-src 'self' 'unsafe-inline'; \
             script-src 'none'; \
             img-src 'self' data:; \
             connect-src 'self'; \
             object-src 'none'; \
             base-uri 'none'; \
             frame-ancestors 'none'; \
             form-action 'self'",
        ),
    );

    response
}

pub async fn admin_security_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SecurityQuery>,
) -> Response {
    let (context, session_public_id) = match context_and_session(&state, &headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };

    let email = match owner_email(&state, context.user_id) {
        Some(email) => email,
        None => {
            return (
                StatusCode::PRECONDITION_FAILED,
                "У Owner отсутствует подтверждённый email",
            )
                .into_response();
        }
    };

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let state_row: (i64, Option<i64>) = connection
        .query_row(
            "SELECT
                two_factor_verified,
                reauthenticated_at
             FROM admin_sessions
             WHERE session_public_id = ?1
               AND user_id = ?2
               AND assignment_id = ?3",
            rusqlite::params![session_public_id, context.user_id, context.assignment_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, None));

    let now = unix_now();

    let remaining_seconds = state_row
        .1
        .map(|verified_at| (verified_at + STEP_UP_VALID_SECONDS - now).max(0))
        .unwrap_or(0);

    let verified = state_row.0 == 1 && remaining_seconds > 0;

    let message = if query.sent == Some(1) {
        "Код отправлен. Проверьте почту."
    } else if query.verified == Some(1) {
        "Owner-сессия успешно подтверждена."
    } else if query.error == Some(1) {
        "Код неверный, использован или истёк."
    } else {
        ""
    };

    drop(connection);

    secure_response(render_admin_security(AdminSecurityData {
        masked_email: mask_email(&email),
        verified,
        remaining_seconds,
        message: message.to_string(),
    }))
}

pub async fn admin_step_up_request(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён CSRF-защитой").into_response();
    }

    let (context, session_public_id) = match context_and_session(&state, &headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };

    let email = match owner_email(&state, context.user_id) {
        Some(email) => email,
        None => {
            return (
                StatusCode::PRECONDITION_FAILED,
                "Подтверждённый email не найден",
            )
                .into_response();
        }
    };

    let now = unix_now();

    let recent: i64 = state
        .db_pool
        .get()
        .ok()
        .and_then(|connection| {
            connection
                .query_row(
                    "SELECT COUNT(*)
                     FROM admin_reauth_challenges
                     WHERE user_id = ?1
                       AND created_at > ?2",
                    rusqlite::params![context.user_id, now - 600],
                    |row| row.get(0),
                )
                .ok()
        })
        .unwrap_or(100);

    if recent >= 5 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Слишком много запросов. Повторите позже.",
        )
            .into_response();
    }

    let code = generate_code(&state, context.user_id, &session_public_id);
    let expires_at = now + CODE_TTL_SECONDS;

    if let Err(error) = send_code(&email, &code).await {
        eprintln!("admin step-up email failed: {error}");

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Почтовый сервис временно недоступен",
        )
            .into_response();
    }

    let stored_hash = code_hash(
        &state,
        context.user_id,
        &session_public_id,
        &code,
        expires_at,
    );

    let seed = format!(
        "{}:{}:{}:{}",
        context.user_id, session_public_id, expires_at, stored_hash
    );

    let public_id = hex::encode(Sha256::digest(seed.as_bytes()))[..32].to_string();

    let email_hash = destination_hash(&state, &email);

    let mut connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let transaction = match connection.transaction() {
        Ok(transaction) => transaction,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось открыть транзакцию",
            )
                .into_response();
        }
    };

    let _ = transaction.execute(
        "UPDATE admin_reauth_challenges
         SET consumed_at = ?2
         WHERE session_public_id = ?1
           AND purpose = 'owner_step_up'
           AND consumed_at = 0",
        rusqlite::params![session_public_id, now],
    );

    let inserted = transaction.execute(
        "INSERT INTO admin_reauth_challenges (
            challenge_public_id,
            user_id,
            assignment_id,
            session_public_id,
            purpose,
            delivery_channel,
            destination_hash,
            code_hash,
            attempts,
            expires_at,
            consumed_at,
            created_at
         )
         VALUES (
            ?1, ?2, ?3, ?4,
            'owner_step_up',
            'email',
            ?5, ?6, 0, ?7, 0, ?8
         )",
        rusqlite::params![
            public_id,
            context.user_id,
            context.assignment_id,
            session_public_id,
            email_hash,
            stored_hash,
            expires_at,
            now
        ],
    );

    if inserted.is_err() || transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось сохранить защищённый код",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/security?sent=1")],
    )
        .into_response()
}

pub async fn admin_step_up_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<VerifyStepUpForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён CSRF-защитой").into_response();
    }

    let (context, session_public_id) = match context_and_session(&state, &headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };

    let code = payload.code.trim();

    if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return (
            StatusCode::SEE_OTHER,
            [(header::LOCATION, "/app/center/security?error=1")],
        )
            .into_response();
    }

    let now = unix_now();

    let mut connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let challenge: Option<(i64, String, i64, i64, i64)> = connection
        .query_row(
            "SELECT
                    id,
                    code_hash,
                    expires_at,
                    attempts,
                    consumed_at
                 FROM admin_reauth_challenges
                 WHERE user_id = ?1
                   AND assignment_id = ?2
                   AND session_public_id = ?3
                   AND purpose = 'owner_step_up'
                 ORDER BY id DESC
                 LIMIT 1",
            rusqlite::params![context.user_id, context.assignment_id, session_public_id],
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

    let Some((challenge_id, expected_hash, expires_at, attempts, consumed_at)) = challenge else {
        return redirect_error();
    };

    if consumed_at != 0 || expires_at <= now || attempts >= MAX_ATTEMPTS {
        return redirect_error();
    }

    let actual_hash = code_hash(
        &state,
        context.user_id,
        &session_public_id,
        code,
        expires_at,
    );

    if actual_hash != expected_hash {
        let _ = connection.execute(
            "UPDATE admin_reauth_challenges
             SET attempts = attempts + 1
             WHERE id = ?1
               AND attempts < ?2",
            rusqlite::params![challenge_id, MAX_ATTEMPTS],
        );

        let _ = connection.execute(
            "INSERT INTO admin_security_events (
                user_id,
                assignment_id,
                session_public_id,
                event_type,
                severity,
                details
             )
             VALUES (
                ?1, ?2, ?3,
                'owner_step_up_failed',
                'high',
                'Неверный код подтверждения'
             )",
            rusqlite::params![context.user_id, context.assignment_id, session_public_id],
        );

        return redirect_error();
    }

    let transaction = match connection.transaction() {
        Ok(transaction) => transaction,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось открыть транзакцию",
            )
                .into_response();
        }
    };

    let consumed = transaction
        .execute(
            "UPDATE admin_reauth_challenges
             SET consumed_at = ?2
             WHERE id = ?1
               AND consumed_at = 0
               AND expires_at > ?2",
            rusqlite::params![challenge_id, now],
        )
        .unwrap_or(0);

    let upgraded = transaction
        .execute(
            "UPDATE admin_sessions
             SET two_factor_verified = 1,
                 reauthenticated_at = ?2
             WHERE session_public_id = ?1
               AND user_id = ?3
               AND assignment_id = ?4
               AND revoked_at IS NULL
               AND expires_at > ?2",
            rusqlite::params![
                session_public_id,
                now,
                context.user_id,
                context.assignment_id
            ],
        )
        .unwrap_or(0);

    if consumed != 1 || upgraded != 1 {
        let _ = transaction.rollback();

        return (StatusCode::CONFLICT, "Состояние сессии изменилось").into_response();
    }

    let _ = transaction.execute(
        "INSERT INTO admin_security_events (
            user_id,
            assignment_id,
            session_public_id,
            event_type,
            severity,
            details
         )
         VALUES (
            ?1, ?2, ?3,
            'owner_step_up_verified',
            'info',
            'Email step-up подтверждён'
         )",
        rusqlite::params![context.user_id, context.assignment_id, session_public_id],
    );

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось подтвердить сессию",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/security?verified=1")],
    )
        .into_response()
}

fn redirect_error() -> Response {
    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/security?error=1")],
    )
        .into_response()
}
