use super::auth::{auth_redirect_target, email_rate_limit_id, normalize_email};
use super::auth_email::{
    auth_related_href, email_password_auth_response, hash_password, validate_password,
    AuthNextQuery,
};
use super::common::{csrf_rejected_response, rate_limit_retry_after, request_is_cross_site, unix_now};
use crate::state::app_state::AppState;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;

#[derive(Debug, Deserialize)]
pub struct EmailResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailResetConfirmRequest {
    pub email: String,
    pub code: String,
    pub password: String,
}

const EMAIL_CODE_TTL_SECONDS: i64 = 600;
const EMAIL_CODE_MAX_ATTEMPTS: i64 = 5;

fn generate_email_code() -> String {
    let mut bytes = [0u8; 4];
    getrandom::getrandom(&mut bytes).expect("secure random");

    let value = u32::from_be_bytes(bytes) % 1_000_000;

    format!("{value:06}")
}

fn hash_reset_code(state: &AppState, email: &str, code: &str, expires_at: i64) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let payload = format!("password-reset-code:{email}:{code}:{expires_at}");

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes()).expect("HMAC key");
    mac.update(payload.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}

async fn send_reset_email(email: &str, code: &str) -> Result<(), String> {
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
            "subject": "Сброс пароля ResursMap",
            "html": format!(
                "<div style=\"font-family:Arial,sans-serif;max-width:520px;margin:auto;padding:32px\">\
                    <h2 style=\"margin:0 0 18px\">ResursMap</h2>\
                    <p>Код для сброса пароля:</p>\
                    <div style=\"font-size:34px;font-weight:800;letter-spacing:8px;margin:24px 0\">{}</div>\
                    <p>Код действует 10 минут.</p>\
                    <p style=\"color:#777;font-size:13px\">Если вы не запрашивали сброс, проигнорируйте письмо.</p>\
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

pub async fn forgot_password_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailResetRequest>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let email = match normalize_email(&payload.email) {
        Some(email) => email,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": "invalid_email" })),
            )
                .into_response();
        }
    };

    let rate_id = email_rate_limit_id(&state, &email);

    if let Some(retry_after) =
        rate_limit_retry_after(&state, rate_id, "password_reset_request", 5, 600).await
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
                Json(json!({ "ok": false, "error": "database_unavailable" })),
            )
                .into_response();
        }
    };

    let account_exists: bool = db
        .query_row(
            "SELECT 1
             FROM auth_identities
             WHERE provider = 'email'
               AND email = ?1
             LIMIT 1",
            rusqlite::params![&email],
            |_| Ok(()),
        )
        .is_ok();

    if account_exists {
        let code = generate_email_code();
        let expires_at = unix_now() + EMAIL_CODE_TTL_SECONDS;
        let code_hash = hash_reset_code(&state, &email, &code, expires_at);

        let _ = db.execute(
            "UPDATE email_login_codes
             SET consumed_at = ?2
             WHERE email = ?1
               AND consumed_at = 0",
            rusqlite::params![&email, unix_now()],
        );

        if db
            .execute(
                "INSERT INTO email_login_codes (
                    email,
                    code_hash,
                    expires_at,
                    attempts,
                    consumed_at,
                    created_at,
                    purpose
                 )
                 VALUES (?1, ?2, ?3, 0, 0, ?4, 'reset')",
                rusqlite::params![&email, &code_hash, expires_at, unix_now()],
            )
            .is_ok()
            && send_reset_email(&email, &code).await.is_err()
        {
            let _ = db.execute(
                "UPDATE email_login_codes
                 SET consumed_at = ?2
                 WHERE email = ?1
                   AND purpose = 'reset'
                   AND consumed_at = 0",
                rusqlite::params![&email, unix_now()],
            );

            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "ok": false, "error": "mail_unavailable" })),
            )
                .into_response();
        }
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

pub async fn reset_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailResetConfirmRequest>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let email = match normalize_email(&payload.email) {
        Some(email) => email,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": "invalid_email" })),
            )
                .into_response();
        }
    };

    if let Err(error) = validate_password(&payload.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": error })),
        )
            .into_response();
    }

    let code = payload.code.trim();

    if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "invalid_code" })),
        )
            .into_response();
    }

    let rate_id = email_rate_limit_id(&state, &email);

    if let Some(retry_after) =
        rate_limit_retry_after(&state, rate_id, "password_reset_confirm", 15, 600).await
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
                Json(json!({ "ok": false, "error": "database_unavailable" })),
            )
                .into_response();
        }
    };

    let row: Option<(i64, String, i64, i64, i64)> = db
        .query_row(
            "SELECT id, code_hash, expires_at, attempts, consumed_at
             FROM email_login_codes
             WHERE email = ?1
               AND purpose = 'reset'
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
                Json(json!({ "ok": false, "error": "code_not_found" })),
            )
                .into_response();
        }
    };

    if consumed_at != 0 {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "code_used" })),
        )
            .into_response();
    }

    if expires_at < unix_now() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "code_expired" })),
        )
            .into_response();
    }

    if attempts >= EMAIL_CODE_MAX_ATTEMPTS {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "ok": false, "error": "too_many_attempts" })),
        )
            .into_response();
    }

    if hash_reset_code(&state, &email, code, expires_at) != expected_hash {
        let _ = db.execute(
            "UPDATE email_login_codes SET attempts = attempts + 1 WHERE id = ?1",
            rusqlite::params![code_id],
        );

        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "wrong_code" })),
        )
            .into_response();
    }

    let user_id: Option<i64> = db
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

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "ok": false, "error": "account_not_found" })),
            )
                .into_response();
        }
    };

    let password_hash = match hash_password(payload.password.trim()) {
        Ok(password_hash) => password_hash,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": error })),
            )
                .into_response();
        }
    };

    let transaction = match db.transaction() {
        Ok(transaction) => transaction,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": "transaction_failed" })),
            )
                .into_response();
        }
    };

    if transaction
        .execute(
            "UPDATE auth_identities
             SET password_hash = ?2,
                 updated_at = ?3
             WHERE provider = 'email'
               AND email = ?1",
            rusqlite::params![&email, &password_hash, unix_now()],
        )
        .ok()
        != Some(1)
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": "password_update_failed" })),
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
            Json(json!({ "ok": false, "error": "code_already_used" })),
        )
            .into_response();
    }

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": "commit_failed" })),
        )
            .into_response();
    }

    email_password_auth_response(&state, user_id, &headers)
}

pub async fn login_code_page(Query(query): Query<AuthNextQuery>) -> Html<String> {
    let redirect_target = auth_redirect_target(query.next.as_deref());
    let login_href = auth_related_href("/login", &redirect_target);

    let body_html = r##"
        <label class="rm-auth-label" for="email-input">Email</label>
        <input id="email-input" class="ui-input rm-auth-input" type="email" autocomplete="email" maxlength="254" placeholder="name@example.com">

        <button id="request-button" type="button" class="ui-button rm-auth-button rm-auth-button--compact">Получить код</button>

        <div id="code-section" hidden class="rm-auth-step">
            <label class="rm-auth-label" for="code-input">Код из письма</label>
            <input id="code-input" class="ui-input rm-auth-input rm-auth-input--code" type="text" inputmode="numeric" autocomplete="one-time-code" maxlength="6" placeholder="000000">

            <button id="verify-button" type="button" class="ui-button rm-auth-button rm-auth-button--compact">Подтвердить и войти</button>
        </div>
"##;

    let footer_html = format!(
        r##"<p class="rm-auth-footer"><a href="{login_href}">Войти с паролем</a></p>"##,
        login_href = login_href,
    );

    let body_after = format!(
        r##"
<script>
(function () {{
    const redirectTarget = {redirect_target_json};
    const emailInput = document.getElementById("email-input");
    const codeInput = document.getElementById("code-input");
    const codeSection = document.getElementById("code-section");
    const requestButton = document.getElementById("request-button");
    const verifyButton = document.getElementById("verify-button");
    const authStatus = document.getElementById("auth-status");

    function setStatus(message, isError) {{
        authStatus.textContent = message;
        authStatus.classList.toggle("is-error", isError);
    }}

    function otpError(error) {{
        const messages = {{
            invalid_email: "Проверьте правильность email.",
            invalid_code: "Введите шестизначный код.",
            code_not_found: "Сначала запросите код.",
            code_used: "Этот код уже использован.",
            code_expired: "Срок действия кода истёк.",
            wrong_code: "Код введён неверно.",
            rate_limited: "Слишком много попыток. Попробуйте позже.",
            mail_unavailable: "Отправка писем временно недоступна."
        }};
        return messages[error] || "Не удалось выполнить запрос.";
    }}

    async function requestCode() {{
        const email = emailInput.value.trim();
        if (!email) {{
            setStatus("Введите email.", true);
            emailInput.focus();
            return;
        }}

        requestButton.disabled = true;
        setStatus("Отправляем код...", false);

        try {{
            const response = await fetch("/auth/email/request", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{ email }})
            }});
            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(otpError(data.error), true);
                return;
            }}

            codeSection.hidden = false;
            setStatus("Код отправлен. Проверьте входящие и спам.", false);
            codeInput.focus();
        }} catch (_) {{
            setStatus("Ошибка соединения.", true);
        }} finally {{
            requestButton.disabled = false;
        }}
    }}

    async function verifyCode() {{
        const email = emailInput.value.trim();
        const code = codeInput.value.trim();

        if (!/^[0-9]{{6}}$/.test(code)) {{
            setStatus("Введите шестизначный код.", true);
            codeInput.focus();
            return;
        }}

        verifyButton.disabled = true;
        setStatus("Проверяем код...", false);

        try {{
            const response = await fetch("/auth/email/verify", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{ email, code }})
            }});
            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(otpError(data.error), true);
                return;
            }}

            setStatus("Вход выполнен", false);
            window.location.replace(redirectTarget);
        }} catch (_) {{
            setStatus("Ошибка соединения.", true);
        }} finally {{
            verifyButton.disabled = false;
        }}
    }}

    requestButton.addEventListener("click", requestCode);
    verifyButton.addEventListener("click", verifyCode);
    codeInput.addEventListener("input", function () {{
        codeInput.value = codeInput.value.replace(/[^0-9]/g, "").slice(0, 6);
    }});
    emailInput.focus();
}})();
</script>
"##,
        redirect_target_json = serde_json::to_string(&redirect_target).unwrap_or_else(|_| "\"/app\"".to_string()),
    );

    Html(crate::web::templates::render_auth_page(
        crate::web::templates::AuthPageParams {
            document_title: "Вход по коду · ResursMap",
            heading: "Вход по коду",
            subtitle: "Для аккаунтов, созданных ранее по email-коду. Новым пользователям проще зарегистрироваться с паролем.",
            body_html,
            footer_html: &footer_html,
            script_html: &body_after,
        },
    ))
}

pub async fn forgot_password_page(Query(query): Query<AuthNextQuery>) -> Html<String> {
    let redirect_target = auth_redirect_target(query.next.as_deref());
    let login_href = auth_related_href("/login", &redirect_target);

    let body_html = r##"
        <label class="rm-auth-label" for="email-input">Email</label>
        <input id="email-input" class="ui-input rm-auth-input" type="email" autocomplete="email" maxlength="254" placeholder="name@example.com">

        <button id="request-button" type="button" class="ui-button rm-auth-button rm-auth-button--compact">Отправить код</button>

        <div id="reset-section" hidden class="rm-auth-step">
            <label class="rm-auth-label" for="code-input">Код из письма</label>
            <input id="code-input" class="ui-input rm-auth-input rm-auth-input--code" type="text" inputmode="numeric" maxlength="6" placeholder="000000">

            <label class="rm-auth-label" for="password-input">Новый пароль</label>
            <input id="password-input" class="ui-input rm-auth-input" type="password" autocomplete="new-password" maxlength="128" placeholder="Минимум 8 символов">

            <button id="reset-button" type="button" class="ui-button rm-auth-button rm-auth-button--compact">Сохранить пароль</button>
        </div>
"##;

    let footer_html = format!(
        r##"<p class="rm-auth-footer"><a href="{login_href}">Вернуться ко входу</a></p>"##,
        login_href = login_href,
    );

    let body_after = format!(
        r##"
<script>
(function () {{
    const redirectTarget = {redirect_target_json};
    const emailInput = document.getElementById("email-input");
    const codeInput = document.getElementById("code-input");
    const passwordInput = document.getElementById("password-input");
    const resetSection = document.getElementById("reset-section");
    const requestButton = document.getElementById("request-button");
    const resetButton = document.getElementById("reset-button");
    const authStatus = document.getElementById("auth-status");

    function setStatus(message, isError) {{
        authStatus.textContent = message;
        authStatus.classList.toggle("is-error", isError);
    }}

    function resetError(error) {{
        const messages = {{
            invalid_email: "Проверьте правильность email.",
            invalid_code: "Введите шестизначный код.",
            password_too_short: "Пароль должен быть не короче 8 символов.",
            code_not_found: "Сначала запросите код.",
            code_used: "Этот код уже использован.",
            code_expired: "Срок действия кода истёк.",
            wrong_code: "Код введён неверно.",
            rate_limited: "Слишком много попыток.",
            mail_unavailable: "Отправка писем временно недоступна."
        }};
        return messages[error] || "Не удалось выполнить запрос.";
    }}

    async function requestCode() {{
        const email = emailInput.value.trim();
        if (!email) {{
            setStatus("Введите email.", true);
            emailInput.focus();
            return;
        }}

        requestButton.disabled = true;
        setStatus("Отправляем код...", false);

        try {{
            const response = await fetch("/auth/forgot-password", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{ email }})
            }});
            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(resetError(data.error), true);
                return;
            }}

            resetSection.hidden = false;
            setStatus("Если аккаунт существует, код отправлен на email.", false);
            codeInput.focus();
        }} catch (_) {{
            setStatus("Ошибка соединения.", true);
        }} finally {{
            requestButton.disabled = false;
        }}
    }}

    async function submitReset() {{
        const email = emailInput.value.trim();
        const code = codeInput.value.trim();
        const password = passwordInput.value;

        if (password.length < 8) {{
            setStatus("Пароль должен быть не короче 8 символов.", true);
            passwordInput.focus();
            return;
        }}

        resetButton.disabled = true;
        setStatus("Сохраняем пароль...", false);

        try {{
            const response = await fetch("/auth/reset-password", {{
                method: "POST",
                headers: {{ "Content-Type": "application/json" }},
                body: JSON.stringify({{ email, code, password }})
            }});
            const data = await response.json().catch(function () {{
                return {{ ok: false, error: "invalid_response" }};
            }});

            if (!response.ok || !data.ok) {{
                setStatus(resetError(data.error), true);
                return;
            }}

            setStatus("Пароль сохранён", false);
            window.location.replace(redirectTarget);
        }} catch (_) {{
            setStatus("Ошибка соединения.", true);
        }} finally {{
            resetButton.disabled = false;
        }}
    }}

    requestButton.addEventListener("click", requestCode);
    resetButton.addEventListener("click", submitReset);
    codeInput.addEventListener("input", function () {{
        codeInput.value = codeInput.value.replace(/[^0-9]/g, "").slice(0, 6);
    }});
    emailInput.focus();
}})();
</script>
"##,
        redirect_target_json = serde_json::to_string(&redirect_target).unwrap_or_else(|_| "\"/app\"".to_string()),
    );

    Html(crate::web::templates::render_auth_page(
        crate::web::templates::AuthPageParams {
            document_title: "Сброс пароля · ResursMap",
            heading: "Сброс пароля",
            subtitle: "Отправим код на email. Подойдёт и для старых аккаунтов без пароля — вы сможете задать новый.",
            body_html,
            footer_html: &footer_html,
            script_html: &body_after,
        },
    ))
}
