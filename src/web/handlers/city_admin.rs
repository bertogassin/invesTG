use super::admin_access::{
    create_admin_session, load_admin_context, record_denied_access, scope_is_authorized,
    verify_admin_session, AdminPermission,
};
use super::auth::verify_authenticated_user;
use crate::state::app_state::AppState;
use crate::web::templates::escape_html;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rusqlite::params;

const PLATFORM_LIMIT: i64 = 30;
const HELPER_LIMIT: i64 = 40;
const REPORT_LIMIT: i64 = 40;
const RESOURCE_LIMIT: i64 = 40;
const SECURITY_LIMIT: i64 = 60;

struct CityInfo {
    id: i64,
    stable_key: String,
    name: String,
    native_name: String,
    country_name: String,
    continent_name: String,
}

struct CityMetrics {
    resources: i64,
    pending_resources: i64,
    open_reports: i64,
    active_platforms: i64,
    helpers: i64,
    security_events: i64,
}

type HelperRow = (i64, i64, String, String, i64, Option<i64>);

fn empty_state(message: &str) -> String {
    format!(r#"<div class="empty">{}</div>"#, escape_html(message),)
}

#[allow(clippy::too_many_arguments)]
fn render_page(
    context_title: &str,
    city: &CityInfo,
    metrics: &CityMetrics,
    platforms: &[(String, String, String, String, String, i64)],
    helpers: &[HelperRow],
    reports: &[(i64, i64, String, String, String, i64)],
    resources: &[(i64, String, String, String, i64, i64)],
    events: &[(i64, String, String, i64, String, i64)],
) -> String {
    let platform_cards = if platforms.is_empty() {
        empty_state("Для города пока не подключено ни одной платформы.")
    } else {
        platforms
            .iter()
            .map(
                |(platform, name, external_id, external_url, status, active)| {
                    let link = if external_url.starts_with("https://") {
                        format!(
                            r#"<a href="{}" target="_blank" rel="noopener noreferrer">Открыть</a>"#,
                            escape_html(external_url),
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"<article class="card">
                            <div class="row">
                                <strong>{name}</strong>
                                <span class="badge">{platform}</span>
                            </div>
                            <p>ID: {external_id}</p>
                            <div class="meta">
                                <span>Статус: {status}</span>
                                <span>Активна: {active}</span>
                                {link}
                            </div>
                        </article>"#,
                        name = escape_html(name),
                        platform = escape_html(platform),
                        external_id = escape_html(external_id),
                        status = escape_html(status),
                        active = if *active == 1 { "да" } else { "нет" },
                        link = link,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let helper_rows = if helpers.is_empty() {
        empty_state("Помощники групп пока не назначены.")
    } else {
        helpers
            .iter()
            .map(
                |(assignment_id, user_id, group_name, status, mask, valid_until)| {
                    format!(
                        r#"<div class="list-row">
                            <div>
                                <strong>User #{user_id}</strong>
                                <small>{group_name}</small>
                            </div>
                            <div class="right">
                                <span class="badge">{status}</span>
                                <small>Назначение #{assignment_id} · права {mask} · до {until}</small>
                            </div>
                        </div>"#,
                        assignment_id = assignment_id,
                        user_id = user_id,
                        group_name = escape_html(group_name),
                        status = escape_html(status),
                        mask = mask,
                        until = valid_until
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "без срока".to_string()),
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let report_rows = if reports.is_empty() {
        empty_state("Открытых жалоб в городе нет.")
    } else {
        reports
            .iter()
            .map(|(id, resource_id, title, reason, status, created_at)| {
                format!(
                    r#"<div class="list-row">
                        <div>
                            <strong>Жалоба #{id} · {title}</strong>
                            <small>{reason}</small>
                        </div>
                        <div class="right">
                            <span class="badge warning">{status}</span>
                            <small>Ресурс #{resource_id} · Unix {created_at}</small>
                        </div>
                    </div>"#,
                    id = id,
                    resource_id = resource_id,
                    title = escape_html(title),
                    reason = escape_html(reason),
                    status = escape_html(status),
                    created_at = created_at,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let resource_rows = if resources.is_empty() {
        empty_state("Ресурсов в городе пока нет.")
    } else {
        resources
            .iter()
            .map(|(id, title, category, status, active, verified)| {
                format!(
                    r#"<div class="list-row">
                        <div>
                            <strong>#{id} · {title}</strong>
                            <small>{category}</small>
                        </div>
                        <div class="right">
                            <span class="badge">{status}</span>
                            <small>Активен: {active} · Проверен: {verified}</small>
                        </div>
                    </div>"#,
                    id = id,
                    title = escape_html(title),
                    category = escape_html(category),
                    status = escape_html(status),
                    active = if *active == 1 { "да" } else { "нет" },
                    verified = if *verified == 1 { "да" } else { "нет" },
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let event_rows = if events.is_empty() {
        empty_state("Событий защиты города пока нет.")
    } else {
        events
            .iter()
            .map(|(user_id, action, reason, risk, level, created_at)| {
                format!(
                    r#"<div class="list-row">
                        <div>
                            <strong>{action} · User #{user_id}</strong>
                            <small>{reason}</small>
                        </div>
                        <div class="right">
                            <span class="badge danger">{level}</span>
                            <small>Риск {risk} · Unix {created_at}</small>
                        </div>
                    </div>"#,
                    user_id = user_id,
                    action = escape_html(action),
                    reason = escape_html(reason),
                    risk = risk,
                    level = escape_html(level),
                    created_at = created_at,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="robots" content="noindex,nofollow">
<title>{city_name} · Управление городом · ResursMap</title>
<style>
:root {{
    color-scheme:dark;
    --bg:#07110d;
    --panel:#0d1c16;
    --panel2:#12251d;
    --line:#254438;
    --text:#eef8f2;
    --muted:#95aa9f;
    --green:#55e49a;
    --gold:#e9c46a;
    --red:#ff7b7b;
}}
* {{ box-sizing:border-box; }}
body {{
    margin:0;
    color:var(--text);
    background:
      radial-gradient(circle at 15% 0%,#163d2b 0,transparent 34%),
      radial-gradient(circle at 90% 12%,#173329 0,transparent 28%),
      var(--bg);
    font-family:Inter,system-ui,-apple-system,sans-serif;
}}
.wrap {{ width:min(1180px,calc(100% - 28px)); margin:0 auto; padding:24px 0 60px; }}
.hero {{
    padding:28px;
    border:1px solid var(--line);
    border-radius:28px;
    background:linear-gradient(145deg,rgba(21,50,38,.98),rgba(8,20,15,.98));
    box-shadow:0 24px 70px rgba(0,0,0,.3);
}}
.hero-top,.row,.section-head,.list-row {{ display:flex; justify-content:space-between; gap:14px; }}
.eyebrow {{ color:var(--green); font-size:12px; letter-spacing:.14em; text-transform:uppercase; }}
h1 {{ margin:10px 0 6px; font-size:clamp(30px,6vw,54px); }}
.hero p,.card p {{ color:var(--muted); }}
.actions {{ display:flex; flex-wrap:wrap; gap:9px; margin-top:20px; }}
.button {{
    padding:11px 14px;
    border:1px solid var(--line);
    border-radius:13px;
    color:var(--text);
    background:#11251c;
    text-decoration:none;
}}
.button.primary {{ color:#062015; background:var(--green); border-color:var(--green); font-weight:800; }}
.metrics {{ display:grid; grid-template-columns:repeat(6,1fr); gap:10px; margin:16px 0 26px; }}
.metric,.card,.panel {{
    border:1px solid var(--line);
    background:linear-gradient(145deg,rgba(18,38,29,.96),rgba(8,19,14,.96));
    border-radius:18px;
}}
.metric {{ padding:16px; }}
.metric strong {{ display:block; color:var(--green); font-size:25px; }}
.metric span,.meta,small {{ color:var(--muted); font-size:12px; }}
.section-head {{ align-items:end; margin:25px 2px 11px; }}
.section-head h2 {{ margin:0; }}
.grid {{ display:grid; grid-template-columns:repeat(2,1fr); gap:11px; }}
.card {{ padding:16px; }}
.panel {{ overflow:hidden; }}
.list-row {{ align-items:center; padding:14px 16px; border-bottom:1px solid var(--line); }}
.list-row:last-child {{ border-bottom:0; }}
.list-row small {{ display:block; margin-top:5px; }}
.right {{ text-align:right; }}
.badge {{
    display:inline-block;
    padding:5px 8px;
    border:1px solid var(--line);
    border-radius:999px;
    color:var(--green);
    font-size:12px;
}}
.badge.warning {{ color:var(--gold); }}
.badge.danger {{ color:var(--red); }}
.meta {{ display:flex; flex-wrap:wrap; gap:10px; }}
.meta a {{ color:var(--green); }}
.empty {{ padding:20px; color:var(--muted); border:1px dashed var(--line); border-radius:16px; }}
footer {{ margin-top:28px; color:var(--muted); font-size:12px; }}
@media(max-width:900px) {{
    .metrics {{ grid-template-columns:repeat(3,1fr); }}
}}
@media(max-width:650px) {{
    .wrap {{ width:min(100% - 18px,1180px); padding-top:10px; }}
    .hero {{ padding:20px; border-radius:21px; }}
    .metrics,.grid {{ grid-template-columns:1fr 1fr; }}
    .list-row,.hero-top {{ align-items:flex-start; }}
}}
@media(max-width:430px) {{
    .metrics,.grid {{ grid-template-columns:1fr; }}
    .list-row {{ flex-direction:column; }}
    .right {{ text-align:left; }}
}}
</style>
</head>
<body>
<main class="wrap">
<section class="hero">
    <div class="hero-top">
        <div>
            <div class="eyebrow">Уровень 2 · городской контур</div>
            <h1>{city_name}</h1>
            <p>{native_name} · {country_name} · {continent_name}</p>
        </div>
        <span class="badge">{context_title}</span>
    </div>
    <p>Защищённое управление ресурсами, жалобами, помощниками и подключёнными сообществами города.</p>
    <div class="actions">
        <a class="button primary" href="/app/center/geography?q={stable_key}">Платформы города</a>
        <a class="button" href="/app/center/administrators">Назначения</a>
        <a class="button" href="/app">Открыть ResursMap</a>
    </div>
</section>

<section class="metrics">
    <div class="metric"><strong>{resource_count}</strong><span>Ресурсы</span></div>
    <div class="metric"><strong>{pending_resources}</strong><span>На проверке</span></div>
    <div class="metric"><strong>{open_reports}</strong><span>Жалобы</span></div>
    <div class="metric"><strong>{active_platforms}</strong><span>Платформы</span></div>
    <div class="metric"><strong>{helper_count}</strong><span>Помощники</span></div>
    <div class="metric"><strong>{security_count}</strong><span>События защиты</span></div>
</section>

<div class="section-head"><h2>Платформы города</h2><span>До {platform_limit}</span></div>
<section class="grid">{platform_cards}</section>

<div class="section-head"><h2>Помощники групп</h2><span>До {helper_limit}</span></div>
<section class="panel">{helper_rows}</section>

<div class="section-head"><h2>Открытые жалобы</h2><span>До {report_limit}</span></div>
<section class="panel">{report_rows}</section>

<div class="section-head"><h2>Ресурсы города</h2><span>До {resource_limit}</span></div>
<section class="panel">{resource_rows}</section>

<div class="section-head"><h2>Безопасность сообществ</h2><span>До {security_limit}</span></div>
<section class="panel">{event_rows}</section>

<footer>
    Территория: {stable_key} · City ID {city_id}. Данные ограничены серверной областью назначения.
</footer>
</main>
</body>
</html>"#,
        city_name = escape_html(&city.name),
        native_name = escape_html(&city.native_name),
        country_name = escape_html(&city.country_name),
        continent_name = escape_html(&city.continent_name),
        stable_key = escape_html(&city.stable_key),
        city_id = city.id,
        context_title = escape_html(context_title),
        resource_count = metrics.resources,
        pending_resources = metrics.pending_resources,
        open_reports = metrics.open_reports,
        active_platforms = metrics.active_platforms,
        helper_count = metrics.helpers,
        security_count = metrics.security_events,
        platform_limit = PLATFORM_LIMIT,
        helper_limit = HELPER_LIMIT,
        report_limit = REPORT_LIMIT,
        resource_limit = RESOURCE_LIMIT,
        security_limit = SECURITY_LIMIT,
        platform_cards = platform_cards,
        helper_rows = helper_rows,
        report_rows = report_rows,
        resource_rows = resource_rows,
        event_rows = event_rows,
    )
}

pub async fn city_admin_panel(State(state): State<AppState>, headers: HeaderMap) -> Response {
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
                "city_admin_access_denied",
                "Нет активного административного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if context.level.number() != 2
        || context.scope_type != "city"
        || !context.has_permission(AdminPermission::ModerationReview)
        || !context.has_permission(AdminPermission::ComplaintsReview)
        || !context.has_permission(AdminPermission::GroupsManage)
        || !context.has_permission(AdminPermission::AssistantsManage)
    {
        record_denied_access(
            &state,
            user.user_id,
            "city_admin_permission_denied",
            "Требуется уровень 2, область города и городские разрешения",
        );

        return (StatusCode::FORBIDDEN, "Кабинет города недоступен").into_response();
    }

    if !scope_is_authorized(&state, &context, context.scope_id) {
        record_denied_access(
            &state,
            user.user_id,
            "city_admin_scope_denied",
            "Назначенная территория не прошла серверную проверку",
        );

        return (StatusCode::FORBIDDEN, "Территория недоступна").into_response();
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

        let mut response = StatusCode::SEE_OTHER.into_response();

        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_static("/app/center/city"),
        );

        if let Ok(value) = HeaderValue::from_str(&cookie) {
            response.headers_mut().insert(header::SET_COOKIE, value);
        }

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

    let city = connection.query_row(
        "SELECT
             city.id,
             city.stable_key,
             city.name_ru,
             city.name_native,
             country.name_ru,
             continent.name_ru
         FROM geographic_scopes AS city_scope
         JOIN geo_cities AS city
           ON city_scope.external_key =
              'city:' || city.stable_key
         JOIN geo_countries AS country
           ON country.id = city.country_id
         JOIN geo_continents AS continent
           ON continent.id = country.continent_id
         WHERE city_scope.id = ?1
           AND city_scope.scope_type = 'city'
           AND city_scope.is_active = 1
           AND city.is_active = 1",
        params![context.scope_id],
        |row| {
            Ok(CityInfo {
                id: row.get(0)?,
                stable_key: row.get(1)?,
                name: row.get(2)?,
                native_name: row.get(3)?,
                country_name: row.get(4)?,
                continent_name: row.get(5)?,
            })
        },
    );

    let city = match city {
        Ok(city) => city,
        Err(_) => {
            record_denied_access(
                &state,
                user.user_id,
                "city_admin_city_resolution_failed",
                "Область назначения не связана с активным городом",
            );

            return (StatusCode::FORBIDDEN, "Город назначения не найден").into_response();
        }
    };

    let metrics = connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM resources WHERE city_id = ?1),
                (SELECT COUNT(*)
                 FROM resources
                 WHERE city_id = ?1
                   AND moderation_status = 'pending'),
                (SELECT COUNT(*)
                 FROM resource_reports AS report
                 JOIN resources AS resource
                   ON resource.id = report.resource_id
                 WHERE resource.city_id = ?1
                   AND report.status <> 'closed'),
                (SELECT COUNT(*)
                 FROM city_publication_targets
                 WHERE city_id = ?1
                   AND is_active = 1
                   AND provider_status = 'active'),
                (SELECT COUNT(*)
                 FROM admin_assignments AS assignment
                 JOIN geographic_scopes AS group_scope
                   ON group_scope.id = assignment.scope_id
                 WHERE assignment.role_level = 1
                   AND assignment.scope_type = 'group'
                   AND assignment.status = 'active'
                   AND group_scope.parent_scope_id = ?2),
                (SELECT COUNT(*)
                 FROM security_moderation_audit AS audit
                 JOIN city_publication_targets AS target
                   ON target.telegram_chat_id = audit.chat_id
                 WHERE target.city_id = ?1)",
            params![city.id, context.scope_id],
            |row| {
                Ok(CityMetrics {
                    resources: row.get(0)?,
                    pending_resources: row.get(1)?,
                    open_reports: row.get(2)?,
                    active_platforms: row.get(3)?,
                    helpers: row.get(4)?,
                    security_events: row.get(5)?,
                })
            },
        )
        .unwrap_or(CityMetrics {
            resources: 0,
            pending_resources: 0,
            open_reports: 0,
            active_platforms: 0,
            helpers: 0,
            security_events: 0,
        });

    let platforms = connection
        .prepare(
            "SELECT
                 platform,
                 target_name,
                 external_target_id,
                 external_url,
                 provider_status,
                 is_active
             FROM city_publication_targets
             WHERE city_id = ?1
             ORDER BY is_active DESC, id
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![city.id, PLATFORM_LIMIT], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
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
                 assignment.permission_mask,
                 assignment.valid_until
             FROM admin_assignments AS assignment
             JOIN geographic_scopes AS group_scope
               ON group_scope.id = assignment.scope_id
             WHERE assignment.role_level = 1
               AND assignment.scope_type = 'group'
               AND group_scope.parent_scope_id = ?1
             ORDER BY
                 CASE assignment.status WHEN 'active' THEN 0 ELSE 1 END,
                 assignment.id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![context.scope_id, HELPER_LIMIT], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
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
               AND report.status <> 'closed'
             ORDER BY report.created_at DESC, report.id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![city.id, REPORT_LIMIT], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let resources = connection
        .prepare(
            "SELECT
                 id,
                 title,
                 category,
                 moderation_status,
                 is_active,
                 is_verified
             FROM resources
             WHERE city_id = ?1
             ORDER BY
                 CASE moderation_status WHEN 'pending' THEN 0 ELSE 1 END,
                 updated_at DESC,
                 id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![city.id, RESOURCE_LIMIT], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let events = connection
        .prepare(
            "SELECT
                 audit.user_id,
                 audit.action,
                 audit.reason,
                 audit.risk_score,
                 audit.risk_level,
                 audit.created_at
             FROM security_moderation_audit AS audit
             JOIN city_publication_targets AS target
               ON target.telegram_chat_id = audit.chat_id
             WHERE target.city_id = ?1
             ORDER BY audit.created_at DESC, audit.id DESC
             LIMIT ?2",
        )
        .and_then(|mut statement| {
            statement
                .query_map(params![city.id, SECURITY_LIMIT], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    let html = render_page(
        context.level.title(),
        &city,
        &metrics,
        &platforms,
        &helpers,
        &reports,
        &resources,
        &events,
    );

    let mut response = Html(html).into_response();

    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_dashboard_limits_are_bounded() {
        assert_eq!(PLATFORM_LIMIT, 30);
        assert_eq!(HELPER_LIMIT, 40);
        assert_eq!(REPORT_LIMIT, 40);
        assert_eq!(RESOURCE_LIMIT, 40);
        assert_eq!(SECURITY_LIMIT, 60);
    }

    #[test]
    fn city_dashboard_contains_territorial_protection() {
        let source = include_str!("city_admin.rs");

        assert!(source.contains("context.level.number() != 2"));
        assert!(source.contains("context.scope_type != \"city\""));
        assert!(source.contains("scope_is_authorized("));
        assert!(source.contains("city_admin_scope_denied"));
    }
}
