use super::auth::{
    append_user_session_cookie, auth_redirect_target, create_user_session, email_rate_limit_id,
    ensure_profile_public_id, normalize_email, EMAIL_USER_ID_BASE,
};
use super::common::{csrf_rejected_response, rate_limit_retry_after, request_is_cross_site, unix_now};
use crate::state::app_state::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct EmailPasswordRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthNextQuery {
    pub next: Option<String>,
}

fn validate_password(password: &str) -> Result<(), &'static str> {
    let password = password.trim();

    if password.len() < 8 {
        return Err("password_too_short");
    }

    if password.len() > 128 {
        return Err("password_too_long");
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<String, &'static str> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "password_hash_failed")
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    if password_hash.is_empty() {
        return false;
    }

    let parsed = match PasswordHash::new(password_hash) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn allocate_email_user_id(transaction: &rusqlite::Transaction<'_>) -> i64 {
    transaction
        .query_row(
            "SELECT COALESCE(MAX(id), ?1 - 1) + 1
             FROM users
             WHERE id >= ?1",
            rusqlite::params![EMAIL_USER_ID_BASE],
            |row| row.get(0),
        )
        .unwrap_or(EMAIL_USER_ID_BASE)
}

fn provision_email_account(
    transaction: &rusqlite::Transaction<'_>,
    email: &str,
    password_hash: &str,
) -> Result<i64, &'static str> {
    let existing_user_id: Option<i64> = transaction
        .query_row(
            "SELECT user_id
             FROM auth_identities
             WHERE provider = 'email'
               AND email = ?1
             LIMIT 1",
            rusqlite::params![email],
            |row| row.get(0),
        )
        .ok();

    if existing_user_id.is_some() {
        return Err("email_already_registered");
    }

    let now = unix_now();
    let next_id = allocate_email_user_id(transaction);

    transaction
        .execute(
            "INSERT INTO users (
                id,
                created_at,
                updated_at,
                is_active,
                telegram_id
             )
             VALUES (?1, ?2, ?2, 1, NULL)",
            rusqlite::params![next_id, now],
        )
        .map_err(|_| "user_create_failed")?;

    transaction
        .execute(
            "INSERT INTO auth_identities (
                user_id,
                provider,
                provider_subject,
                email,
                password_hash,
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
                ?4,
                ?4,
                ?4
             )",
            rusqlite::params![next_id, email, password_hash, now],
        )
        .map_err(|_| "identity_create_failed")?;

    let client_id = format!("user:{next_id}");

    transaction
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
            rusqlite::params![&client_id, next_id, now],
        )
        .map_err(|_| "profile_create_failed")?;

    ensure_profile_public_id(transaction, next_id)?;

    Ok(next_id)
}

fn email_password_auth_response(
    state: &AppState,
    user_id: i64,
    headers: &HeaderMap,
) -> Response {
    let session = match create_user_session(state, user_id, headers) {
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

pub async fn register_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailPasswordRequest>,
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

    if let Err(error) = validate_password(&payload.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error
            })),
        )
            .into_response();
    }

    let rate_id = email_rate_limit_id(&state, &email);

    if let Some(retry_after) =
        rate_limit_retry_after(&state, rate_id, "email_register", 8, 600).await
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

    let password_hash = match hash_password(payload.password.trim()) {
        Ok(password_hash) => password_hash,
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

    let user_id = match provision_email_account(&transaction, &email, &password_hash) {
        Ok(user_id) => user_id,
        Err(error) => {
            let status = if error == "email_already_registered" {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            return (
                status,
                Json(json!({
                    "ok": false,
                    "error": error
                })),
            )
                .into_response();
        }
    };

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

    email_password_auth_response(&state, user_id, &headers)
}

pub async fn login_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailPasswordRequest>,
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

    if payload.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_password"
            })),
        )
            .into_response();
    }

    let rate_id = email_rate_limit_id(&state, &email);

    if let Some(retry_after) =
        rate_limit_retry_after(&state, rate_id, "email_login", 20, 600).await
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

    let row: Option<(i64, String)> = db
        .query_row(
            "SELECT user_id, password_hash
             FROM auth_identities
             WHERE provider = 'email'
               AND email = ?1
             LIMIT 1",
            rusqlite::params![&email],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let (user_id, password_hash) = match row {
        Some(row) => row,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "invalid_credentials"
                })),
            )
                .into_response();
        }
    };

    if !verify_password(payload.password.trim(), &password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "invalid_credentials"
            })),
        )
            .into_response();
    }

    email_password_auth_response(&state, user_id, &headers)
}

pub async fn app_auth_page(Query(query): Query<AuthNextQuery>) -> Redirect {
    let target = auth_redirect_target(query.next.as_deref());

    if target == "/app" {
        Redirect::temporary("/login")
    } else {
        Redirect::temporary(&format!(
            "/login?next={}",
            urlencoding::encode(&target)
        ))
    }
}

pub async fn login_page(Query(query): Query<AuthNextQuery>) -> Html<String> {
    let redirect_target = auth_redirect_target(query.next.as_deref());
    let register_href = if redirect_target == "/app" {
        "/register".to_string()
    } else {
        format!(
            "/register?next={}",
            urlencoding::encode(&redirect_target)
        )
    };

    let main_html = format!(
        r##"
<div style="width:min(100%,520px);margin:0 auto;padding:18px;">
    <section class="card" style="display:block;padding:26px 22px;border-color:rgba(214,183,122,.24);">
        <h1 style="margin:0 0 10px;color:var(--text);font-size:clamp(30px,8vw,42px);">Вход</h1>
        <p style="margin:0 0 24px;color:var(--muted);font-size:16px;line-height:1.58;">
            Карта и поиск работают без регистрации. Вход нужен для сообщений, избранного и публикаций.
        </p>

        <label for="email-input" style="display:block;margin-bottom:8px;color:var(--text);font-size:14px;font-weight:750;">Email</label>
        <input id="email-input" class="ui-input" type="email" autocomplete="email" maxlength="254" placeholder="name@example.com" style="width:100%;min-height:52px;padding:0 15px;border-radius:14px;">

        <label for="password-input" style="display:block;margin:16px 0 8px;color:var(--text);font-size:14px;font-weight:750;">Пароль</label>
        <input id="password-input" class="ui-input" type="password" autocomplete="current-password" maxlength="128" placeholder="********" style="width:100%;min-height:52px;padding:0 15px;border-radius:14px;">

        <button id="login-button" type="button" class="ui-button" style="width:100%;min-height:52px;margin-top:16px;border-radius:14px;font-size:16px;font-weight:850;cursor:pointer;">
            Войти
        </button>

        <p id="auth-status" role="status" aria-live="polite" style="min-height:22px;margin:15px 0 0;color:var(--muted);font-size:14px;"></p>

        <p style="margin:18px 0 0;text-align:center;color:var(--muted);font-size:14px;">
            Нет аккаунта? <a href="{register_href}" style="color:var(--gold-light);">Зарегистрироваться</a>
        </p>
    </section>

    <div style="margin-top:18px;text-align:center;">
        <a href="/app" style="color:var(--muted);text-decoration:none;font-size:14px;">&larr; Вернуться на карту</a>
    </div>
</div>
"##
    );

    let body_after = format!(
        r##"
<script>
(function () {{
    const redirectTarget = {redirect_target_json};
    const emailInput = document.getElementById("email-input");
    const passwordInput = document.getElementById("password-input");
    const loginButton = document.getElementById("login-button");
    const authStatus = document.getElementById("auth-status");

    function setStatus(message, isError) {{
        authStatus.textContent = message;
        authStatus.style.color = isError ? "#ef6b72" : "var(--muted)";
    }}

    function errorMessage(error) {{
        const messages = {{
            invalid_email: "Проверьте правильность email.",
            invalid_password: "Введите пароль.",
            invalid_credentials: "Неверный email или пароль.",
            rate_limited: "Слишком много попыток. Попробуйте позже.",
            database_unavailable: "Сервис временно недоступен."
        }};
        return messages[error] || "Не удалось выполнить вход.";
    }}

    async function login() {{
        const email = emailInput.value.trim();
        const password = passwordInput.value;

        if (!email) {{
            setStatus("Введите email.", true);
            emailInput.focus();
            return;
        }}

        if (!password) {{
            setStatus("Введите пароль.", true);
            passwordInput.focus();
            return;
        }}

        loginButton.disabled = true;
        setStatus("Входим...", false);

        try {{
            const response = await fetch("/auth/login-email", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{ email, password }})
            }});

            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(errorMessage(data.error), true);
                return;
            }}

            setStatus("Вход выполнен", false);
            window.location.replace(redirectTarget);
        }} catch (_) {{
            setStatus("Ошибка соединения. Попробуйте ещё раз.", true);
        }} finally {{
            loginButton.disabled = false;
        }}
    }}

    loginButton.addEventListener("click", login);
    passwordInput.addEventListener("keydown", function (event) {{
        if (event.key === "Enter") login();
    }});
    emailInput.focus();
}})();
</script>
"##,
        redirect_target_json = serde_json::to_string(&redirect_target).unwrap_or_else(|_| "\"/app\"".to_string()),
    );

    Html(crate::web::templates::page_document(
        "Вход · ResursMap",
        "",
        "",
        &main_html,
        "",
        &body_after,
    ))
}

pub async fn register_page(Query(query): Query<AuthNextQuery>) -> Html<String> {
    let redirect_target = auth_redirect_target(query.next.as_deref());
    let login_href = if redirect_target == "/app" {
        "/login".to_string()
    } else {
        format!("/login?next={}", urlencoding::encode(&redirect_target))
    };

    let main_html = format!(
        r##"
<div style="width:min(100%,520px);margin:0 auto;padding:18px;">
    <section class="card" style="display:block;padding:26px 22px;border-color:rgba(214,183,122,.24);">
        <h1 style="margin:0 0 10px;color:var(--text);font-size:clamp(30px,8vw,42px);">Регистрация</h1>
        <p style="margin:0 0 24px;color:var(--muted);font-size:16px;line-height:1.58;">
            Создайте аккаунт на сайте. Telegram для входа не требуется.
        </p>

        <label for="email-input" style="display:block;margin-bottom:8px;color:var(--text);font-size:14px;font-weight:750;">Email</label>
        <input id="email-input" class="ui-input" type="email" autocomplete="email" maxlength="254" placeholder="name@example.com" style="width:100%;min-height:52px;padding:0 15px;border-radius:14px;">

        <label for="password-input" style="display:block;margin:16px 0 8px;color:var(--text);font-size:14px;font-weight:750;">Пароль</label>
        <input id="password-input" class="ui-input" type="password" autocomplete="new-password" maxlength="128" placeholder="Минимум 8 символов" style="width:100%;min-height:52px;padding:0 15px;border-radius:14px;">

        <button id="register-button" type="button" class="ui-button" style="width:100%;min-height:52px;margin-top:16px;border-radius:14px;font-size:16px;font-weight:850;cursor:pointer;">
            Создать аккаунт
        </button>

        <p id="auth-status" role="status" aria-live="polite" style="min-height:22px;margin:15px 0 0;color:var(--muted);font-size:14px;"></p>

        <p style="margin:18px 0 0;text-align:center;color:var(--muted);font-size:14px;">
            Уже есть аккаунт? <a href="{login_href}" style="color:var(--gold-light);">Войти</a>
        </p>
    </section>

    <div style="margin-top:18px;text-align:center;">
        <a href="/app" style="color:var(--muted);text-decoration:none;font-size:14px;">&larr; Вернуться на карту</a>
    </div>
</div>
"##
    );

    let body_after = format!(
        r##"
<script>
(function () {{
    const redirectTarget = {redirect_target_json};
    const emailInput = document.getElementById("email-input");
    const passwordInput = document.getElementById("password-input");
    const registerButton = document.getElementById("register-button");
    const authStatus = document.getElementById("auth-status");

    function setStatus(message, isError) {{
        authStatus.textContent = message;
        authStatus.style.color = isError ? "#ef6b72" : "var(--muted)";
    }}

    function errorMessage(error) {{
        const messages = {{
            invalid_email: "Проверьте правильность email.",
            password_too_short: "Пароль должен быть не короче 8 символов.",
            password_too_long: "Пароль слишком длинный.",
            email_already_registered: "Этот email уже зарегистрирован.",
            rate_limited: "Слишком много попыток. Попробуйте позже.",
            database_unavailable: "Сервис временно недоступен."
        }};
        return messages[error] || "Не удалось зарегистрироваться.";
    }}

    async function register() {{
        const email = emailInput.value.trim();
        const password = passwordInput.value;

        if (!email) {{
            setStatus("Введите email.", true);
            emailInput.focus();
            return;
        }}

        if (password.length < 8) {{
            setStatus("Пароль должен быть не короче 8 символов.", true);
            passwordInput.focus();
            return;
        }}

        registerButton.disabled = true;
        setStatus("Создаём аккаунт...", false);

        try {{
            const response = await fetch("/auth/register-email", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{ email, password }})
            }});

            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(errorMessage(data.error), true);
                return;
            }}

            setStatus("Аккаунт создан", false);
            window.location.replace(redirectTarget);
        }} catch (_) {{
            setStatus("Ошибка соединения. Попробуйте ещё раз.", true);
        }} finally {{
            registerButton.disabled = false;
        }}
    }}

    registerButton.addEventListener("click", register);
    passwordInput.addEventListener("keydown", function (event) {{
        if (event.key === "Enter") register();
    }});
    emailInput.focus();
}})();
</script>
"##,
        redirect_target_json = serde_json::to_string(&redirect_target).unwrap_or_else(|_| "\"/app\"".to_string()),
    );

    Html(crate::web::templates::page_document(
        "Регистрация · ResursMap",
        "",
        "",
        &main_html,
        "",
        &body_after,
    ))
}
