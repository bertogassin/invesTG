use super::admin_access::{
    load_admin_context, record_denied_access, verify_admin_session, AdminPermission,
};
use super::auth::verify_authenticated_user;
use crate::state::app_state::AppState;
use crate::web::templates::{
    render_admin_administrators, AdminAdministratorRow, AdminAdministratorsData, AdminSessionRow,
};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};

pub async fn administrators_panel(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let authenticated_user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Требуется вход в аккаунт ResursMap",
            )
                .into_response();
        }
    };

    let context = match load_admin_context(&state, authenticated_user.user_id) {
        Some(context) => context,
        None => {
            record_denied_access(
                &state,
                authenticated_user.user_id,
                "admin_directory_access_denied",
                "Нет активного административного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if !context.is_owner()
        || !context.has_permission(AdminPermission::GlobalAdminsManage)
        || !context.has_permission(AdminPermission::AuditRead)
    {
        record_denied_access(
            &state,
            authenticated_user.user_id,
            "admin_directory_permission_denied",
            "Недостаточно прав для просмотра администраторов",
        );

        return (StatusCode::FORBIDDEN, "Доступ запрещён").into_response();
    }

    if !verify_admin_session(&state, &headers, context.user_id, context.assignment_id) {
        let mut response = StatusCode::SEE_OTHER.into_response();

        response
            .headers_mut()
            .insert(header::LOCATION, HeaderValue::from_static("/app/center"));

        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
        );

        return response;
    }

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let mut administrators = Vec::new();

    let administrator_sql = "
        SELECT
            aa.id,
            aa.user_id,
            aa.role_level,
            aa.scope_type,
            gs.display_name,
            aa.permission_mask,
            aa.status,
            aa.valid_from,
            aa.valid_until,
            COALESCE(p.username, ''),
            COALESCE(p.first_name, ''),
            COALESCE(p.last_name, ''),
            (
                SELECT COUNT(*)
                FROM admin_sessions AS sessions
                WHERE sessions.assignment_id = aa.id
                  AND sessions.revoked_at IS NULL
                  AND sessions.expires_at > strftime('%s','now')
            ),
            (
                SELECT COUNT(*)
                FROM admin_action_audit AS audit
                WHERE audit.assignment_id = aa.id
            )
        FROM admin_assignments AS aa
        JOIN geographic_scopes AS gs
          ON gs.id = aa.scope_id
        LEFT JOIN profiles AS p
          ON p.user_id = aa.user_id
        ORDER BY
            aa.role_level DESC,
            aa.status = 'active' DESC,
            aa.id
    ";

    if let Ok(mut statement) = connection.prepare(administrator_sql) {
        if let Ok(rows) = statement.query_map([], |row| {
            let permission_mask: i64 = row.get(5)?;
            let username: String = row.get(9)?;
            let first_name: String = row.get(10)?;
            let last_name: String = row.get(11)?;

            let full_name = format!("{first_name} {last_name}").trim().to_string();

            let display_name = if !full_name.is_empty() {
                full_name
            } else if !username.is_empty() {
                format!("@{username}")
            } else {
                format!("Пользователь #{}", row.get::<_, i64>(1)?)
            };

            Ok(AdminAdministratorRow {
                assignment_id: row.get(0)?,
                user_id: row.get(1)?,
                level: row.get(2)?,
                scope_type: row.get(3)?,
                territory: row.get(4)?,
                permission_count: permission_mask.count_ones(),
                status: row.get(6)?,
                valid_from: row.get(7)?,
                valid_until: row.get(8)?,
                display_name,
                username,
                active_sessions: row.get(12)?,
                audit_events: row.get(13)?,
            })
        }) {
            administrators.extend(rows.flatten());
        }
    }

    let mut sessions = Vec::new();

    let session_sql = "
        SELECT
            sessions.session_public_id,
            sessions.user_id,
            sessions.assignment_id,
            sessions.ip_address,
            sessions.user_agent,
            sessions.device_label,
            sessions.two_factor_verified,
            sessions.created_at,
            sessions.last_seen_at,
            sessions.expires_at,
            COALESCE(p.username, ''),
            COALESCE(p.first_name, ''),
            COALESCE(p.last_name, '')
        FROM admin_sessions AS sessions
        LEFT JOIN profiles AS p
          ON p.user_id = sessions.user_id
        WHERE sessions.revoked_at IS NULL
          AND sessions.expires_at > strftime('%s','now')
        ORDER BY sessions.last_seen_at DESC, sessions.id DESC
        LIMIT 100
    ";

    if let Ok(mut statement) = connection.prepare(session_sql) {
        if let Ok(rows) = statement.query_map([], |row| {
            let public_id: String = row.get(0)?;
            let username: String = row.get(10)?;
            let first_name: String = row.get(11)?;
            let last_name: String = row.get(12)?;

            let full_name = format!("{first_name} {last_name}").trim().to_string();

            let display_name = if !full_name.is_empty() {
                full_name
            } else if !username.is_empty() {
                format!("@{username}")
            } else {
                format!("Пользователь #{}", row.get::<_, i64>(1)?)
            };

            Ok(AdminSessionRow {
                public_id,
                user_id: row.get(1)?,
                assignment_id: row.get(2)?,
                ip_address: row.get(3)?,
                user_agent: row.get(4)?,
                device_label: row.get(5)?,
                two_factor_verified: row.get::<_, i64>(6)? == 1,
                created_at: row.get(7)?,
                last_seen_at: row.get(8)?,
                expires_at: row.get(9)?,
                display_name,
            })
        }) {
            sessions.extend(rows.flatten());
        }
    }

    let active_assignments = administrators
        .iter()
        .filter(|administrator| administrator.status == "active")
        .count();

    let expiring_assignments = administrators
        .iter()
        .filter(|administrator| {
            administrator.status == "active" && administrator.valid_until.is_some()
        })
        .count();

    let data = AdminAdministratorsData {
        viewer_name: context.level.title().to_string(),
        administrators,
        sessions,
        active_assignments,
        expiring_assignments,
    };

    drop(connection);

    let html = render_admin_administrators(data);
    let mut response = Html(html).into_response();

    let response_headers = response.headers_mut();

    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );
    response_headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response_headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    response_headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response_headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response_headers.insert(
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
