use super::admin_access::{
    load_admin_context, record_denied_access, valid_admin_session_public_id, AdminPermission,
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

#[derive(Debug, Deserialize)]
pub struct RevokeAdminSessionForm {
    reason: String,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn revoke_reason_is_valid(reason: &str) -> bool {
    let length = reason.chars().count();

    (5..=500).contains(&length) && !reason.chars().any(char::is_control)
}

fn session_id_is_valid(public_id: &str) -> bool {
    public_id.len() == 32 && public_id.bytes().all(|byte| byte.is_ascii_hexdigit())
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

pub async fn revoke_admin_session(
    State(state): State<AppState>,
    Path(target_session_public_id): Path<String>,
    headers: HeaderMap,
    Form(payload): Form<RevokeAdminSessionForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён CSRF-защитой").into_response();
    }

    let authenticated = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Требуется вход в аккаунт ResursMap",
            )
                .into_response();
        }
    };

    let context = match load_admin_context(&state, authenticated.user_id) {
        Some(context) => context,
        None => {
            record_denied_access(
                &state,
                authenticated.user_id,
                "admin_session_revoke_denied",
                "Нет активного административного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if !context.is_owner() || !context.has_permission(AdminPermission::SessionsRevoke) {
        record_denied_access(
            &state,
            authenticated.user_id,
            "admin_session_revoke_permission_denied",
            "Недостаточно прав для отзыва admin-сессии",
        );

        return (StatusCode::FORBIDDEN, "Доступ запрещён").into_response();
    }

    let current_session_public_id = match valid_admin_session_public_id(
        &state,
        &headers,
        context.user_id,
        context.assignment_id,
    ) {
        Some(public_id) => public_id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Административная сессия недействительна",
            )
                .into_response();
        }
    };

    if !session_id_is_valid(&target_session_public_id) {
        return (StatusCode::BAD_REQUEST, "Некорректный идентификатор сессии").into_response();
    }

    let reason = payload.reason.trim();

    if !revoke_reason_is_valid(reason) {
        return (
            StatusCode::BAD_REQUEST,
            "Причина должна содержать от 5 до 500 символов",
        )
            .into_response();
    }

    let now = unix_now();
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

    let step_up_valid: i64 = connection
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
                current_session_public_id,
                context.user_id,
                context.assignment_id,
                now,
                now - STEP_UP_MAX_AGE_SECONDS
            ],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if step_up_valid != 1 {
        let _ = connection.execute(
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
                'session_revoke_step_up_required',
                'high',
                ?4, ?5,
                'Требуется свежее подтверждение владельца'
             )",
            rusqlite::params![
                context.user_id,
                context.assignment_id,
                current_session_public_id,
                ip_address,
                user_agent
            ],
        );

        return (
            StatusCode::PRECONDITION_REQUIRED,
            [(header::LOCATION, "/app/center/security")],
            "Требуется повторное подтверждение владельца",
        )
            .into_response();
    }

    let transaction =
        match connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
            Ok(transaction) => transaction,
            Err(_) => {
                return (StatusCode::CONFLICT, "Не удалось заблокировать операцию").into_response();
            }
        };

    let target: Option<(i64, i64)> = transaction
        .query_row(
            "SELECT user_id, assignment_id
             FROM admin_sessions
             WHERE session_public_id = ?1
               AND revoked_at IS NULL
               AND expires_at > ?2",
            rusqlite::params![target_session_public_id, now],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .unwrap_or(None);

    let Some((target_user_id, target_assignment_id)) = target else {
        return (StatusCode::NOT_FOUND, "Активная сессия не найдена").into_response();
    };

    let changed = transaction
        .execute(
            "UPDATE admin_sessions
             SET revoked_at = ?2,
                 revoked_by_user_id = ?3,
                 revoke_reason = ?4
             WHERE session_public_id = ?1
               AND revoked_at IS NULL
               AND expires_at > ?2",
            rusqlite::params![target_session_public_id, now, context.user_id, reason],
        )
        .unwrap_or(0);

    if changed != 1 {
        return (StatusCode::CONFLICT, "Сессия уже изменена").into_response();
    }

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
        context.user_id, current_session_public_id, target_session_public_id, now, nanos
    );

    let event_public_id = hex::encode(Sha256::digest(event_seed.as_bytes()))[..32].to_string();

    let old_state = serde_json::json!({
        "session_public_id": target_session_public_id,
        "user_id": target_user_id.to_string(),
        "assignment_id": target_assignment_id,
        "revoked": false
    })
    .to_string();

    let new_state = serde_json::json!({
        "session_public_id": target_session_public_id,
        "user_id": target_user_id.to_string(),
        "assignment_id": target_assignment_id,
        "revoked": true,
        "revoked_at": now
    })
    .to_string();

    let hash_payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        previous_hash,
        event_public_id,
        context.user_id,
        "admin_session_revoked",
        target_session_public_id,
        old_state,
        new_state,
        now
    );

    let event_hash = hex::encode(Sha256::digest(hash_payload.as_bytes()));

    let audit_inserted = transaction
        .execute(
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
                'admin_session_revoked',
                'admin_session',
                ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                'success', '[]', ?14, ?15, ?16
             )",
            rusqlite::params![
                event_public_id,
                context.user_id,
                context.assignment_id,
                context.level.number(),
                context.scope_type,
                context.scope_id,
                target_session_public_id,
                old_state,
                new_state,
                reason,
                ip_address,
                user_agent,
                current_session_public_id,
                previous_hash,
                event_hash,
                now
            ],
        )
        .unwrap_or(0);

    if audit_inserted != 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось записать защищённый аудит",
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
            'admin_session_revoked',
            'warning',
            ?4, ?5, ?6
         )",
        rusqlite::params![
            target_user_id,
            target_assignment_id,
            target_session_public_id,
            ip_address,
            user_agent,
            reason
        ],
    );

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось завершить транзакцию",
        )
            .into_response();
    }

    if target_session_public_id == current_session_public_id {
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/app/center")]).into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/administrators?revoked=1")],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoke_reason_policy_is_enforced() {
        assert!(!revoke_reason_is_valid(""));
        assert!(!revoke_reason_is_valid("нет"));
        assert!(revoke_reason_is_valid("Подозрительное устройство"));
        assert!(!revoke_reason_is_valid("строка\nс переводом"));
    }

    #[test]
    fn session_public_id_policy_is_enforced() {
        assert!(session_id_is_valid("0123456789abcdef0123456789abcdef"));
        assert!(!session_id_is_valid("short"));
        assert!(!session_id_is_valid("0123456789abcdef0123456789abcdeg"));
    }
}
