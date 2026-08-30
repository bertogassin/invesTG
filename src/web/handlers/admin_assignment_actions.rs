use super::admin_access::{
    load_admin_context, record_denied_access, valid_admin_session_public_id, AdminContext,
    AdminPermission,
};
use super::auth::verify_authenticated_user;
use super::common::request_is_cross_site;
use crate::state::app_state::AppState;
use axum::{
    extract::{Form, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rusqlite::OptionalExtension;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const STEP_UP_MAX_AGE_SECONDS: i64 = 900;
const MIN_DURATION_DAYS: i64 = 1;
const MAX_DURATION_DAYS: i64 = 365;

#[derive(Debug, Deserialize)]
pub struct CreateAdminAssignmentForm {
    user_id: i64,
    role_level: i64,
    scope_id: i64,
    duration_days: i64,
    reason: String,
}

#[derive(Debug)]
struct AssignableUser {
    id: i64,
    display_name: String,
    username: String,
}

#[derive(Debug)]
struct AssignableScope {
    id: i64,
    scope_type: String,
    display_name: String,
    parent_name: String,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn expected_scope_type(level: i64) -> Option<&'static str> {
    match level {
        1 => Some("group"),
        2 => Some("city"),
        3 => Some("country"),
        4 => Some("continent"),
        _ => None,
    }
}

fn role_title(level: i64) -> Option<&'static str> {
    match level {
        1 => Some("Помощник группы"),
        2 => Some("Администратор города"),
        3 => Some("Администратор страны"),
        4 => Some("Администратор континента"),
        _ => None,
    }
}

fn safe_permission_mask(level: i64) -> Option<i64> {
    match level {
        1 => Some((1_i64 << 0) | (1_i64 << 1) | (1_i64 << 2) | (1_i64 << 3)),
        2 => Some(
            (1_i64 << 0) | (1_i64 << 1) | (1_i64 << 2) | (1_i64 << 3) | (1_i64 << 4) | (1_i64 << 5),
        ),
        3..=4 => Some((1_i64 << 0) | (1_i64 << 1) | (1_i64 << 2) | (1_i64 << 3) | (1_i64 << 4)),
        _ => None,
    }
}

fn reason_is_valid(reason: &str) -> bool {
    let length = reason.chars().count();

    (5..=500).contains(&length) && !reason.chars().any(char::is_control)
}

fn duration_is_valid(days: i64) -> bool {
    (MIN_DURATION_DAYS..=MAX_DURATION_DAYS).contains(&days)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn request_metadata(headers: &HeaderMap) -> (String, String) {
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

    (ip_address, user_agent)
}

fn owner_context_and_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(AdminContext, String), Box<Response>> {
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
            "admin_assignment_access_denied",
            "Нет активного административного назначения",
        );

        Box::new((StatusCode::NOT_FOUND, "404").into_response())
    })?;

    if !context.is_owner() || !context.has_permission(AdminPermission::GlobalAdminsManage) {
        record_denied_access(
            state,
            authenticated.user_id,
            "admin_assignment_permission_denied",
            "Недостаточно прав для назначения администратора",
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

fn step_up_is_fresh(
    state: &AppState,
    context: &AdminContext,
    session_public_id: &str,
    now: i64,
) -> bool {
    state.db_pool.get().ok().and_then(|connection| {
        connection
            .query_row(
                "SELECT COUNT(*)
                     FROM admin_sessions
                     WHERE session_public_id = ?1
                       AND user_id = ?2
                       AND assignment_id = ?3
                       AND revoked_at IS NULL
                       AND expires_at > ?4
                       AND two_factor_verified = 1
                       AND reauthenticated_at IS NOT NULL
                       AND reauthenticated_at > ?5",
                rusqlite::params![
                    session_public_id,
                    context.user_id,
                    context.assignment_id,
                    now,
                    now - STEP_UP_MAX_AGE_SECONDS
                ],
                |row| row.get::<_, i64>(0),
            )
            .ok()
    }) == Some(1)
}

fn protected_html_response(html: String) -> Response {
    let mut response = Html(html).into_response();

    let headers = response.headers_mut();

    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             script-src 'self'; \
             connect-src 'self'; \
             object-src 'none'; \
             base-uri 'none'; \
             frame-ancestors 'none'; \
             form-action 'self'",
        ),
    );

    response
}

fn render_assignment_form(users: &[AssignableUser], scopes: &[AssignableScope]) -> String {
    let user_options = users
        .iter()
        .map(|user| {
            let username = if user.username.is_empty() {
                String::new()
            } else {
                format!(" · @{}", user.username)
            };

            format!(
                r#"<option value="{id}">{name}{username} · User #{id}</option>"#,
                id = user.id,
                name = escape_html(&user.display_name),
                username = escape_html(&username),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let mut scope_options = String::new();

    for scope_type in ["continent", "country", "city", "group"] {
        let label = match scope_type {
            "continent" => "Уровень 4 · Континенты",
            "country" => "Уровень 3 · Страны",
            "city" => "Уровень 2 · Города",
            "group" => "Уровень 1 · Группы",
            _ => "Территории",
        };

        scope_options.push_str(&format!(r#"<optgroup label="{}">"#, escape_html(label)));

        for scope in scopes.iter().filter(|scope| scope.scope_type == scope_type) {
            let parent = if scope.parent_name.is_empty() {
                String::new()
            } else {
                format!(" · {}", scope.parent_name)
            };

            scope_options.push_str(&format!(
                r#"<option value="{id}">{name}{parent}</option>"#,
                id = scope.id,
                name = escape_html(&scope.display_name),
                parent = escape_html(&parent),
            ));
        }

        scope_options.push_str("</optgroup>");
    }

    format!(
        r##"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="color-scheme" content="dark">
<title>Новое назначение · ResursMap</title>
<style>
:root {{
    --bg:#07090d;
    --surface:#11151c;
    --gold:#dfc07f;
    --green:#62e0ad;
    --red:#ff7882;
    --text:#f5f2eb;
    --muted:#969dab;
    --line:rgba(223,192,127,.20);
}}
* {{ box-sizing:border-box; }}
body {{
    margin:0;
    min-height:100vh;
    padding:
        max(18px,env(safe-area-inset-top))
        16px
        max(32px,env(safe-area-inset-bottom));
    color:var(--text);
    background:
        radial-gradient(circle at 10% 0%,rgba(45,110,82,.18),transparent 34%),
        radial-gradient(circle at 100% 5%,rgba(125,91,190,.17),transparent 34%),
        linear-gradient(160deg,#090b10,#05070a 70%);
    font-family:Inter,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;
}}
.page {{
    width:min(780px,100%);
    margin:0 auto;
}}
.topbar {{
    display:flex;
    justify-content:space-between;
    align-items:center;
    gap:14px;
    margin-bottom:18px;
}}
.back {{
    min-height:44px;
    display:inline-flex;
    align-items:center;
    padding:0 16px;
    border:1px solid var(--line);
    border-radius:14px;
    color:var(--text);
    text-decoration:none;
    font-weight:850;
}}
.protected {{
    color:var(--green);
    font-size:11px;
    font-weight:950;
}}
.hero,
.form-card,
.policy {{
    border:1px solid var(--line);
    background:
        linear-gradient(145deg,rgba(20,24,32,.96),rgba(10,12,17,.97));
    box-shadow:0 24px 70px rgba(0,0,0,.28);
}}
.hero {{
    padding:28px;
    border-radius:26px;
}}
.kicker {{
    color:var(--gold);
    font-size:11px;
    font-weight:950;
    letter-spacing:.17em;
}}
h1 {{
    margin:12px 0 10px;
    font-size:clamp(32px,7vw,54px);
    line-height:1;
    letter-spacing:-.04em;
}}
.hero p,
.policy {{
    color:var(--muted);
    line-height:1.65;
}}
.policy {{
    margin-top:15px;
    padding:17px;
    border-radius:18px;
    font-size:13px;
}}
.form-card {{
    display:grid;
    gap:18px;
    margin-top:18px;
    padding:24px;
    border-radius:24px;
}}
label {{
    display:grid;
    gap:8px;
    color:var(--muted);
    font-size:12px;
    font-weight:850;
}}
input,
select,
textarea {{
    width:100%;
    min-height:49px;
    padding:11px 14px;
    border:1px solid rgba(255,255,255,.12);
    border-radius:14px;
    outline:none;
    color:var(--text);
    background:rgba(255,255,255,.04);
    font:inherit;
}}
textarea {{
    min-height:115px;
    resize:vertical;
}}
option,
optgroup {{
    color:#111;
}}
.grid {{
    display:grid;
    grid-template-columns:1fr 1fr;
    gap:16px;
}}
button {{
    min-height:52px;
    border:1px solid rgba(223,192,127,.38);
    border-radius:15px;
    color:#17130b;
    background:linear-gradient(135deg,#efd49a,#cda85f);
    font-size:14px;
    font-weight:950;
    cursor:pointer;
}}
.warning {{
    margin:0;
    color:var(--red);
    font-size:12px;
    line-height:1.55;
}}
@media (max-width:620px) {{
    .hero,
    .form-card {{
        padding:20px;
    }}
    .grid {{
        grid-template-columns:1fr;
    }}
    .protected {{
        display:none;
    }}
}}
</style>
</head>
<body>
<main class="page">
    <div class="topbar">
        <a class="back"
           href="/app/center/administrators">
            ← Администраторы
        </a>
        <span class="protected">
            OWNER · FRESH STEP-UP
        </span>
    </div>

    <section class="hero">
        <div class="kicker">
            ResursMap · Назначения
        </div>
        <h1>Новое назначение</h1>
        <p>
            Назначение создаётся только владельцем ResursMap.
            Уровень строго связан с территорией, действие
            записывается в защищённую цепочку аудита.
        </p>
    </section>

    <section class="policy">
        Уровень 1 → группа · Уровень 2 → город ·
        Уровень 3 → страна · Уровень 4 → континент.
        Критические полномочия автоматически не выдаются.
        Срок назначения: от 1 до 365 дней.
    </section>

    <form class="form-card"
          method="post"
          action="/app/center/administrators">
        <label>
            Пользователь
            <select name="user_id" required>
                <option value="">
                    Выберите активного пользователя
                </option>
                {user_options}
            </select>
        </label>

        <div class="grid">
            <label>
                Уровень
                <select name="role_level" required>
                    <option value="">
                        Выберите уровень
                    </option>
                    <option value="1">
                        1 · Помощник группы
                    </option>
                    <option value="2">
                        2 · Администратор города
                    </option>
                    <option value="3">
                        3 · Администратор страны
                    </option>
                    <option value="4">
                        4 · Администратор континента
                    </option>
                </select>
            </label>

            <label>
                Срок в днях
                <input type="number"
                       name="duration_days"
                       min="1"
                       max="365"
                       value="90"
                       required>
            </label>
        </div>

        <label>
            Территория
            <select name="scope_id" required>
                <option value="">
                    Выберите территорию нужного уровня
                </option>
                {scope_options}
            </select>
        </label>

        <label>
            Причина назначения
            <textarea name="reason"
                      minlength="5"
                      maxlength="500"
                      required
                      placeholder="Опишите основание и ответственность администратора"></textarea>
        </label>

        <p class="warning">
            Перед созданием назначения потребуется свежее
            подтверждение владельца, выполненное не более
            15 минут назад.
        </p>

        <button type="submit">
            Создать защищённое назначение
        </button>
    </form>
</main>
</body>
</html>"##,
        user_options = user_options,
        scope_options = scope_options,
    )
}

pub async fn new_admin_assignment_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let (context, session_public_id) = match owner_context_and_session(&state, &headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };

    let now = unix_now();

    if !step_up_is_fresh(&state, &context, &session_public_id, now) {
        return (
            StatusCode::SEE_OTHER,
            [(header::LOCATION, "/app/center/security")],
        )
            .into_response();
    }

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

    let mut users = Vec::new();

    if let Ok(mut statement) = connection.prepare(
        "SELECT
            users.id,
            COALESCE(profiles.username, ''),
            COALESCE(profiles.first_name, ''),
            COALESCE(profiles.last_name, '')
         FROM users
         LEFT JOIN profiles
           ON profiles.user_id = users.id
         WHERE users.is_active = 1
           AND users.id <> ?1
           AND NOT EXISTS (
                SELECT 1
                FROM admin_assignments AS assignment
                WHERE assignment.user_id = users.id
                  AND assignment.status = 'active'
                  AND assignment.valid_from <= ?2
                  AND (
                      assignment.valid_until IS NULL
                      OR assignment.valid_until > ?2
                  )
           )
         ORDER BY
            profiles.first_name,
            profiles.username,
            users.id",
    ) {
        if let Ok(rows) = statement.query_map(rusqlite::params![context.user_id, now], |row| {
            let id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            let first_name: String = row.get(2)?;
            let last_name: String = row.get(3)?;

            let full_name = format!("{first_name} {last_name}").trim().to_string();

            let display_name = if !full_name.is_empty() {
                full_name
            } else if !username.is_empty() {
                format!("@{username}")
            } else {
                format!("Пользователь #{id}")
            };

            Ok(AssignableUser {
                id,
                display_name,
                username,
            })
        }) {
            users.extend(rows.flatten());
        }
    }

    let mut scopes = Vec::new();

    if let Ok(mut statement) = connection.prepare(
        "SELECT
            scope.id,
            scope.scope_type,
            scope.display_name,
            COALESCE(parent.display_name, '')
         FROM geographic_scopes AS scope
         LEFT JOIN geographic_scopes AS parent
           ON parent.id = scope.parent_scope_id
         WHERE scope.is_active = 1
           AND scope.scope_type IN (
               'continent', 'country', 'city', 'group'
           )
         ORDER BY
            CASE scope.scope_type
                WHEN 'continent' THEN 1
                WHEN 'country' THEN 2
                WHEN 'city' THEN 3
                WHEN 'group' THEN 4
            END,
            scope.display_name,
            scope.id",
    ) {
        if let Ok(rows) = statement.query_map([], |row| {
            Ok(AssignableScope {
                id: row.get(0)?,
                scope_type: row.get(1)?,
                display_name: row.get(2)?,
                parent_name: row.get(3)?,
            })
        }) {
            scopes.extend(rows.flatten());
        }
    }

    drop(connection);

    protected_html_response(render_assignment_form(&users, &scopes))
}

pub async fn create_admin_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<CreateAdminAssignmentForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён CSRF-защитой").into_response();
    }

    let (context, session_public_id) = match owner_context_and_session(&state, &headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };

    let expected_scope = match expected_scope_type(payload.role_level) {
        Some(value) => value,
        None => {
            return (StatusCode::BAD_REQUEST, "Назначать можно только уровни 1–4").into_response();
        }
    };

    let role_name = role_title(payload.role_level).unwrap_or("Администратор");

    let permission_mask = match safe_permission_mask(payload.role_level) {
        Some(value) => value,
        None => {
            return (StatusCode::BAD_REQUEST, "Некорректный уровень").into_response();
        }
    };

    if payload.user_id <= 0 || payload.user_id == context.user_id {
        return (StatusCode::BAD_REQUEST, "Некорректный пользователь").into_response();
    }

    if !duration_is_valid(payload.duration_days) {
        return (
            StatusCode::BAD_REQUEST,
            "Срок должен составлять от 1 до 365 дней",
        )
            .into_response();
    }

    let reason = payload.reason.trim();

    if !reason_is_valid(reason) {
        return (
            StatusCode::BAD_REQUEST,
            "Причина должна содержать от 5 до 500 символов",
        )
            .into_response();
    }

    let now = unix_now();

    if !step_up_is_fresh(&state, &context, &session_public_id, now) {
        if let Ok(connection) = state.db_pool.get() {
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
                    'admin_assignment_step_up_required',
                    'high',
                    'Требуется свежее подтверждение владельца'
                 )",
                rusqlite::params![context.user_id, context.assignment_id, session_public_id],
            );
        }

        return (
            StatusCode::PRECONDITION_REQUIRED,
            [(header::LOCATION, "/app/center/security")],
            "Требуется повторное подтверждение владельца",
        )
            .into_response();
    }

    let valid_until = now.saturating_add(payload.duration_days.saturating_mul(86_400));

    let (ip_address, user_agent) = request_metadata(&headers);

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

    let transaction =
        match connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
            Ok(transaction) => transaction,
            Err(_) => {
                return (StatusCode::CONFLICT, "Не удалось заблокировать операцию").into_response();
            }
        };

    let target_active: i64 = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM users
             WHERE id = ?1
               AND is_active = 1",
            rusqlite::params![payload.user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if target_active != 1 {
        return (StatusCode::BAD_REQUEST, "Активный пользователь не найден").into_response();
    }

    let target_is_owner: i64 = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM admin_assignments
             WHERE user_id = ?1
               AND role_level = 5",
            rusqlite::params![payload.user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if target_is_owner != 0 {
        return (
            StatusCode::FORBIDDEN,
            "Назначение владельца изменять нельзя",
        )
            .into_response();
    }

    let existing_active: i64 = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM admin_assignments
             WHERE user_id = ?1
               AND status = 'active'
               AND valid_from <= ?2
               AND (
                   valid_until IS NULL
                   OR valid_until > ?2
               )",
            rusqlite::params![payload.user_id, now],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if existing_active != 0 {
        return (
            StatusCode::CONFLICT,
            "У пользователя уже есть активное назначение",
        )
            .into_response();
    }

    let scope: Option<(String, String)> = transaction
        .query_row(
            "SELECT scope_type, display_name
             FROM geographic_scopes
             WHERE id = ?1
               AND is_active = 1",
            rusqlite::params![payload.scope_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .unwrap_or(None);

    let Some((actual_scope_type, scope_name)) = scope else {
        return (StatusCode::BAD_REQUEST, "Территория не найдена").into_response();
    };

    if actual_scope_type != expected_scope {
        return (
            StatusCode::BAD_REQUEST,
            "Уровень не соответствует типу территории",
        )
            .into_response();
    }

    let inserted = transaction.execute(
        "INSERT INTO admin_assignments (
            user_id,
            role_level,
            scope_type,
            scope_id,
            permission_mask,
            category_restrictions,
            valid_from,
            valid_until,
            status,
            assigned_by_user_id,
            assignment_reason,
            last_change_reason,
            created_at,
            updated_at
         )
         VALUES (
            ?1, ?2, ?3, ?4, ?5,
            '[]', ?6, ?7, 'active',
            ?8, ?9, ?9, ?6, ?6
         )",
        rusqlite::params![
            payload.user_id,
            payload.role_level,
            actual_scope_type,
            payload.scope_id,
            permission_mask,
            now,
            valid_until,
            context.user_id,
            reason
        ],
    );

    if inserted.is_err() {
        return (StatusCode::CONFLICT, "Не удалось создать назначение").into_response();
    }

    let new_assignment_id = transaction.last_insert_rowid();

    let previous_hash: String = transaction
        .query_row(
            "SELECT event_hash
             FROM admin_action_audit
             ORDER BY id DESC
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None)
        .unwrap_or_else(|| "genesis".to_string());

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let event_seed = format!(
        "{}:{}:{}:{}:{}",
        context.user_id, payload.user_id, new_assignment_id, now, nanos
    );

    let event_public_id = hex::encode(Sha256::digest(event_seed.as_bytes()))[..32].to_string();

    let new_state = serde_json::json!({
        "assignment_id": new_assignment_id,
        "user_id": payload.user_id.to_string(),
        "role_level": payload.role_level,
        "role_title": role_name,
        "scope_type": actual_scope_type,
        "scope_id": payload.scope_id,
        "scope_name": scope_name,
        "permission_mask": permission_mask,
        "valid_from": now,
        "valid_until": valid_until,
        "status": "active"
    })
    .to_string();

    let target_id = new_assignment_id.to_string();

    let hash_payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        previous_hash,
        event_public_id,
        context.user_id,
        "admin_assignment_created",
        target_id,
        "{}",
        new_state,
        now
    );

    let event_hash = hex::encode(Sha256::digest(hash_payload.as_bytes()));

    let audit_inserted = transaction.execute(
        "INSERT INTO admin_action_audit (
            event_public_id,
            administrator_user_id,
            assignment_id,
            administrator_level,
            scope_type,
            scope_id,
            action_type,
            target_type,
            target_id,
            old_state,
            new_state,
            reason,
            ip_address,
            user_agent,
            session_public_id,
            result,
            approvers,
            previous_hash,
            event_hash,
            created_at
         )
         VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            'admin_assignment_created',
            'admin_assignment',
            ?7, '{}', ?8, ?9, ?10, ?11, ?12,
            'success', '[]', ?13, ?14, ?15
         )",
        rusqlite::params![
            event_public_id,
            context.user_id,
            context.assignment_id,
            context.level.number(),
            context.scope_type,
            context.scope_id,
            target_id,
            new_state,
            reason,
            ip_address,
            user_agent,
            session_public_id,
            previous_hash,
            event_hash,
            now
        ],
    );

    if audit_inserted.unwrap_or(0) != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось записать защищённый аудит",
        )
            .into_response();
    }

    let notification_message = format!(
        "Вы назначены: {role_name}. Территория: \
         {scope_name}. Назначение действует {} дней.",
        payload.duration_days
    );

    let notification_inserted = transaction.execute(
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
            ?1, NULL,
            'admin_assignment',
            'Административное назначение',
            ?2, 0, ?3
         )",
        rusqlite::params![payload.user_id, notification_message, now],
    );

    if notification_inserted.unwrap_or(0) != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось создать уведомление",
        )
            .into_response();
    }

    let security_inserted = transaction.execute(
        "INSERT INTO admin_security_events (
            user_id,
            assignment_id,
            session_public_id,
            event_type,
            severity,
            ip_address,
            user_agent,
            details
         )
         VALUES (
            ?1, ?2, ?3,
            'admin_assignment_created',
            'warning',
            ?4, ?5, ?6
         )",
        rusqlite::params![
            payload.user_id,
            new_assignment_id,
            session_public_id,
            ip_address,
            user_agent,
            reason
        ],
    );

    if security_inserted.unwrap_or(0) != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось записать событие безопасности",
        )
            .into_response();
    }

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось завершить транзакцию",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/administrators?assigned=1")],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_scope_mapping_is_strict() {
        assert_eq!(expected_scope_type(1), Some("group"));
        assert_eq!(expected_scope_type(2), Some("city"));
        assert_eq!(expected_scope_type(3), Some("country"));
        assert_eq!(expected_scope_type(4), Some("continent"));
        assert_eq!(expected_scope_type(5), None);
        assert_eq!(expected_scope_type(0), None);
    }

    #[test]
    fn default_masks_exclude_critical_permissions() {
        assert_eq!(safe_permission_mask(1), Some(15));
        assert_eq!(safe_permission_mask(2), Some(63));
        assert_eq!(safe_permission_mask(3), Some(31));
        assert_eq!(safe_permission_mask(4), Some(31));
        assert_eq!(safe_permission_mask(5), None);

        assert_eq!(
            safe_permission_mask(1).expect("helper mask") & (1_i64 << 5),
            0,
        );

        assert_ne!(
            safe_permission_mask(2).expect("city mask") & (1_i64 << 5),
            0,
        );

        for level in 1..=4 {
            let mask = safe_permission_mask(level).expect("mask");

            assert_eq!(mask & (1_i64 << 6), 0);
            assert_eq!(mask & (1_i64 << 7), 0);
            assert_eq!(mask & (1_i64 << 13), 0);
        }
    }

    #[test]
    fn assignment_input_policy_is_enforced() {
        assert!(duration_is_valid(1));
        assert!(duration_is_valid(365));
        assert!(!duration_is_valid(0));
        assert!(!duration_is_valid(366));

        assert!(reason_is_valid("Управление городской группой"));
        assert!(!reason_is_valid("нет"));
        assert!(!reason_is_valid("Причина\nс переводом строки"));
    }
}
