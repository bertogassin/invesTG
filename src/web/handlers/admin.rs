use super::admin_access::{
    admin_resource_scope_filter, load_admin_context,
    verify_admin_session as verify_admin_v2_session, AdminPermission,
};
use super::auth::verify_authenticated_user;
use super::common::{
    csrf_rejected_response, input_text_is_valid, request_is_cross_site, resource_owner_user_id,
};
use super::types::RejectResourceForm;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use serde_json::json;
use std::collections::BTreeMap;

pub(crate) fn is_resource_moderation_session(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(user) = verify_authenticated_user(state, headers) else {
        return false;
    };

    let Some(context) = load_admin_context(state, user.user_id) else {
        return false;
    };

    context.has_permission(AdminPermission::ModerationReview)
        && verify_admin_v2_session(state, headers, context.user_id, context.assignment_id)
}

pub(crate) fn moderation_scope_filter(state: &AppState, headers: &HeaderMap) -> String {
    let Some(user) = verify_authenticated_user(state, headers) else {
        return String::new();
    };

    load_admin_context(state, user.user_id)
        .map(|context| admin_resource_scope_filter(&context, "resources"))
        .unwrap_or_default()
}

pub async fn admin_login_page() -> Redirect {
    Redirect::temporary("/login?next=%2Fapp%2Fadmin%2Fresources")
}

pub async fn admin_login(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(_payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    (
        StatusCode::GONE,
        Json(json!({
            "ok": false,
            "error": "telegram_auth_removed",
            "message": "Вход через Telegram отключён. Используйте email и пароль на /login."
        })),
    )
        .into_response()
}

pub async fn admin_reports(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    if !is_resource_moderation_session(&state, &headers) {
        return Html(
            r#"<h1>403</h1>
<p>Доступ запрещён.</p>
<a href="/app/center">Вернуться в центр управления</a>"#
                .to_string(),
        );
    }

    let key_query = String::new();

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let scope_filter = moderation_scope_filter(&state, &headers).replace("resources.", "r.");

    let rows: Vec<crate::web::view_models::AdminReportRow> = db
        .prepare(
            &format!(
                "SELECT
                rr.id,
                rr.reporter_user_id,
                rr.resource_id,
                rr.reason,
                rr.status,
                rr.created_at,
                COALESCE(r.title, ''),
                COALESCE(r.category, ''),
                COALESCE(r.moderation_status, ''),
                COALESCE(r.is_active, 0)
             FROM resource_reports rr
             LEFT JOIN resources r
               ON r.id = rr.resource_id
             WHERE 1 = 1
               {scope_filter}
             ORDER BY
                CASE rr.status
                    WHEN 'pending' THEN 0
                    ELSE 1
                END,
                rr.created_at DESC,
                rr.id DESC"
            ),
        )
        .and_then(|mut stmt| {
            stmt.query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    let pending_count = rows.iter().filter(|r| r.4 == "pending").count();

    let closed_count = rows.iter().filter(|r| r.4 == "closed").count();

    let mut cards = String::new();

    for (
        report_id,
        reporter_user_id,
        resource_id,
        reason,
        report_status,
        created_at,
        title,
        category,
        moderation_status,
        is_active,
    ) in rows
    {
        let safe_reason = templates::escape_html(&reason);
        let safe_title = templates::escape_html(&title);
        let safe_category = templates::escape_html(&category);
        let safe_moderation_status = templates::escape_html(&moderation_status);

        let report_badge = templates::report_queue_badge(&report_status);

        let resource_status =
            templates::resource_visibility_with_status(is_active, &safe_moderation_status);

        let close_button = if report_status == "pending" {
            format!(
                r#"
<form method="post"
      action="/app/admin/report/{report_id}/close{key_query}">
    <button type="submit" class="rm-mod-btn rm-mod-btn--ok">
        ✓ Закрыть жалобу
    </button>
</form>
"#,
                report_id = report_id,
                key_query = key_query,
            )
        } else {
            String::new()
        };

        cards.push_str(&format!(
            r#"
<article class="rm-mod-card">

    <div class="rm-mod-card-head">
        <div>
            <div class="rm-mod-card-kicker">
                Жалоба #{report_id} · ресурс #{resource_id}
            </div>

            <h2 class="rm-mod-card-title">
                {title}
            </h2>

            <div class="rm-mod-card-category">
                {category}
            </div>
        </div>

        <div class="rm-mod-badges">
            {report_badge}
            {resource_status}
        </div>
    </div>

    <div class="rm-mod-reason">
        <strong>Причина:</strong><br>
        {reason}
    </div>

    <div class="rm-mod-meta">
        Reporter Telegram ID: {reporter_user_id}
        · created_at: {created_at}
    </div>

    <div class="rm-mod-actions">

        {close_button}

        <form method="post"
              action="/app/admin/report/{report_id}/hide-resource{key_query}">
            <button type="submit" class="rm-mod-btn rm-mod-btn--danger">
                Скрыть ресурс
            </button>
        </form>

        <form method="post"
              action="/app/admin/report/{report_id}/reject-resource{key_query}">
            <button type="submit" class="rm-mod-btn rm-mod-btn--danger-strong">
                ✕ Отклонить ресурс
            </button>
        </form>

        <a href="/app/resource/{resource_id}"
           target="_blank"
           class="rm-mod-link">
            Открыть ресурс
        </a>

    </div>

</article>
"#,
            report_id = report_id,
            resource_id = resource_id,
            title = safe_title,
            category = safe_category,
            report_badge = report_badge,
            resource_status = resource_status,
            reason = safe_reason,
            reporter_user_id = reporter_user_id,
            created_at = created_at,
            close_button = close_button,
            key_query = key_query,
        ));
    }

    if cards.is_empty() {
        cards = r#"
<div class="card rm-mod-empty">
    <div class="card-content">
        <div class="card-title">Жалоб пока нет</div>
        <div class="card-meta">Очередь модерации пуста.</div>
    </div>
</div>
"#
        .to_string();
    }

    let main_html = format!(
        r#"
<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">ResursMap</div>
            <div class="brand-sub">ЦЕНТР ЖАЛОБ</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {shield}
        Модерация
    </div>

    <h1>Жалобы</h1>

    <p>
        Проверка жалоб пользователей на опубликованные ресурсы.
    </p>

</section>

<div class="rm-mod-nav">

    <a href="/app/admin/resources{key_query}" class="rm-mod-chip">
        Ресурсы
    </a>

    <a href="/app/admin/reports{key_query}" class="rm-mod-chip rm-mod-chip--reports">
        Жалобы
    </a>

</div>

<div class="rm-mod-metrics rm-mod-metrics--2">

    <div class="card">
        <div class="card-content">
            <div class="card-title">{pending_count}</div>
            <div class="card-meta">Ожидают</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{closed_count}</div>
            <div class="card-meta">Закрыты</div>
        </div>
    </div>

</div>

<section>
    {cards}
</section>
"#,
        logo = templates::brand_logo(),
        shield = templates::icon("shield"),
        pending_count = pending_count,
        closed_count = closed_count,
        cards = cards,
        key_query = key_query,
    );

    Html(templates::page_document(
        "Жалобы · ResursMap",
        "",
        "",
        &main_html,
        "",
        "",
    ))
}

pub async fn admin_close_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let _ = db.execute(
        "UPDATE resource_reports
         SET status = 'closed',
             updated_at = strftime('%s','now')
         WHERE id = ?1",
        rusqlite::params![id],
    );

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/admin/reports")],
    )
        .into_response()
}

pub async fn admin_hide_reported_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let resource_id: Option<i64> = db
        .query_row(
            "SELECT resource_id
             FROM resource_reports
             WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    if let Some(resource_id) = resource_id {
        let transaction_result = (|| -> rusqlite::Result<()> {
            let tx = db.unchecked_transaction()?;

            tx.execute(
                "UPDATE resources
                 SET is_active = 0,
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![resource_id],
            )?;

            tx.execute(
                "UPDATE resource_reports
                 SET status = 'closed',
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![id],
            )?;

            tx.commit()
        })();

        if let Err(err) = transaction_result {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка обработки жалобы: {}", err),
            )
                .into_response();
        }
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/admin/reports")],
    )
        .into_response()
}

pub async fn admin_reject_reported_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let report: Option<(i64, String)> = db
        .query_row(
            "SELECT resource_id, reason
             FROM resource_reports
             WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    if let Some((resource_id, reason)) = report {
        let rejection_reason = format!("Жалоба пользователя: {}", reason);

        let transaction_result = (|| -> rusqlite::Result<()> {
            let tx = db.unchecked_transaction()?;

            tx.execute(
                "UPDATE resources
                 SET moderation_status = 'rejected',
                     rejection_reason = ?2,
                     is_verified = 0,
                     is_active = 0,
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![resource_id, rejection_reason],
            )?;

            tx.execute(
                "UPDATE resource_reports
                 SET status = 'closed',
                     updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![id],
            )?;

            tx.commit()
        })();

        if let Err(err) = transaction_result {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка отклонения ресурса по жалобе: {}", err),
            )
                .into_response();
        }
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/admin/reports")],
    )
        .into_response()
}

pub async fn admin_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<BTreeMap<String, String>>,
) -> Html<String> {
    let filter = params.get("filter").map(|s| s.as_str()).unwrap_or("all");

    let q = params.get("q").map(|s| s.trim()).unwrap_or("");

    if q.chars().count() > 100 || q.chars().any(|c| c.is_control()) {
        return Html(
            "<h1>400</h1><p>Поисковый запрос должен содержать не более 100 символов.</p>"
                .to_string(),
        );
    }

    if !is_resource_moderation_session(&state, &headers) {
        return Html(
            r#"<h1>403</h1>
<p>Доступ запрещён.</p>
<a href="/app/center">Вернуться в центр управления</a>"#
                .to_string(),
        );
    }

    let key_query = String::new();
    let key_join = "?";

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let filter_clause = match filter {
        "pending" => "moderation_status = 'pending'",
        "verified" => "moderation_status = 'approved'",
        "rejected" => "moderation_status = 'rejected'",
        "premium" => "is_premium = 1",
        "hidden" => "is_active = 0",
        _ => "1 = 1",
    };

    let search_clause = if q.is_empty() {
        "1 = 1"
    } else {
        "(LOWER(title) LIKE LOWER(?1)
          OR LOWER(category) LIKE LOWER(?1)
          OR LOWER(description) LIKE LOWER(?1))"
    };

    let scope_filter = moderation_scope_filter(&state, &headers);

    let sql = format!(
        "SELECT
            id,
            title,
            category,
            description,
            rating,
            votes,
            is_verified,
            is_premium,
            is_active,
            moderation_status,
            rejection_reason
         FROM resources
         WHERE {}
           AND {}
           {}
         ORDER BY
            is_verified ASC,
            is_active DESC,
            is_premium DESC,
            id DESC",
        filter_clause, search_clause, scope_filter
    );

    let rows: Vec<crate::web::view_models::AdminResourceRow> = if q.is_empty() {
        db.prepare(&sql)
            .and_then(|mut stmt| {
                stmt.query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default()
    } else {
        let pattern = format!("%{}%", q);

        db.prepare(&sql)
            .and_then(|mut stmt| {
                stmt.query_map(rusqlite::params![pattern], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default()
    };

    let pending_reports_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resource_reports
             WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    drop(db);

    let result_count = rows.len();

    let pending = rows.iter().filter(|r| r.9 == "pending").count();

    let verified_count = rows.iter().filter(|r| r.9 == "approved").count();

    let rejected_count = rows.iter().filter(|r| r.9 == "rejected").count();

    let premium_count = rows.iter().filter(|r| r.7 == 1).count();

    let mut cards = String::new();

    for (
        id,
        title,
        category,
        description,
        rating,
        votes,
        _verified,
        premium,
        active,
        moderation_status,
        rejection_reason,
    ) in rows
    {
        let safe_title = templates::escape_html(&title);
        let safe_category = templates::escape_html(&category);
        let safe_description = templates::escape_html(&description);
        let safe_rejection_reason = templates::escape_html(&rejection_reason);

        let moderation_badge = templates::moderation_queue_badge(&moderation_status);

        let rejection_html =
            if moderation_status == "rejected" && !rejection_reason.trim().is_empty() {
                format!(
                    r#"<div class="rm-mod-rejection"><strong>Причина отказа:</strong> {}</div>"#,
                    safe_rejection_reason
                )
            } else {
                String::new()
            };

        let premium_badge = if premium == 1 {
            templates::premium_badge_html("admin")
        } else {
            ""
        };

        let active_badge = templates::resource_visibility_badge(active);

        let premium_label = if premium == 1 {
            "Снять премиум"
        } else {
            "★ Включить премиум"
        };

        let active_label = if active == 1 {
            "Скрыть"
        } else {
            "Вернуть"
        };

        cards.push_str(&format!(
            r#"
<article class="rm-mod-card rm-mod-card--resource">

    <div class="rm-mod-card-head rm-mod-card-head--resource">
        <div>
            <div class="rm-mod-card-kicker">
                #{id} · {category}
            </div>

            <h2 class="rm-mod-card-title rm-mod-card-title--resource">
                {title}
            </h2>

            <div class="rm-mod-card-desc">
                {description}
            </div>
        </div>

        <div class="rm-mod-rating">
            ⭐ {rating:.1} · {votes}
        </div>
    </div>

    <div class="rm-mod-badges rm-mod-badges--resource">
        {moderation_badge}
        {premium_badge}
        {active_badge}
    </div>

    {rejection_html}

    <div class="rm-mod-actions">

        <form method="post"
              action="/app/admin/resource/{id}/approve{key_query}">
            <button type="submit" class="rm-mod-btn rm-mod-btn--ok">
                ✓ Одобрить
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/reject{key_query}"
              class="rm-mod-reject-form">

            <input
                type="text"
                name="reason"
                required
                maxlength="500"
                placeholder="Причина отклонения"
                class="rm-mod-reject-input">

            <button type="submit" class="rm-mod-reject-btn">
                ✕ Отклонить
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/toggle-premium{key_query}">
            <button type="submit" class="rm-mod-btn rm-mod-btn--gold">
                {premium_label}
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/toggle-active{key_query}">
            <button type="submit" class="rm-mod-btn rm-mod-btn--neutral">
                {active_label}
            </button>
        </form>

        <a href="/app/resource/{id}"
           target="_blank"
           class="rm-mod-link">
            Открыть ресурс
        </a>

    </div>
</article>
"#,
            id = id,
            title = safe_title,
            category = safe_category,
            description = safe_description,
            rating = rating,
            votes = votes,
            moderation_badge = moderation_badge,
            rejection_html = rejection_html,
            premium_badge = premium_badge,
            active_badge = active_badge,
            premium_label = premium_label,
            active_label = active_label,
            key_query = key_query,
        ));
    }

    if cards.is_empty() {
        cards = r#"
<div class="card rm-mod-empty">
    <div class="card-content">
        <div class="card-title">Ресурсы не найдены</div>
        <div class="card-meta">По текущему фильтру и поиску ничего не найдено.</div>
    </div>
</div>
"#
        .to_string();
    }

    let safe_q = templates::escape_html(q);
    let safe_filter = templates::escape_html(filter);

    let main_html = format!(
        r####"<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">ResursMap</div>
            <div class="brand-sub">ЦЕНТР МОДЕРАЦИИ</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {shield}
        Модерация
    </div>

    <h1>Модерация ресурсов</h1>

    <p>
        Проверка, премиум-статус и видимость ресурсов.
    </p>

</section>

<form method="get"
      action="/app/admin/resources"
      class="rm-mod-search">

    <input type="hidden" name="filter" value="{filter}">

    <input
        type="search"
        name="q"
        value="{q}"
        placeholder="Поиск по названию, категории, описанию..."
        class="rm-mod-search-input"
    >

    <button type="submit" class="rm-mod-search-btn">
        Найти
    </button>

</form>

<div class="rm-mod-nav">

    <a href="/app/admin/reports{key_query}" class="rm-mod-chip rm-mod-chip--reports-alert">
        🚩 Жалобы ({pending_reports_count})
    </a>

    <a href="/app/admin/promotions{key_query}" class="rm-mod-chip rm-mod-chip--premium">
        📣 Продвижение
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=all" class="rm-mod-chip">
        Все
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=pending" class="rm-mod-chip rm-mod-chip--pending">
        Ожидают
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=verified" class="rm-mod-chip rm-mod-chip--verified">
        Проверены
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=rejected" class="rm-mod-chip rm-mod-chip--rejected">
        Отклонённые
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=premium" class="rm-mod-chip rm-mod-chip--premium">
        Премиум
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=hidden" class="rm-mod-chip rm-mod-chip--hidden">
        Скрытые
    </a>

</div>

<div class="rm-mod-metrics">

    <div class="card">
        <div class="card-content">
            <div class="card-title">{pending}</div>
            <div class="card-meta">Ожидают проверки</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{verified_count}</div>
            <div class="card-meta">Проверены</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{rejected_count}</div>
            <div class="card-meta">Отклонены</div>
        </div>
    </div>

    <div class="card">
        <div class="card-content">
            <div class="card-title">{premium_count}</div>
            <div class="card-meta">Премиум</div>
        </div>
    </div>

</div>

<section>

    <div class="rm-mod-bulk-head">
        <strong>Найдено: {result_count}</strong>
        <span class="rm-mod-bulk-note">
            Массовые действия применяются к текущей выборке
        </span>
    </div>

    <div class="rm-mod-actions rm-mod-actions--bulk">

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=verify">
            <button type="submit" class="rm-mod-btn rm-mod-btn--ok">
                ✓ Одобрить найденные
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=unverify">
            <button type="submit" class="rm-mod-btn rm-mod-btn--warn">
                Снять проверку
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=premium">
            <button type="submit" class="rm-mod-btn rm-mod-btn--gold">
                ★ Включить премиум
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=hide">
            <button type="submit" class="rm-mod-btn rm-mod-btn--danger">
                Скрыть найденные
            </button>
        </form>

    </div>
</section>

<section>
{cards}
</section>"####,
        logo = templates::brand_logo(),
        shield = templates::icon("shield"),
        filter = safe_filter,
        q = safe_q,
        key_query = key_query,
        pending_reports_count = pending_reports_count,
        key_join = key_join,
        pending = pending,
        verified_count = verified_count,
        rejected_count = rejected_count,
        premium_count = premium_count,
        result_count = result_count,
        cards = cards,
    );

    Html(templates::page_document(
        "Модерация · ResursMap",
        "",
        "",
        &main_html,
        "",
        "",
    ))
}

pub async fn admin_bulk_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<BTreeMap<String, String>>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    let filter = params.get("filter").map(|s| s.as_str()).unwrap_or("all");
    let q = params.get("q").map(|s| s.trim()).unwrap_or("");
    let action = params.get("action").map(|s| s.as_str()).unwrap_or("");

    if q.chars().count() > 100 || q.chars().any(|c| c.is_control()) {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>400</h1><p>Поисковый запрос должен содержать не более 100 символов.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    if !is_resource_moderation_session(&state, &headers) {
        return (
            StatusCode::FORBIDDEN,
            Html("<h1>403</h1><p>Доступ запрещён</p>".to_string()),
        )
            .into_response();
    }

    let filter_clause = match filter {
        "pending" => "is_verified = 0 AND is_active = 1",
        "verified" => "is_verified = 1",
        "premium" => "is_premium = 1",
        "hidden" => "is_active = 0",
        _ => "1 = 1",
    };

    let action_sql = match action {
        "verify" => "is_verified = 1",
        "unverify" => "is_verified = 0",
        "premium" => "is_premium = 1",
        "hide" => "is_active = 0",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html("<h1>400</h1><p>Неизвестное действие</p>".to_string()),
            )
                .into_response();
        }
    };

    let search_clause = if q.is_empty() {
        "1 = 1"
    } else {
        "(LOWER(title) LIKE LOWER(?1)
          OR LOWER(category) LIKE LOWER(?1)
          OR LOWER(description) LIKE LOWER(?1))"
    };

    let sql = format!(
        "UPDATE resources
         SET {},
             updated_at = strftime('%s','now')
         WHERE {}
           AND {}",
        action_sql, filter_clause, search_clause
    );

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let changed = if q.is_empty() {
        db.execute(&sql, []).unwrap_or(0)
    } else {
        let pattern = format!("%{}%", q);

        db.execute(&sql, rusqlite::params![pattern]).unwrap_or(0)
    };

    drop(db);

    let filter_url = urlencoding::encode(filter);
    let q_url = urlencoding::encode(q);

    let head_extra = format!(
        r#"<meta http-equiv="refresh"
content="1;url=/app/admin/resources?filter={filter_url}&amp;q={q_url}">"#,
        filter_url = filter_url,
        q_url = q_url,
    );

    let main_html = format!(
        r#"<section class="card rm-mod-result">
    <div class="card-title">
        Изменено ресурсов: {changed}
    </div>

    <div class="card-meta rm-mod-result-meta">
        Возвращаемся в модерацию…
    </div>
</section>"#,
        changed = changed,
    );

    Html(templates::page_document(
        "Готово · ResursMap",
        &head_extra,
        "",
        &main_html,
        "",
        "",
    ))
    .into_response()
}

async fn admin_toggle_field(
    state: &AppState,
    headers: &HeaderMap,
    id: i64,
    field: &str,
) -> Response {
    if !is_resource_moderation_session(state, headers) {
        return (StatusCode::FORBIDDEN, Html("<h1>403</h1>".to_string())).into_response();
    }

    let allowed = ["is_verified", "is_premium", "is_active"];

    if !allowed.contains(&field) {
        return (StatusCode::BAD_REQUEST, Html("<h1>400</h1>".to_string())).into_response();
    }

    let sql = format!(
        "UPDATE resources
         SET {0} = CASE WHEN {0}=1 THEN 0 ELSE 1 END,
             updated_at = strftime('%s','now')
         WHERE id=?1",
        field
    );

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };
    let _ = db.execute(&sql, rusqlite::params![id]);
    drop(db);

    Html(r#"<meta http-equiv="refresh" content="0;url=/app/admin/resources">"#.to_string())
        .into_response()
}

pub async fn admin_toggle_verified(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    admin_toggle_field(&state, &headers, id, "is_verified").await
}

pub async fn admin_toggle_premium(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    admin_toggle_field(&state, &headers, id, "is_premium").await
}

pub async fn admin_toggle_active(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    admin_toggle_field(&state, &headers, id, "is_active").await
}

pub async fn admin_approve_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let result = db.execute(
        "UPDATE resources
         SET moderation_status = 'approved',
             rejection_reason = '',
             is_verified = 1,
             is_active = 1,
             updated_at = strftime('%s','now')
         WHERE id = ?1",
        rusqlite::params![id],
    );

    if result.is_ok() {
        let owner: Option<(String, String)> = db
            .query_row(
                "SELECT client_id, title
                 FROM resources
                 WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((client_id, resource_title)) = owner {
            if let Some(user_id) = resource_owner_user_id(&client_id) {
                let _ = db.execute(
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
                        ?1,
                        ?2,
                        'resource_approved',
                        'Ресурс одобрен',
                        ?3,
                        0,
                        strftime('%s','now')
                     )",
                    rusqlite::params![
                        user_id,
                        id,
                        format!(
                            "Ваш ресурс «{}» прошёл модерацию и опубликован.",
                            resource_title
                        ),
                    ],
                );
            }
        }
    }

    drop(db);

    match result {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            [(header::LOCATION, "/app/admin/resources?filter=pending")],
        )
            .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка одобрения ресурса: {}", err),
        )
            .into_response(),
    }
}

pub async fn admin_reject_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<RejectResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, "Доступ запрещён").into_response();
    }

    let reason = form.reason.trim();

    if !input_text_is_valid(reason, 1, 500) {
        return (
            StatusCode::BAD_REQUEST,
            "Причина отклонения должна содержать от 1 до 500 символов",
        )
            .into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let result = db.execute(
        "UPDATE resources
         SET moderation_status = 'rejected',
             rejection_reason = ?2,
             is_verified = 0,
             is_active = 0,
             updated_at = strftime('%s','now')
         WHERE id = ?1",
        rusqlite::params![id, reason],
    );

    if result.is_ok() {
        let owner: Option<(String, String)> = db
            .query_row(
                "SELECT client_id, title
                 FROM resources
                 WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((client_id, resource_title)) = owner {
            if let Some(user_id) = resource_owner_user_id(&client_id) {
                let message = format!("Ресурс «{}» отклонён. Причина: {}", resource_title, reason);

                let _ = db.execute(
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
                        ?1,
                        ?2,
                        'resource_rejected',
                        'Ресурс требует исправления',
                        ?3,
                        0,
                        strftime('%s','now')
                     )",
                    rusqlite::params![user_id, id, message,],
                );
            }
        }
    }

    drop(db);

    match result {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            [(header::LOCATION, "/app/admin/resources?filter=pending")],
        )
            .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка отклонения ресурса: {}", err),
        )
            .into_response(),
    }
}

// ============ ПАНЕЛЬ МОДЕРАТОРА (уровни 1-3) ============

pub async fn moderator_panel(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return (StatusCode::UNAUTHORIZED, "Требуется вход").into_response();
        }
    };

    if let Some(context) = load_admin_context(&state, user.user_id) {
        if context.has_permission(AdminPermission::ModerationReview)
            && verify_admin_v2_session(&state, &headers, context.user_id, context.assignment_id)
        {
            return Redirect::temporary("/app/admin/resources").into_response();
        }
    }

    // Legacy moderator_roles table is optional; redirect Admin V2 moderators above.
    let mod_level: i64 = state
        .db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COALESCE(MAX(level), 0)
                 FROM moderator_roles
                 WHERE user_id = ?1
                   AND is_active = 1",
                rusqlite::params![user.user_id],
                |row| row.get(0),
            )
            .ok()
        })
        .unwrap_or(0);

    if mod_level == 0 || mod_level == 4 {
        return (StatusCode::NOT_FOUND, "404").into_response();
    }

    Redirect::temporary("/app/admin/resources").into_response()
}

pub async fn moderate_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"ok": false}))).into_response(),
    };

    if !is_resource_moderation_session(&state, &headers) {
        let mod_level: i64 = state
            .db_pool
            .get()
            .ok()
            .and_then(|conn| {
                conn.query_row(
                    "SELECT COALESCE(MAX(level), 0)
                     FROM moderator_roles
                     WHERE user_id = ?1
                       AND is_active = 1",
                    rusqlite::params![user.user_id],
                    |row| row.get(0),
                )
                .ok()
            })
            .unwrap_or(0);

        if mod_level == 0 || mod_level == 4 {
            return (StatusCode::FORBIDDEN, Json(json!({"ok": false}))).into_response();
        }
    }

    let resource_id = payload.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("");

    if resource_id <= 0 || (status != "approved" && status != "rejected") {
        return (StatusCode::BAD_REQUEST, Json(json!({"ok": false}))).into_response();
    }

    let updated = state.db_pool.get().ok().and_then(|conn| {
        conn.execute(
            "UPDATE resources
             SET moderation_status = ?1,
                 updated_at = strftime('%s','now')
             WHERE id = ?2",
            rusqlite::params![status, resource_id],
        )
        .ok()
    });

    if updated != Some(1) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"ok": false, "error": "resource_not_found"})),
        )
            .into_response();
    }

    let owner_row: Option<(String, String)> = state.db_pool.get().ok().and_then(|conn| {
        conn.query_row(
            "SELECT client_id, title FROM resources WHERE id = ?1",
            rusqlite::params![resource_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok()
    });

    if let Some((client_id, resource_title)) = owner_row {
        if let Some(user_id) = resource_owner_user_id(&client_id) {
            let (kind, title, message) = if status == "approved" {
                (
                    "resource_approved",
                    "Ресурс одобрен",
                    format!(
                        "Ваш ресурс «{}» прошёл модерацию и опубликован.",
                        resource_title
                    ),
                )
            } else {
                (
                    "resource_rejected",
                    "Ресурс отклонён",
                    format!("Ваш ресурс «{}» отклонён модератором.", resource_title),
                )
            };

            let _ = state.db_pool.get().ok().map(|conn| {
                conn.execute(
                    "INSERT INTO user_notifications (
                        user_id,
                        resource_id,
                        kind,
                        title,
                        message,
                        is_read,
                        created_at
                     )
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, strftime('%s','now'))",
                    rusqlite::params![user_id, resource_id, kind, title, message],
                )
            });
        }
    }

    let _ = state.db_pool.get().ok().and_then(|conn| {
        conn.execute(
            "INSERT INTO moderation_actions (
                moderator_id,
                action_type,
                target_type,
                target_id,
                details
             )
             VALUES (?1, 'moderate_resource', 'resource', ?2, ?3)",
            rusqlite::params![user.user_id, resource_id, status],
        )
        .ok()
    });

    Json(json!({"ok": true})).into_response()
}
