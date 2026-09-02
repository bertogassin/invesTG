use super::auth::{
    append_user_session_cookie, auth_redirect_target, create_user_session, email_rate_limit_id,
    ensure_profile_public_id, normalize_email, EMAIL_USER_ID_BASE,
};
use super::common::{
    csrf_rejected_response, rate_limit_retry_after, request_is_cross_site, unix_now,
};
use crate::state::app_state::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
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
pub struct EmailRegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub password_confirm: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthNextQuery {
    pub next: Option<String>,
}

pub(super) fn auth_related_href(base: &str, redirect_target: &str) -> String {
    if redirect_target == "/app" {
        base.to_string()
    } else {
        format!("{base}?next={}", urlencoding::encode(redirect_target))
    }
}

pub(super) fn validate_password(password: &str) -> Result<(), &'static str> {
    let password = password.trim();

    if password.len() < 8 {
        return Err("password_too_short");
    }

    if password.len() > 128 {
        return Err("password_too_long");
    }

    Ok(())
}

pub(super) fn hash_password(password: &str) -> Result<String, &'static str> {
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
                ?5,
                ?5
             )",
            rusqlite::params![next_id, email, password_hash, 0, now],
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

pub(super) fn email_password_auth_response(
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
    Json(payload): Json<EmailRegisterRequest>,
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

    if payload.password.trim() != payload.password_confirm.trim() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "password_mismatch"
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

    drop(db);

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "user_id": user_id,
            "verification_required": true
        })),
    )
        .into_response()
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

    if let Some(retry_after) = rate_limit_retry_after(&state, rate_id, "email_login", 20, 600).await
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

    let row: Option<(i64, String, i64)> = db
        .query_row(
            "SELECT user_id, password_hash, verified_at
             FROM auth_identities
             WHERE provider = 'email'
               AND email = ?1
             LIMIT 1",
            rusqlite::params![&email],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let (user_id, password_hash, verified_at) = match row {
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

    if verified_at <= 0 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "ok": false,
                "error": "verification_required"
            })),
        )
            .into_response();
    }

    if !verify_password(payload.password.trim(), &password_hash) {
        let error = if password_hash.is_empty() {
            "password_not_set"
        } else {
            "invalid_credentials"
        };

        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": error
            })),
        )
            .into_response();
    }

    email_password_auth_response(&state, user_id, &headers)
}

pub async fn login_page(Query(query): Query<AuthNextQuery>) -> Html<String> {
    let redirect_target = auth_redirect_target(query.next.as_deref());
    let register_href = auth_related_href("/register", &redirect_target);
    let forgot_href = auth_related_href("/login/forgot", &redirect_target);
    let code_href = auth_related_href("/login/code", &redirect_target);

    let body_html = format!(
        r##"
        <label class="rm-auth-label" for="email-input">Email</label>
        <input id="email-input" class="ui-input rm-auth-input" type="email" autocomplete="email" maxlength="254" placeholder="name@example.com">

        <label class="rm-auth-label" for="password-input">Пароль</label>
        <div class="rm-auth-password-row">
            <input id="password-input" class="ui-input rm-auth-input" type="password" autocomplete="current-password" maxlength="128" placeholder="********">
            <button id="password-toggle" type="button" class="rm-auth-password-toggle" aria-label="Показать пароль">Показать</button>
        </div>

        <div class="rm-auth-links">
            <a href="{forgot_href}">Забыли пароль?</a>
            <a href="{code_href}">Войти по коду</a>
        </div>

        <button id="login-button" type="button" class="ui-button rm-auth-button">Войти</button>
"##,
        forgot_href = forgot_href,
        code_href = code_href,
    );

    let footer_html = format!(
        r##"<p class="rm-auth-footer">Нет аккаунта? <a href="{register_href}">Зарегистрироваться</a></p>"##,
        register_href = register_href,
    );

    let body_after = format!(
        r##"
<script>
(function () {{
    const redirectTarget = {redirect_target_json};
    const emailInput = document.getElementById("email-input");
    const passwordInput = document.getElementById("password-input");
    const passwordToggle = document.getElementById("password-toggle");
    const loginButton = document.getElementById("login-button");
    const authStatus = document.getElementById("auth-status");

    function setStatus(message, isError) {{
        authStatus.textContent = message;
        authStatus.classList.toggle("is-error", isError);
    }}

    function errorMessage(error) {{
        const messages = {{
            invalid_email: "Проверьте правильность email.",
            invalid_password: "Введите пароль.",
            invalid_credentials: "Неверный email или пароль.",
            password_not_set: "Для этого email пароль ещё не задан. Используйте «Забыли пароль?» или вход по коду.",
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

    if (window.resursmapAuthForms) {{
        window.resursmapAuthForms.bindPasswordToggle(passwordToggle, passwordInput);
        window.resursmapAuthForms.bindEnterSubmit([emailInput, passwordInput], login);
    }}

    emailInput.focus();
}})();
</script>
"##,
        redirect_target_json =
            serde_json::to_string(&redirect_target).unwrap_or_else(|_| "\"/app\"".to_string()),
    );

    Html(crate::web::templates::render_auth_page(
        crate::web::templates::AuthPageParams {
            document_title: "Вход · ResursMap",
            heading: "Вход",
            subtitle:
                "Города и поиск работают без регистрации. Вход нужен для сообщений, избранного и публикаций.",
            body_html: &body_html,
            footer_html: &footer_html,
            script_html: &body_after,
        },
    ))
}

pub async fn register_page(Query(query): Query<AuthNextQuery>) -> Html<String> {
    let redirect_target = auth_redirect_target(query.next.as_deref());
    let login_href = auth_related_href("/login", &redirect_target);
    let forgot_href = auth_related_href("/login/forgot", &redirect_target);

    let body_html = r##"
        <label class="rm-auth-label" for="email-input">Email</label>
        <input id="email-input" class="ui-input rm-auth-input" type="email" autocomplete="email" maxlength="254" placeholder="name@example.com">

        <label class="rm-auth-label" for="password-input">Пароль</label>
        <div class="rm-auth-password-row">
            <input id="password-input" class="ui-input rm-auth-input" type="password" autocomplete="new-password" maxlength="128" placeholder="Минимум 8 символов">
            <button id="password-toggle" type="button" class="rm-auth-password-toggle" aria-label="Показать пароль">Показать</button>
        </div>

        <label class="rm-auth-label" for="password-confirm-input">Повторите пароль</label>
        <input id="password-confirm-input" class="ui-input rm-auth-input" type="password" autocomplete="new-password" maxlength="128" placeholder="Ещё раз">

        <button id="register-button" type="button" class="ui-button rm-auth-button">Создать аккаунт</button>
"##;

    let footer_html = format!(
        r##"<p class="rm-auth-footer">Уже есть аккаунт? <a href="{login_href}">Войти</a> · <a href="{forgot_href}">Забыли пароль?</a></p>"##,
        login_href = login_href,
        forgot_href = forgot_href,
    );

    let body_after = format!(
        r##"
<script>
(function () {{
    const redirectTarget = {redirect_target_json};
    const emailInput = document.getElementById("email-input");
    const passwordInput = document.getElementById("password-input");
    const passwordConfirmInput = document.getElementById("password-confirm-input");
    const passwordToggle = document.getElementById("password-toggle");
    const registerButton = document.getElementById("register-button");
    const authStatus = document.getElementById("auth-status");

    function setStatus(message, isError) {{
        authStatus.textContent = message;
        authStatus.classList.toggle("is-error", isError);
    }}

    function errorMessage(error) {{
        const messages = {{
            invalid_email: "Проверьте правильность email.",
            password_too_short: "Пароль должен быть не короче 8 символов.",
            password_too_long: "Пароль слишком длинный.",
            password_mismatch: "Пароли не совпадают.",
            email_already_registered: "Этот email уже зарегистрирован. Попробуйте войти или восстановить пароль.",
            verification_required: "Подтвердите email кодом из письма.",
            rate_limited: "Слишком много попыток. Попробуйте позже.",
            database_unavailable: "Сервис временно недоступен."
        }};
        return messages[error] || "Не удалось зарегистрироваться.";
    }}

    async function register() {{
        const email = emailInput.value.trim();
        const password = passwordInput.value;
        const passwordConfirm = passwordConfirmInput.value;

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

        if (password !== passwordConfirm) {{
            setStatus("Пароли не совпадают.", true);
            passwordConfirmInput.focus();
            return;
        }}

        registerButton.disabled = true;
        setStatus("Создаём аккаунт...", false);

        try {{
            const response = await fetch("/auth/register-email", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{
                    email,
                    password,
                    password_confirm: passwordConfirm
                }})
            }});

            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(errorMessage(data.error), true);
                return;
            }}

            if (data.verification_required) {{
                setStatus("Теперь подтвердите email кодом.", false);
                const next = encodeURIComponent(redirectTarget);
                window.location.replace("/login/code?next=" + next);
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

    if (window.resursmapAuthForms) {{
        window.resursmapAuthForms.bindPasswordToggle(passwordToggle, passwordInput);
        window.resursmapAuthForms.bindEnterSubmit(
            [emailInput, passwordInput, passwordConfirmInput],
            register
        );
    }}

    registerButton.addEventListener("click", register);
    emailInput.focus();
}})();
</script>
"##,
        redirect_target_json =
            serde_json::to_string(&redirect_target).unwrap_or_else(|_| "\"/app\"".to_string()),
    );

    Html(crate::web::templates::render_auth_page(
        crate::web::templates::AuthPageParams {
            document_title: "Регистрация · ResursMap",
            heading: "Регистрация",
            subtitle: "Создайте аккаунт по email и паролю. Telegram используется только для уведомлений и публикаций, не для входа.",
            body_html,
            footer_html: &footer_html,
            script_html: &body_after,
        },
    ))
}
