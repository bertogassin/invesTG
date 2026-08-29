use super::admin_access::{
    load_admin_context, record_denied_access, valid_admin_session_public_id, AdminContext,
    AdminPermission,
};
use super::auth::verify_authenticated_user;
use super::common::request_is_cross_site;
use crate::state::app_state::AppState;
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use rusqlite::OptionalExtension;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const STEP_UP_MAX_AGE_SECONDS: i64 = 900;
const MIN_DURATION_DAYS: i64 = 1;
const MAX_DURATION_DAYS: i64 = 365;

#[derive(Debug, Deserialize)]
pub struct AssignmentLifecycleForm {
    reason: String,
    duration_days: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecycleAction {
    Suspend,
    Restore,
    Revoke,
    ChangeExpiry,
}

impl LifecycleAction {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "suspend" => Some(Self::Suspend),
            "restore" => Some(Self::Restore),
            "revoke" => Some(Self::Revoke),
            "change-expiry" => Some(Self::ChangeExpiry),
            _ => None,
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Suspend => "admin_assignment_suspended",
            Self::Restore => "admin_assignment_restored",
            Self::Revoke => "admin_assignment_revoked",
            Self::ChangeExpiry => "admin_assignment_expiry_changed",
        }
    }

    fn notification_title(self) -> &'static str {
        match self {
            Self::Suspend => "Назначение приостановлено",
            Self::Restore => "Назначение восстановлено",
            Self::Revoke => "Назначение отозвано",
            Self::ChangeExpiry => "Срок назначения изменён",
        }
    }

    fn severity(self) -> &'static str {
        match self {
            Self::Revoke => "high",
            Self::Suspend | Self::Restore | Self::ChangeExpiry => "warning",
        }
    }
}

#[derive(Debug)]
struct TargetAssignment {
    id: i64,
    user_id: i64,
    role_level: i64,
    scope_type: String,
    scope_id: i64,
    scope_name: String,
    permission_mask: i64,
    valid_from: i64,
    valid_until: Option<i64>,
    status: String,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn reason_is_valid(reason: &str) -> bool {
    let length = reason.chars().count();

    (5..=500).contains(&length) && !reason.chars().any(char::is_control)
}

fn duration_is_valid(days: i64) -> bool {
    (MIN_DURATION_DAYS..=MAX_DURATION_DAYS).contains(&days)
}

fn transition_is_valid(action: LifecycleAction, current_status: &str) -> bool {
    match action {
        LifecycleAction::Suspend => current_status == "active",
        LifecycleAction::Restore => current_status == "suspended",
        LifecycleAction::Revoke => {
            current_status == "active"
                || current_status == "suspended"
                || current_status == "pending"
        }
        LifecycleAction::ChangeExpiry => {
            current_status == "active" || current_status == "suspended"
        }
    }
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
            "admin_lifecycle_access_denied",
            "Нет активного административного назначения",
        );

        Box::new((StatusCode::NOT_FOUND, "404").into_response())
    })?;

    if !context.is_owner() || !context.has_permission(AdminPermission::GlobalAdminsManage) {
        record_denied_access(
            state,
            authenticated.user_id,
            "admin_lifecycle_permission_denied",
            "Недостаточно прав для управления назначениями",
        );

        return Err(Box::new(
            (StatusCode::FORBIDDEN, "Доступ запрещён").into_response(),
        ));
    }

    let session_public_id =
        valid_admin_session_public_id(state, headers, context.user_id, context.assignment_id)
            .ok_or_else(|| {
                Box::new(
                    (
                        StatusCode::UNAUTHORIZED,
                        "Административная сессия недействительна",
                    )
                        .into_response(),
                )
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

fn target_state_json(target: &TargetAssignment, status: &str, valid_until: Option<i64>) -> String {
    serde_json::json!({
        "assignment_id": target.id,
        "user_id": target.user_id.to_string(),
        "role_level": target.role_level,
        "scope_type": target.scope_type,
        "scope_id": target.scope_id,
        "scope_name": target.scope_name,
        "permission_mask": target.permission_mask,
        "valid_from": target.valid_from,
        "valid_until": valid_until,
        "status": status
    })
    .to_string()
}

pub async fn manage_admin_assignment(
    State(state): State<AppState>,
    Path((assignment_id, raw_action)): Path<(i64, String)>,
    headers: HeaderMap,
    Form(payload): Form<AssignmentLifecycleForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён CSRF-защитой").into_response();
    }

    let action = match LifecycleAction::parse(&raw_action) {
        Some(action) => action,
        None => {
            return (StatusCode::BAD_REQUEST, "Неизвестное действие").into_response();
        }
    };

    if assignment_id <= 0 {
        return (StatusCode::BAD_REQUEST, "Некорректное назначение").into_response();
    }

    let reason = payload.reason.trim();

    if !reason_is_valid(reason) {
        return (
            StatusCode::BAD_REQUEST,
            "Причина должна содержать от 5 до 500 символов",
        )
            .into_response();
    }

    let requested_days = if action == LifecycleAction::ChangeExpiry {
        match payload.duration_days {
            Some(days) if duration_is_valid(days) => Some(days),
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Срок должен составлять от 1 до 365 дней",
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let (context, session_public_id) = match owner_context_and_session(&state, &headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };

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
                    'admin_lifecycle_step_up_required',
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

    let target: Option<TargetAssignment> = transaction
        .query_row(
            "SELECT
                assignment.id,
                assignment.user_id,
                assignment.role_level,
                assignment.scope_type,
                assignment.scope_id,
                scope.display_name,
                assignment.permission_mask,
                assignment.valid_from,
                assignment.valid_until,
                assignment.status
             FROM admin_assignments AS assignment
             JOIN geographic_scopes AS scope
               ON scope.id = assignment.scope_id
             WHERE assignment.id = ?1",
            rusqlite::params![assignment_id],
            |row| {
                Ok(TargetAssignment {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    role_level: row.get(2)?,
                    scope_type: row.get(3)?,
                    scope_id: row.get(4)?,
                    scope_name: row.get(5)?,
                    permission_mask: row.get(6)?,
                    valid_from: row.get(7)?,
                    valid_until: row.get(8)?,
                    status: row.get(9)?,
                })
            },
        )
        .optional()
        .unwrap_or(None);

    let Some(target) = target else {
        return (StatusCode::NOT_FOUND, "Назначение не найдено").into_response();
    };

    if target.role_level == 5 || target.user_id == context.user_id {
        return (
            StatusCode::FORBIDDEN,
            "Назначение владельца изменять нельзя",
        )
            .into_response();
    }

    if !transition_is_valid(action, &target.status) {
        return (
            StatusCode::CONFLICT,
            "Действие недоступно для текущего состояния",
        )
            .into_response();
    }

    if action == LifecycleAction::Restore {
        let conflicting_assignment: i64 = transaction
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
                rusqlite::params![target.user_id, target.id, now],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if conflicting_assignment != 0 {
            return (
                StatusCode::CONFLICT,
                "У пользователя уже есть другое активное назначение",
            )
                .into_response();
        }

        if target.valid_until.is_some_and(|value| value <= now) {
            return (
                StatusCode::CONFLICT,
                "Сначала установите новый срок назначения",
            )
                .into_response();
        }
    }

    let new_status = match action {
        LifecycleAction::Suspend => "suspended",
        LifecycleAction::Restore => "active",
        LifecycleAction::Revoke => "revoked",
        LifecycleAction::ChangeExpiry => target.status.as_str(),
    };

    let new_valid_until = requested_days
        .map(|days| now.saturating_add(days.saturating_mul(86_400)))
        .or(target.valid_until);

    let changed = match action {
        LifecycleAction::ChangeExpiry => transaction.execute(
            "UPDATE admin_assignments
             SET valid_until = ?2,
                 last_change_reason = ?3,
                 updated_at = ?4
             WHERE id = ?1
               AND role_level BETWEEN 1 AND 4
               AND status IN ('active','suspended')",
            rusqlite::params![target.id, new_valid_until, reason, now],
        ),
        _ => transaction.execute(
            "UPDATE admin_assignments
             SET status = ?2,
                 last_change_reason = ?3,
                 updated_at = ?4
             WHERE id = ?1
               AND role_level BETWEEN 1 AND 4
               AND status = ?5",
            rusqlite::params![target.id, new_status, reason, now, target.status],
        ),
    }
    .unwrap_or(0);

    if changed != 1 {
        return (StatusCode::CONFLICT, "Назначение уже было изменено").into_response();
    }

    let sessions_revoked =
        if action == LifecycleAction::Suspend || action == LifecycleAction::Revoke {
            transaction
                .execute(
                    "UPDATE admin_sessions
                 SET revoked_at = ?2,
                     revoked_by_user_id = ?3,
                     revoke_reason = ?4
                 WHERE assignment_id = ?1
                   AND revoked_at IS NULL",
                    rusqlite::params![target.id, now, context.user_id, reason],
                )
                .unwrap_or(0)
        } else {
            0
        };

    let old_state = target_state_json(&target, &target.status, target.valid_until);

    let new_state = target_state_json(&target, new_status, new_valid_until);

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
        "{}:{}:{}:{}:{}:{}",
        context.user_id,
        target.id,
        action.event_type(),
        now,
        nanos,
        sessions_revoked
    );

    let event_public_id = hex::encode(Sha256::digest(event_seed.as_bytes()))[..32].to_string();

    let target_id = target.id.to_string();

    let hash_payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        previous_hash,
        event_public_id,
        context.user_id,
        action.event_type(),
        target_id,
        old_state,
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
            ?7, 'admin_assignment',
            ?8, ?9, ?10, ?11, ?12, ?13, ?14,
            'success', '[]', ?15, ?16, ?17
         )",
        rusqlite::params![
            event_public_id,
            context.user_id,
            context.assignment_id,
            context.level.number(),
            context.scope_type,
            context.scope_id,
            action.event_type(),
            target_id,
            old_state,
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

    let notification_message = match action {
        LifecycleAction::Suspend => format!(
            "Ваше административное назначение на территории «{}» приостановлено. Причина: {}",
            target.scope_name, reason
        ),
        LifecycleAction::Restore => format!(
            "Ваше административное назначение на территории «{}» восстановлено.",
            target.scope_name
        ),
        LifecycleAction::Revoke => format!(
            "Ваше административное назначение на территории «{}» отозвано. Причина: {}",
            target.scope_name, reason
        ),
        LifecycleAction::ChangeExpiry => format!(
            "Срок административного назначения на территории «{}» изменён. Новый срок: {} дней.",
            target.scope_name,
            requested_days.unwrap_or(0)
        ),
    };

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
            ?2, ?3, 0, ?4
         )",
        rusqlite::params![
            target.user_id,
            action.notification_title(),
            notification_message,
            now
        ],
    );

    if notification_inserted.unwrap_or(0) != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось создать уведомление",
        )
            .into_response();
    }

    let security_details = format!(
        "{}; revoked_sessions={}; reason={}",
        action.event_type(),
        sessions_revoked,
        reason
    );

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
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
         )",
        rusqlite::params![
            target.user_id,
            target.id,
            session_public_id,
            action.event_type(),
            action.severity(),
            ip_address,
            user_agent,
            security_details
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
        [(header::LOCATION, "/app/center/administrators?updated=1")],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_actions_are_parsed_strictly() {
        assert_eq!(
            LifecycleAction::parse("suspend"),
            Some(LifecycleAction::Suspend)
        );
        assert_eq!(
            LifecycleAction::parse("restore"),
            Some(LifecycleAction::Restore)
        );
        assert_eq!(
            LifecycleAction::parse("revoke"),
            Some(LifecycleAction::Revoke)
        );
        assert_eq!(
            LifecycleAction::parse("change-expiry"),
            Some(LifecycleAction::ChangeExpiry)
        );
        assert_eq!(LifecycleAction::parse("delete"), None);
    }

    #[test]
    fn lifecycle_transitions_are_strict() {
        assert!(transition_is_valid(LifecycleAction::Suspend, "active"));
        assert!(!transition_is_valid(LifecycleAction::Suspend, "suspended"));
        assert!(transition_is_valid(LifecycleAction::Restore, "suspended"));
        assert!(!transition_is_valid(LifecycleAction::Restore, "revoked"));
        assert!(transition_is_valid(LifecycleAction::Revoke, "active"));
        assert!(!transition_is_valid(LifecycleAction::Revoke, "revoked"));
    }

    #[test]
    fn lifecycle_reason_policy_is_enforced() {
        assert!(reason_is_valid("Изменение ответственности администратора"));
        assert!(!reason_is_valid("нет"));
        assert!(!reason_is_valid("Причина\nс переводом строки"));
    }

    #[test]
    fn lifecycle_duration_policy_is_enforced() {
        assert!(duration_is_valid(1));
        assert!(duration_is_valid(365));
        assert!(!duration_is_valid(0));
        assert!(!duration_is_valid(366));
    }
}
