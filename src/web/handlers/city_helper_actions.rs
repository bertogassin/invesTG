use super::admin_access::{
    create_admin_session, load_admin_context, record_denied_access, valid_admin_session_public_id,
    verify_admin_session, AdminContext, AdminPermission,
};
use super::auth::verify_authenticated_user;
use super::common::{csrf_rejected_response, request_is_cross_site};
use crate::state::app_state::AppState;
use crate::web::templates::{admin_ops_page_themed, escape_html, workflow_status_label_or_raw};
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rusqlite::{params, OptionalExtension, Transaction};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const MIN_DURATION_DAYS: i64 = 1;
const MAX_DURATION_DAYS: i64 = 365;
const HELPER_PERMISSION_MASK: i64 = 15;
const HELPER_LIST_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
pub struct CityHelperCreateForm {
    user_id: i64,
    group_scope_id: i64,
    duration_days: i64,
    reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CityHelperLifecycleForm {
    reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HelperAction {
    Suspend,
    Restore,
    Revoke,
}

impl HelperAction {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "suspend" => Some(Self::Suspend),
            "restore" => Some(Self::Restore),
            "revoke" => Some(Self::Revoke),
            _ => None,
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Suspend => "city_helper_suspended",
            Self::Restore => "city_helper_restored",
            Self::Revoke => "city_helper_revoked",
        }
    }

    fn next_status(self) -> &'static str {
        match self {
            Self::Suspend => "suspended",
            Self::Restore => "active",
            Self::Revoke => "revoked",
        }
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn reason_is_valid(value: &str) -> bool {
    let length = value.trim().chars().count();

    (5..=500).contains(&length) && !value.chars().any(char::is_control)
}

fn duration_is_valid(value: i64) -> bool {
    (MIN_DURATION_DAYS..=MAX_DURATION_DAYS).contains(&value)
}

fn request_metadata(headers: &HeaderMap) -> (String, String) {
    let ip = headers
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

    (ip, user_agent)
}

fn city_manager_context(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminContext, Box<Response>> {
    let user = verify_authenticated_user(state, headers).ok_or_else(|| {
        Box::new(
            (
                StatusCode::UNAUTHORIZED,
                "Требуется вход в аккаунт ResursMap",
            )
                .into_response(),
        )
    })?;

    let context = load_admin_context(state, user.user_id).ok_or_else(|| {
        record_denied_access(
            state,
            user.user_id,
            "city_helper_manager_missing",
            "Нет активного назначения",
        );

        Box::new((StatusCode::NOT_FOUND, "404").into_response())
    })?;

    if context.level.number() != 2
        || context.scope_type != "city"
        || !context.has_permission(AdminPermission::AssistantsManage)
    {
        record_denied_access(
            state,
            user.user_id,
            "city_helper_manager_permission_denied",
            "Требуется уровень 2, область города и AssistantsManage",
        );

        return Err(Box::new(
            (StatusCode::FORBIDDEN, "Управление помощниками недоступно").into_response(),
        ));
    }

    Ok(context)
}

#[allow(clippy::too_many_arguments)]
fn append_audit(
    transaction: &Transaction<'_>,
    context: &AdminContext,
    session_public_id: &str,
    action_type: &str,
    target_assignment_id: i64,
    old_state: &str,
    new_state: &str,
    reason: &str,
    ip: &str,
    user_agent: &str,
    now: i64,
) -> rusqlite::Result<()> {
    let previous_hash = transaction
        .query_row(
            "SELECT event_hash
             FROM admin_action_audit
             ORDER BY id DESC
             LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| "genesis".to_string());

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let event_seed = format!(
        "{}:{}:{}:{}:{}:{}",
        context.user_id, target_assignment_id, action_type, now, nanos, previous_hash,
    );

    let event_public_id = hex::encode(Sha256::digest(event_seed.as_bytes()))[..32].to_string();

    let target_id = target_assignment_id.to_string();

    let hash_payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        previous_hash,
        event_public_id,
        context.user_id,
        action_type,
        target_id,
        old_state,
        new_state,
        now,
    );

    let event_hash = hex::encode(Sha256::digest(hash_payload.as_bytes()));

    let inserted = transaction.execute(
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
             ?7, 'admin_assignment', ?8,
             ?9, ?10, ?11, ?12, ?13, ?14,
             'success', '[]', ?15, ?16, ?17
         )",
        params![
            event_public_id,
            context.user_id,
            context.assignment_id,
            context.level.number(),
            context.scope_type,
            context.scope_id,
            action_type,
            target_id,
            old_state,
            new_state,
            reason,
            ip,
            user_agent,
            session_public_id,
            previous_hash,
            event_hash,
            now,
        ],
    )?;

    if inserted != 1 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

pub async fn city_helpers_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let context = match city_manager_context(&state, &headers) {
        Ok(context) => context,
        Err(response) => return *response,
    };

    if !verify_admin_session(&state, &headers, context.user_id, context.assignment_id) {
        let cookie = match create_admin_session(&state, &context, &headers) {
            Ok(cookie) => cookie,
            Err(_) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Не удалось открыть административную сессию",
                )
                    .into_response();
            }
        };

        let mut response = StatusCode::SEE_OTHER.into_response();

        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_static("/app/center/city/helpers"),
        );

        if let Ok(value) = HeaderValue::from_str(&cookie) {
            response.headers_mut().insert(header::SET_COOKIE, value);
        }

        return response;
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

    let city_name = connection
        .query_row(
            "SELECT display_name
             FROM geographic_scopes
             WHERE id = ?1
               AND scope_type = 'city'
               AND is_active = 1",
            params![context.scope_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| context.scope_name.clone());

    let groups = connection
        .prepare(
            "SELECT id, display_name
             FROM geographic_scopes
             WHERE parent_scope_id = ?1
               AND scope_type = 'group'
               AND is_active = 1
             ORDER BY display_name, id",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![context.scope_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let helpers = connection
        .prepare(
            "SELECT
                 assignment.id,
                 assignment.user_id,
                 group_scope.display_name,
                 assignment.status,
                 assignment.valid_until,
                 assignment.last_change_reason
             FROM admin_assignments AS assignment
             JOIN geographic_scopes AS group_scope
               ON group_scope.id = assignment.scope_id
             WHERE assignment.role_level = 1
               AND assignment.scope_type = 'group'
               AND group_scope.parent_scope_id = ?1
             ORDER BY
                 CASE assignment.status
                     WHEN 'active' THEN 0
                     WHEN 'suspended' THEN 1
                     ELSE 2
                 END,
                 assignment.id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![context.scope_id, HELPER_LIST_LIMIT], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let group_options = groups
        .iter()
        .map(|(id, name)| {
            format!(
                r#"<option value="{id}">{name}</option>"#,
                id = id,
                name = escape_html(name),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let helper_cards = if helpers.is_empty() {
        r#"<div class="empty">Помощники пока не назначены.</div>"#.to_string()
    } else {
        helpers
            .iter()
            .map(
                |(assignment_id, user_id, group_name, status, valid_until, last_reason)| {
                    let actions = match status.as_str() {
                        "active" => format!(
                            r#"
                            <form method="post"
                                  action="/app/center/city/helpers/{id}/suspend">
                                <input name="reason"
                                       minlength="5"
                                       maxlength="500"
                                       placeholder="Причина приостановки"
                                       required>
                                <button class="warning">Приостановить</button>
                            </form>
                            <form method="post"
                                  action="/app/center/city/helpers/{id}/revoke">
                                <input name="reason"
                                       minlength="5"
                                       maxlength="500"
                                       placeholder="Причина отзыва"
                                       required>
                                <button class="danger">Отозвать</button>
                            </form>"#,
                            id = assignment_id,
                        ),
                        "suspended" => format!(
                            r#"
                            <form method="post"
                                  action="/app/center/city/helpers/{id}/restore">
                                <input name="reason"
                                       minlength="5"
                                       maxlength="500"
                                       placeholder="Причина восстановления"
                                       required>
                                <button>Восстановить</button>
                            </form>
                            <form method="post"
                                  action="/app/center/city/helpers/{id}/revoke">
                                <input name="reason"
                                       minlength="5"
                                       maxlength="500"
                                       placeholder="Причина отзыва"
                                       required>
                                <button class="danger">Отозвать</button>
                            </form>"#,
                            id = assignment_id,
                        ),
                        _ => String::new(),
                    };

                    format!(
                        r#"<article class="card">
                            <div class="head">
                                <div>
                                    <strong>Пользователь #{user_id}</strong>
                                    <small>{group_name}</small>
                                </div>
                                <span>{status}</span>
                            </div>
                            <p>Назначение #{assignment_id} · срок: {until}</p>
                            <p>{last_reason}</p>
                            <div class="actions">{actions}</div>
                        </article>"#,
                        user_id = user_id,
                        group_name = escape_html(group_name),
                        status = escape_html(&workflow_status_label_or_raw(status)),
                        assignment_id = assignment_id,
                        until = valid_until
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "без срока".to_string()),
                        last_reason = escape_html(last_reason),
                        actions = actions,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let content = format!(
        r#"
<section class="hero">
    <div class="kicker">Уровень 2 · управление командой</div>
    <h1>Помощники города</h1>
    <p>{city}</p>
    <a class="back" href="/app/center/city">← Вернуться в кабинет города</a>
</section>

<form class="create"
      method="post"
      action="/app/center/city/helpers">
    <h2>Назначить помощника</h2>
    <div class="fields">
        <label>
            ID пользователя
            <input name="user_id"
                   type="number"
                   min="1"
                   required>
        </label>
        <label>
            Группа города
            <select name="group_scope_id" required>
                {group_options}
            </select>
        </label>
        <label>
            Срок в днях
            <input name="duration_days"
                   type="number"
                   min="1"
                   max="365"
                   value="90"
                   required>
        </label>
        <label>
            Причина назначения
            <input name="reason"
                   minlength="5"
                   maxlength="500"
                   required>
        </label>
    </div>
    <p>
        Помощник получит только права уровня 1:
        проверка, отклонение, эскалация и жалобы.
    </p>
    <button type="submit">Назначить помощника</button>
</form>

<section class="grid">
    {helper_cards}
</section>
"#,
        city = escape_html(&city_name),
        group_options = group_options,
        helper_cards = helper_cards,
    );

    let html = admin_ops_page_themed(
        &format!("Помощники · {}", escape_html(&city_name)),
        "rm-admin-ops--city rm-admin-ops--helpers",
        &content,
    );

    let mut response = Html(html).into_response();

    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );

    response
}

pub async fn city_helper_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CityHelperCreateForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if form.user_id <= 0
        || form.group_scope_id <= 0
        || !duration_is_valid(form.duration_days)
        || !reason_is_valid(&form.reason)
    {
        return (StatusCode::BAD_REQUEST, "Некорректные данные назначения").into_response();
    }

    let context = match city_manager_context(&state, &headers) {
        Ok(context) => context,
        Err(response) => return *response,
    };

    if form.user_id == context.user_id {
        return (
            StatusCode::BAD_REQUEST,
            "Нельзя назначить самого себя помощником",
        )
            .into_response();
    }

    let session_public_id = match valid_admin_session_public_id(
        &state,
        &headers,
        context.user_id,
        context.assignment_id,
    ) {
        Some(value) => value,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Административная сессия недействительна",
            )
                .into_response();
        }
    };

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

    let now = unix_now();
    let valid_until = now.saturating_add(form.duration_days.saturating_mul(86_400));
    let reason = form.reason.trim();
    let (ip, user_agent) = request_metadata(&headers);

    let transaction = match connection.transaction() {
        Ok(transaction) => transaction,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Не удалось открыть транзакцию",
            )
                .into_response();
        }
    };

    let group_name = transaction
        .query_row(
            "SELECT display_name
             FROM geographic_scopes
             WHERE id = ?1
               AND parent_scope_id = ?2
               AND scope_type = 'group'
               AND is_active = 1",
            params![form.group_scope_id, context.scope_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .unwrap_or(None);

    let Some(group_name) = group_name else {
        record_denied_access(
            &state,
            context.user_id,
            "city_helper_group_scope_denied",
            &format!("group_scope_id={}", form.group_scope_id),
        );

        return (
            StatusCode::FORBIDDEN,
            "Группа находится вне назначенного города",
        )
            .into_response();
    };

    let user_exists = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM users
             WHERE id = ?1
               AND is_active = 1",
            params![form.user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    if user_exists != 1 {
        return (StatusCode::NOT_FOUND, "Активный пользователь не найден").into_response();
    }

    let conflicting = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM admin_assignments
             WHERE user_id = ?1
               AND status IN (
                   'pending','active','suspended'
               )
               AND (
                   valid_until IS NULL
                   OR valid_until > ?2
               )",
            params![form.user_id, now],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    if conflicting != 0 {
        return (
            StatusCode::CONFLICT,
            "У пользователя уже есть действующее назначение",
        )
            .into_response();
    }

    let existing_id = transaction
        .query_row(
            "SELECT id
             FROM admin_assignments
             WHERE user_id = ?1
               AND role_level = 1
               AND scope_type = 'group'
               AND scope_id = ?2",
            params![form.user_id, form.group_scope_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .unwrap_or(None);

    let assignment_id = if let Some(id) = existing_id {
        let changed = transaction
            .execute(
                "UPDATE admin_assignments
                 SET permission_mask = ?2,
                     category_restrictions = '[]',
                     valid_from = ?3,
                     valid_until = ?4,
                     status = 'active',
                     assigned_by_user_id = ?5,
                     assignment_reason = ?6,
                     last_change_reason = ?6,
                     updated_at = ?3
                 WHERE id = ?1
                   AND status IN ('revoked','expired')",
                params![
                    id,
                    HELPER_PERMISSION_MASK,
                    now,
                    valid_until,
                    context.user_id,
                    reason,
                ],
            )
            .unwrap_or(0);

        if changed != 1 {
            return (StatusCode::CONFLICT, "Назначение уже существует").into_response();
        }

        id
    } else {
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
                 ?1, 1, 'group', ?2, ?3, '[]',
                 ?4, ?5, 'active', ?6, ?7, ?7,
                 ?4, ?4
             )",
            params![
                form.user_id,
                form.group_scope_id,
                HELPER_PERMISSION_MASK,
                now,
                valid_until,
                context.user_id,
                reason,
            ],
        );

        if inserted.unwrap_or(0) != 1 {
            return (StatusCode::CONFLICT, "Не удалось создать назначение").into_response();
        }

        transaction.last_insert_rowid()
    };

    let new_state = serde_json::json!({
        "assignment_id": assignment_id,
        "user_id": form.user_id.to_string(),
        "role_level": 1,
        "scope_type": "group",
        "scope_id": form.group_scope_id,
        "scope_name": group_name,
        "permission_mask": HELPER_PERMISSION_MASK,
        "valid_from": now,
        "valid_until": valid_until,
        "status": "active"
    })
    .to_string();

    if append_audit(
        &transaction,
        &context,
        &session_public_id,
        "city_helper_created",
        assignment_id,
        "{}",
        &new_state,
        reason,
        &ip,
        &user_agent,
        now,
    )
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось записать защищённый аудит",
        )
            .into_response();
    }

    let notification = transaction.execute(
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
             ?1, NULL, 'admin_assignment',
             'Назначение помощником',
             ?2, 0, ?3
         )",
        params![
            form.user_id,
            format!(
                "Вы назначены помощником группы «{}» на {} дней.",
                group_name, form.duration_days
            ),
            now,
        ],
    );

    if notification.unwrap_or(0) != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось создать уведомление",
        )
            .into_response();
    }

    let _ = transaction.execute(
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
             'city_helper_created',
             'info', ?4, ?5, ?6
         )",
        params![
            form.user_id,
            assignment_id,
            session_public_id,
            ip,
            user_agent,
            format!(
                "group_scope_id={}; assigned_by={}",
                form.group_scope_id, context.user_id
            ),
        ],
    );

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось сохранить назначение",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/city/helpers?created=1")],
    )
        .into_response()
}

pub async fn city_helper_lifecycle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((assignment_id, raw_action)): Path<(i64, String)>,
    Form(form): Form<CityHelperLifecycleForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let action = match HelperAction::parse(&raw_action) {
        Some(action) => action,
        None => {
            return (StatusCode::BAD_REQUEST, "Неизвестное действие").into_response();
        }
    };

    if assignment_id <= 0 || !reason_is_valid(&form.reason) {
        return (
            StatusCode::BAD_REQUEST,
            "Некорректное назначение или причина",
        )
            .into_response();
    }

    let context = match city_manager_context(&state, &headers) {
        Ok(context) => context,
        Err(response) => return *response,
    };

    let session_public_id = match valid_admin_session_public_id(
        &state,
        &headers,
        context.user_id,
        context.assignment_id,
    ) {
        Some(value) => value,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Административная сессия недействительна",
            )
                .into_response();
        }
    };

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

    let now = unix_now();
    let reason = form.reason.trim();
    let (ip, user_agent) = request_metadata(&headers);

    let transaction = match connection.transaction() {
        Ok(transaction) => transaction,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Не удалось открыть транзакцию",
            )
                .into_response();
        }
    };

    let target = transaction
        .query_row(
            "SELECT
                 assignment.user_id,
                 assignment.status,
                 assignment.valid_until,
                 group_scope.display_name,
                 group_scope.id
             FROM admin_assignments AS assignment
             JOIN geographic_scopes AS group_scope
               ON group_scope.id = assignment.scope_id
             WHERE assignment.id = ?1
               AND assignment.role_level = 1
               AND assignment.scope_type = 'group'
               AND group_scope.parent_scope_id = ?2
               AND group_scope.is_active = 1",
            params![assignment_id, context.scope_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()
        .unwrap_or(None);

    let Some((target_user_id, current_status, valid_until, group_name, group_scope_id)) = target
    else {
        record_denied_access(
            &state,
            context.user_id,
            "city_helper_assignment_scope_denied",
            &format!("assignment_id={assignment_id}"),
        );

        return (
            StatusCode::FORBIDDEN,
            "Назначение находится вне вашего города",
        )
            .into_response();
    };

    let transition_allowed = match action {
        HelperAction::Suspend => current_status == "active",
        HelperAction::Restore => {
            current_status == "suspended" && valid_until.is_none_or(|value| value > now)
        }
        HelperAction::Revoke => {
            matches!(current_status.as_str(), "active" | "suspended" | "pending")
        }
    };

    if !transition_allowed {
        return (StatusCode::CONFLICT, "Такой переход состояния недоступен").into_response();
    }

    if action == HelperAction::Restore {
        let conflict = transaction
            .query_row(
                "SELECT COUNT(*)
                 FROM admin_assignments
                 WHERE user_id = ?1
                   AND id <> ?2
                   AND status = 'active'
                   AND valid_from <= ?3
                   AND (
                       valid_until IS NULL
                       OR valid_until > ?3
                   )",
                params![target_user_id, assignment_id, now],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);

        if conflict != 0 {
            return (
                StatusCode::CONFLICT,
                "У пользователя уже есть другое назначение",
            )
                .into_response();
        }
    }

    let next_status = action.next_status();

    let changed = transaction
        .execute(
            "UPDATE admin_assignments
             SET status = ?2,
                 last_change_reason = ?3,
                 updated_at = ?4
             WHERE id = ?1
               AND status = ?5",
            params![assignment_id, next_status, reason, now, current_status,],
        )
        .unwrap_or(0);

    if changed != 1 {
        return (StatusCode::CONFLICT, "Назначение уже было изменено").into_response();
    }

    let revoked_sessions = if matches!(action, HelperAction::Suspend | HelperAction::Revoke) {
        transaction
            .execute(
                "UPDATE admin_sessions
                 SET revoked_at = ?2,
                     revoked_by_user_id = ?3,
                     revoke_reason = ?4
                 WHERE assignment_id = ?1
                   AND revoked_at IS NULL",
                params![assignment_id, now, context.user_id, reason,],
            )
            .unwrap_or(0)
    } else {
        0
    };

    let old_state = serde_json::json!({
        "assignment_id": assignment_id,
        "user_id": target_user_id.to_string(),
        "scope_id": group_scope_id,
        "scope_name": group_name,
        "status": current_status,
        "valid_until": valid_until
    })
    .to_string();

    let new_state = serde_json::json!({
        "assignment_id": assignment_id,
        "user_id": target_user_id.to_string(),
        "scope_id": group_scope_id,
        "scope_name": group_name,
        "status": next_status,
        "valid_until": valid_until,
        "revoked_sessions": revoked_sessions
    })
    .to_string();

    if append_audit(
        &transaction,
        &context,
        &session_public_id,
        action.event_type(),
        assignment_id,
        &old_state,
        &new_state,
        reason,
        &ip,
        &user_agent,
        now,
    )
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось записать защищённый аудит",
        )
            .into_response();
    }

    let notification_title = match action {
        HelperAction::Suspend => "Назначение помощника приостановлено",
        HelperAction::Restore => "Назначение помощника восстановлено",
        HelperAction::Revoke => "Назначение помощника отозвано",
    };

    let notification = transaction.execute(
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
             ?1, NULL, 'admin_assignment',
             ?2, ?3, 0, ?4
         )",
        params![
            target_user_id,
            notification_title,
            format!("Группа «{}». Причина: {}", group_name, reason),
            now,
        ],
    );

    if notification.unwrap_or(0) != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось создать уведомление",
        )
            .into_response();
    }

    let _ = transaction.execute(
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
             ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
         )",
        params![
            target_user_id,
            assignment_id,
            session_public_id,
            action.event_type(),
            if action == HelperAction::Revoke {
                "high"
            } else {
                "warning"
            },
            ip,
            user_agent,
            format!(
                "manager={}; revoked_sessions={}; reason={}",
                context.user_id, revoked_sessions, reason
            ),
        ],
    );

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось сохранить действие",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/city/helpers?updated=1")],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_assignment_policy_is_strict() {
        assert_eq!(HELPER_PERMISSION_MASK, 15);
        assert!(duration_is_valid(1));
        assert!(duration_is_valid(365));
        assert!(!duration_is_valid(0));
        assert!(!duration_is_valid(366));
        assert!(reason_is_valid("Назначение помощником группы"));
        assert!(!reason_is_valid("нет"));
        assert!(!reason_is_valid("Причина\nс переводом"));
    }

    #[test]
    fn helper_lifecycle_is_strict() {
        assert_eq!(HelperAction::parse("suspend"), Some(HelperAction::Suspend));
        assert_eq!(HelperAction::parse("restore"), Some(HelperAction::Restore));
        assert_eq!(HelperAction::parse("revoke"), Some(HelperAction::Revoke));
        assert_eq!(HelperAction::parse("delete"), None);
    }

    #[test]
    fn territorial_checks_are_present() {
        let source = include_str!("city_helper_actions.rs");

        assert!(source.contains("group_scope.parent_scope_id = ?2"));
        assert!(source.contains("city_helper_assignment_scope_denied"));
        assert!(source.contains("AdminPermission::AssistantsManage"));
    }
}
