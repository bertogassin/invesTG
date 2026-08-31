use super::admin_access::{
    create_admin_session, load_admin_context, record_denied_access, verify_admin_session,
    AdminPermission,
};
use super::auth::verify_authenticated_user;
use super::common::{csrf_rejected_response, request_is_cross_site};
use crate::state::app_state::AppState;
use crate::web::templates::{admin_ops_page, escape_html, workflow_status_label_or_raw};
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;

const EVENT_LIMIT: i64 = 80;
const RISK_LIMIT: i64 = 40;
const REPORT_LIMIT: i64 = 50;

struct GroupInfo {
    city_id: i64,
    stable_key: String,
    city_name: String,
    target_name: String,
    telegram_chat_id: i64,
    telegram_url: String,
}

struct SecurityEvent {
    user_id: i64,
    message_id: i64,
    action: String,
    reason: String,
    risk_score: i64,
    risk_level: String,
    mute_seconds: i64,
    success: i64,
    created_at: i64,
}

struct RiskUser {
    user_id: i64,
    risk_score: i64,
    critical_count: i64,
    updated_at: i64,
}

struct GroupReport {
    id: i64,
    resource_id: i64,
    resource_title: String,
    reason: String,
    status: String,
    created_at: i64,
}

fn action_title(value: &str) -> &'static str {
    match value {
        "WARNING" => "Предупреждение",
        "WARNING_AFTER_DELETE" => "Предупреждение после удаления",
        "WARNING_AFTER_MUTE" => "Предупреждение после ограничения",
        "DELETE_MESSAGE" => "Сообщение удалено",
        "MUTE_10_MINUTES" => "Ограничение на 10 минут",
        "MUTE_1_HOUR" => "Ограничение на 1 час",
        "MUTE_24_HOURS" => "Ограничение на 24 часа",
        _ => "Событие защиты",
    }
}

fn render_page(
    group: &GroupInfo,
    events: &[SecurityEvent],
    risks: &[RiskUser],
    reports: &[GroupReport],
) -> String {
    let active_events = events.iter().filter(|event| event.success == 1).count();

    let critical_users = risks.iter().filter(|risk| risk.critical_count > 0).count();

    let pending_reports = reports
        .iter()
        .filter(|report| report.status == "pending")
        .count();

    let event_cards = if events.is_empty() {
        r#"<div class="empty">
            Событий защиты пока нет.
        </div>"#
            .to_string()
    } else {
        events
            .iter()
            .map(|event| {
                let status = if event.success == 1 {
                    "Выполнено"
                } else {
                    "Ошибка"
                };

                format!(
                    r#"<article class="event-card">
                        <div class="event-head">
                            <strong>{action}</strong>
                            <span class="risk {risk_class}">
                                {risk_level} · {risk_score}
                            </span>
                        </div>
                        <p>{reason}</p>
                        <div class="meta">
                            <span>ID пользователя: {user_id}</span>
                            <span>ID сообщения: {message_id}</span>
                            <span>Заглушка: {mute_seconds} с</span>
                            <span>{status}</span>
                            <span>время: {created_at}</span>
                        </div>
                    </article>"#,
                    action = escape_html(action_title(&event.action)),
                    risk_class = escape_html(&event.risk_level.to_lowercase()),
                    risk_level = escape_html(&event.risk_level),
                    risk_score = event.risk_score,
                    reason = escape_html(&event.reason),
                    user_id = event.user_id,
                    message_id = event.message_id,
                    mute_seconds = event.mute_seconds,
                    status = status,
                    created_at = event.created_at,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let risk_rows = if risks.is_empty() {
        r#"<div class="empty">
            Активных записей риска пока нет.
        </div>"#
            .to_string()
    } else {
        risks
            .iter()
            .map(|risk| {
                format!(
                    r#"<div class="risk-row">
                        <div>
                            <strong>ID пользователя {user_id}</strong>
                            <small>Обновлено: {updated_at}</small>
                        </div>
                        <div class="risk-values">
                            <span>Риск: {risk_score}</span>
                            <span>Критических: {critical_count}</span>
                        </div>
                    </div>"#,
                    user_id = risk.user_id,
                    updated_at = risk.updated_at,
                    risk_score = risk.risk_score,
                    critical_count = risk.critical_count,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let report_cards = if reports.is_empty() {
        r#"<div class="empty">
            Жалоб на ресурсы этого города пока нет.
        </div>"#
            .to_string()
    } else {
        reports
            .iter()
            .map(|report| {
                let actions = if report.status == "closed" {
                    String::new()
                } else {
                    format!(
                        r#"<div class="report-actions">
                            <form method="post"
                                  action="/app/center/group/report/{id}">
                                <input name="reason"
                                       minlength="5"
                                       maxlength="500"
                                       required
                                       placeholder="Причина решения">
                                <div class="action-buttons">
                                    <button name="action"
                                            value="close"
                                            class="close">
                                        Закрыть
                                    </button>
                                    <button name="action"
                                            value="escalate"
                                            class="escalate">
                                        Передать выше
                                    </button>
                                    <button name="action"
                                            value="reject"
                                            class="reject">
                                        Отклонить материал
                                    </button>
                                </div>
                            </form>
                        </div>"#,
                        id = report.id,
                    )
                };

                format!(
                    r#"<article class="report-card">
                        <div class="event-head">
                            <strong>{title}</strong>
                            <span class="report-status">
                                {status}
                            </span>
                        </div>
                        <p>{reason}</p>
                        <div class="meta">
                            <span>Жалоба #{id}</span>
                            <span>Ресурс #{resource_id}</span>
                            <span>время: {created_at}</span>
                        </div>
                        {actions}
                    </article>"#,
                    title = escape_html(&report.resource_title),
                    status = escape_html(&workflow_status_label_or_raw(&report.status)),
                    reason = escape_html(&report.reason),
                    id = report.id,
                    resource_id = report.resource_id,
                    created_at = report.created_at,
                    actions = actions,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let telegram_link = if group.telegram_url.starts_with("https://t.me/") {
        format!(
            r#"<a class="telegram"
                   href="{}"
                   target="_blank"
                   rel="noopener noreferrer">
                    Открыть группу в Telegram
               </a>"#,
            escape_html(&group.telegram_url),
        )
    } else {
        String::new()
    };

    let content = format!(
        r#"
    <div class="topbar">
        <a class="back" href="/app/me">← Личный кабинет</a>
        <span class="protected">УРОВЕНЬ 1 · ТОЛЬКО ГРУППА</span>
    </div>

    <section class="hero">
        <div class="kicker">ResursMap · Помощник группы</div>
        <h1>{city_name}</h1>
        <p>
            {target_name}. Здесь отображаются только события,
            риски и жалобы вашей назначенной группы.
        </p>

        <div class="group-meta">
            <span>{stable_key}</span>
            <span>ID чата: {chat_id}</span>
            <span>ID города: {city_id}</span>
        </div>

        {telegram_link}
    </section>

    <section class="metrics">
        <div class="metric">
            <strong>{active_events}</strong>
            <span>успешных действий защиты</span>
        </div>
        <div class="metric">
            <strong>{critical_users}</strong>
            <span>пользователей с критическими событиями</span>
        </div>
        <div class="metric">
            <strong>{pending_reports}</strong>
            <span>открытых жалоб города</span>
        </div>
    </section>

    <div class="section-head">
        <h2>События защиты</h2>
        <span>Последние {event_count}</span>
    </div>
    <section class="grid">{event_cards}</section>

    <div class="section-head">
        <h2>Риск пользователей</h2>
        <span>Только текущая группа</span>
    </div>
    <section class="grid">{risk_rows}</section>

    <div class="section-head">
        <h2>Жалобы города</h2>
        <span>Последние {report_count}</span>
    </div>
    <section class="grid">{report_cards}</section>
"#,
        city_name = escape_html(&group.city_name),
        target_name = escape_html(&group.target_name),
        stable_key = escape_html(&group.stable_key),
        chat_id = group.telegram_chat_id,
        city_id = group.city_id,
        telegram_link = telegram_link,
        active_events = active_events,
        critical_users = critical_users,
        pending_reports = pending_reports,
        event_count = events.len(),
        report_count = reports.len(),
        event_cards = event_cards,
        risk_rows = risk_rows,
        report_cards = report_cards,
    );

    admin_ops_page("Помощник группы · ResursMap", &content)
}

pub async fn group_helper_panel(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Требуется вход в аккаунт ResursMap",
            )
                .into_response();
        }
    };

    let context = match load_admin_context(&state, user.user_id) {
        Some(context) => context,
        None => {
            record_denied_access(
                &state,
                user.user_id,
                "group_helper_access_denied",
                "Нет активного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if context.level.number() != 1
        || context.scope_type != "group"
        || !context.has_permission(AdminPermission::ModerationReview)
        || !context.has_permission(AdminPermission::ComplaintsReview)
    {
        record_denied_access(
            &state,
            user.user_id,
            "group_helper_permission_denied",
            "Требуется уровень 1 и область группы",
        );

        return (StatusCode::FORBIDDEN, "Кабинет помощника недоступен").into_response();
    }

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

        let cookie = match HeaderValue::from_str(&cookie) {
            Ok(cookie) => cookie,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Ошибка административной сессии",
                )
                    .into_response();
            }
        };

        let mut response = StatusCode::SEE_OTHER.into_response();

        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_static("/app/center/group"),
        );

        response.headers_mut().insert(header::SET_COOKIE, cookie);

        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
        );

        return response;
    }

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "База данных недоступна").into_response();
        }
    };

    let group = connection.query_row(
        "SELECT
             city.id,
             city.stable_key,
             city.name_ru,
             target.target_name,
             target.telegram_chat_id,
             target.telegram_url
         FROM geographic_scopes AS group_scope
         JOIN geographic_scopes AS city_scope
           ON city_scope.id = group_scope.parent_scope_id
          AND city_scope.scope_type = 'city'
          AND city_scope.is_active = 1
         JOIN geo_cities AS city
           ON city_scope.external_key =
              'city:' || city.stable_key
         JOIN city_publication_targets AS target
           ON target.city_id = city.id
          AND target.target_kind = 'group'
          AND target.is_active = 1
          AND target.telegram_chat_id < 0
         WHERE group_scope.id = ?1
           AND group_scope.scope_type = 'group'
           AND group_scope.is_active = 1
         ORDER BY target.id
         LIMIT 1",
        params![context.scope_id],
        |row| {
            Ok(GroupInfo {
                city_id: row.get(0)?,
                stable_key: row.get(1)?,
                city_name: row.get(2)?,
                target_name: row.get(3)?,
                telegram_chat_id: row.get(4)?,
                telegram_url: row.get(5)?,
            })
        },
    );

    let group = match group {
        Ok(group) => group,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                "Назначенная Telegram-группа не найдена",
            )
                .into_response();
        }
    };

    let events = connection
        .prepare(
            "SELECT
                 user_id,
                 message_id,
                 action,
                 reason,
                 risk_score,
                 risk_level,
                 mute_seconds,
                 success,
                 created_at
             FROM security_moderation_audit
             WHERE chat_id = ?1
             ORDER BY created_at DESC, id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![group.telegram_chat_id, EVENT_LIMIT], |row| {
                    Ok(SecurityEvent {
                        user_id: row.get(0)?,
                        message_id: row.get(1)?,
                        action: row.get(2)?,
                        reason: row.get(3)?,
                        risk_score: row.get(4)?,
                        risk_level: row.get(5)?,
                        mute_seconds: row.get(6)?,
                        success: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let risks = connection
        .prepare(
            "SELECT
                 risk.user_id,
                 risk.risk_score,
                 COALESCE(moderation.critical_count, 0),
                 MAX(
                     risk.updated_at,
                     COALESCE(moderation.updated_at, 0)
                 )
             FROM security_risk_state AS risk
             LEFT JOIN security_moderation_state AS moderation
               ON moderation.chat_id = risk.chat_id
              AND moderation.user_id = risk.user_id
             WHERE risk.chat_id = ?1
             ORDER BY
                 risk.risk_score DESC,
                 moderation.critical_count DESC,
                 risk.updated_at DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![group.telegram_chat_id, RISK_LIMIT], |row| {
                    Ok(RiskUser {
                        user_id: row.get(0)?,
                        risk_score: row.get(1)?,
                        critical_count: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let reports = connection
        .prepare(
            "SELECT
                 report.id,
                 report.resource_id,
                 resource.title,
                 report.reason,
                 report.status,
                 report.created_at
             FROM resource_reports AS report
             JOIN resources AS resource
               ON resource.id = report.resource_id
             WHERE resource.city_id = ?1
             ORDER BY
                 CASE report.status
                     WHEN 'pending' THEN 0
                     ELSE 1
                 END,
                 report.created_at DESC,
                 report.id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![group.city_id, REPORT_LIMIT], |row| {
                    Ok(GroupReport {
                        id: row.get(0)?,
                        resource_id: row.get(1)?,
                        resource_title: row.get(2)?,
                        reason: row.get(3)?,
                        status: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let html = render_page(&group, &events, &risks, &reports);

    let mut response = Html(html).into_response();

    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );

    response
}

#[derive(Debug, Deserialize)]
pub struct HelperReportActionForm {
    action: String,
    reason: String,
}

fn report_action_is_valid(value: &str) -> bool {
    matches!(value, "close" | "escalate" | "reject")
}

fn action_reason_is_valid(value: &str) -> bool {
    let length = value.trim().chars().count();

    (5..=500).contains(&length) && !value.chars().any(char::is_control)
}

pub async fn group_helper_report_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<i64>,
    Form(form): Form<HelperReportActionForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if report_id <= 0
        || !report_action_is_valid(&form.action)
        || !action_reason_is_valid(&form.reason)
    {
        return (StatusCode::BAD_REQUEST, "Некорректное действие или причина").into_response();
    }

    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Требуется вход в аккаунт ResursMap",
            )
                .into_response();
        }
    };

    let context = match load_admin_context(&state, user.user_id) {
        Some(context) => context,
        None => {
            record_denied_access(
                &state,
                user.user_id,
                "group_helper_action_denied",
                "Нет активного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if context.level.number() != 1
        || context.scope_type != "group"
        || !context.has_permission(AdminPermission::ModerationReview)
        || !context.has_permission(AdminPermission::ComplaintsReview)
    {
        record_denied_access(
            &state,
            user.user_id,
            "group_helper_action_permission_denied",
            "Нет территориальных прав уровня 1",
        );

        return (StatusCode::FORBIDDEN, "Действие недоступно").into_response();
    }

    if !verify_admin_session(&state, &headers, context.user_id, context.assignment_id) {
        return (
            StatusCode::UNAUTHORIZED,
            "Административная сессия недействительна",
        )
            .into_response();
    }

    let mut connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "База данных недоступна").into_response();
        }
    };

    let scoped_report = connection
        .query_row(
            "SELECT
                 report.resource_id,
                 report.status,
                 city.stable_key
             FROM resource_reports AS report
             JOIN resources AS resource
               ON resource.id = report.resource_id
             JOIN geo_cities AS city
               ON city.id = resource.city_id
             JOIN geographic_scopes AS city_scope
               ON city_scope.scope_type = 'city'
              AND city_scope.external_key =
                  'city:' || city.stable_key
             JOIN geographic_scopes AS group_scope
               ON group_scope.parent_scope_id = city_scope.id
              AND group_scope.scope_type = 'group'
              AND group_scope.is_active = 1
             WHERE report.id = ?1
               AND group_scope.id = ?2",
            params![report_id, context.scope_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional();

    let (resource_id, report_status, stable_key) = match scoped_report {
        Ok(Some(report)) => report,
        Ok(None) => {
            record_denied_access(
                &state,
                context.user_id,
                "group_helper_report_scope_denied",
                &format!("report_id={report_id}"),
            );

            return (
                StatusCode::FORBIDDEN,
                "Жалоба находится вне назначенной группы",
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось проверить жалобу",
            )
                .into_response();
        }
    };

    if report_status == "closed" {
        return (StatusCode::CONFLICT, "Жалоба уже закрыта").into_response();
    }

    let reason = form.reason.trim();
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

    let result = match form.action.as_str() {
        "close" => transaction.execute(
            "UPDATE resource_reports
             SET status = 'closed',
                 updated_at = strftime('%s','now')
             WHERE id = ?1",
            params![report_id],
        ),

        "escalate" => transaction.execute(
            "UPDATE resource_reports
             SET status = 'escalated',
                 updated_at = strftime('%s','now')
             WHERE id = ?1",
            params![report_id],
        ),

        "reject" => {
            let resource_update = transaction.execute(
                "UPDATE resources
                 SET moderation_status = 'rejected',
                     rejection_reason = ?2,
                     is_verified = 0,
                     is_active = 0,
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                params![resource_id, format!("Решение помощника группы: {reason}"),],
            );

            if resource_update.is_err() {
                resource_update
            } else {
                transaction.execute(
                    "UPDATE resource_reports
                     SET status = 'closed',
                         updated_at = strftime('%s','now')
                     WHERE id = ?1",
                    params![report_id],
                )
            }
        }

        _ => unreachable!("validated action"),
    };

    if result.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось применить решение",
        )
            .into_response();
    }

    let _ = transaction.execute(
        "INSERT INTO moderation_actions (
             moderator_id,
             action_type,
             target_type,
             target_id,
             details
         )
         VALUES (
             ?1,
             ?2,
             'resource_report',
             ?3,
             ?4
         )",
        params![
            context.user_id,
            format!("GROUP_HELPER_{}", form.action.to_uppercase()),
            report_id,
            format!("city={stable_key}; resource_id={resource_id}; reason={reason}"),
        ],
    );

    let _ = transaction.execute(
        "INSERT INTO admin_security_events (
             user_id,
             assignment_id,
             event_type,
             severity,
             details
         )
         VALUES (
             ?1,
             ?2,
             'group_helper_report_action',
             'info',
             ?3
         )",
        params![
            context.user_id,
            context.assignment_id,
            format!(
                "action={}; report_id={}; city={}",
                form.action, report_id, stable_key
            ),
        ],
    );

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось сохранить решение",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/center/group")],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_action_policy_is_strict() {
        assert!(report_action_is_valid("close"));
        assert!(report_action_is_valid("escalate"));
        assert!(report_action_is_valid("reject"));
        assert!(!report_action_is_valid("delete"));
        assert!(!report_action_is_valid(""));

        assert!(action_reason_is_valid("Жалоба проверена"));
        assert!(!action_reason_is_valid("нет"));
        assert!(!action_reason_is_valid("Причина\nс переводом"));
    }

    #[test]
    fn security_actions_have_stable_titles() {
        assert_eq!(action_title("DELETE_MESSAGE"), "Сообщение удалено");

        assert_eq!(action_title("MUTE_10_MINUTES"), "Ограничение на 10 минут");

        assert_eq!(action_title("UNKNOWN"), "Событие защиты");
    }

    #[test]
    fn helper_limits_are_bounded() {
        assert_eq!(EVENT_LIMIT, 80);
        assert_eq!(RISK_LIMIT, 40);
        assert_eq!(REPORT_LIMIT, 50);
    }
}
