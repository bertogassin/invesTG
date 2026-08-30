use super::admin_access::{
    load_admin_context, verify_admin_session as verify_admin_v2_session, AdminPermission,
};
use super::auth::{
    create_admin_session, is_admin_session, verify_authenticated_user, verify_telegram_init_data,
};
use super::common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
    telegram_owner_user_id,
};
use super::types::RejectResourceForm;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::BTreeMap;

fn is_resource_moderation_session(state: &AppState, headers: &HeaderMap) -> bool {
    // Сохраняем поддержку старой административной панели.
    if is_admin_session(state, headers) {
        return true;
    }

    // Admin V2 использует отдельную короткую серверную сессию.
    let Some(user) = verify_authenticated_user(state, headers) else {
        return false;
    };

    let Some(context) = load_admin_context(state, user.user_id) else {
        return false;
    };

    context.is_owner()
        && context.has_permission(AdminPermission::ModerationReview)
        && verify_admin_v2_session(state, headers, context.user_id, context.assignment_id)
}

pub async fn admin_login_page() -> Html<String> {
    let head_extra = r####"<script src="https://telegram.org/js/telegram-web-app.js"></script>
<style>
body {
    margin: 0;
    padding: 28px;
    background: #0b0e12;
    color: #f5f5f5;
    font-family: system-ui;
}
</style>"####;

    let main_html = r####"<div style="max-width:520px;margin:auto;">
    <h1>ResursMap · Модерация</h1>
    <p id="status">Проверяем Telegram…</p>
</div>"####;

    let body_after = r####"<script>
(async function () {
    const status = document.getElementById("status");

    try {
        const tg = window.Telegram && window.Telegram.WebApp
            ? window.Telegram.WebApp
            : null;

        if (!tg || !tg.initData) {
            status.textContent =
                "Откройте эту страницу через Telegram Mini App.";
            return;
        }

        tg.ready();

        const response = await fetch("/app/admin/login", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                init_data: tg.initData
            })
        });

        const data = await response.json();

        if (!data.ok) {
            status.textContent = "Доступ запрещён.";
            return;
        }

        status.textContent = "✓ Вход выполнен";

        window.location.replace("/app/admin/resources");
    } catch (_) {
        status.textContent = "Ошибка авторизации.";
    }
})();
</script>"####;

    Html(templates::page_document(
        "Вход администратора · ResursMap",
        head_extra,
        "",
        main_html,
        "",
        body_after,
    ))
}

pub async fn admin_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }
    let init_data = payload
        .get("init_data")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let user_id = match verify_telegram_init_data(init_data, &state.bot_token) {
        Some(id) if id == state.admin_telegram_id => id,

        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "ok": false,
                    "error": "forbidden"
                })),
            )
                .into_response();
        }
    };

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "admin_auth", 10, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(json!({
                "ok": false,
                "error": "rate_limited",
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    let session = create_admin_session(&state, user_id);

    let cookie = format!(
        "resursmap_admin={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=43200",
        session
    );

    let mut response = Json(json!({
        "ok": true
    }))
    .into_response();

    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }

    response
}

pub async fn admin_reports(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    if !is_admin_session(&state, &headers) {
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

    let rows: Vec<crate::web::view_models::AdminReportRow> = db
        .prepare(
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
             ORDER BY
                CASE rr.status
                    WHEN 'pending' THEN 0
                    ELSE 1
                END,
                rr.created_at DESC,
                rr.id DESC",
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

        let report_badge = if report_status == "pending" {
            r#"<span style="color:#d97706;font-weight:800;">● Ожидает</span>"#
        } else {
            r#"<span style="color:#16a34a;font-weight:800;">✓ Закрыта</span>"#
        };

        let resource_status = if is_active == 1 {
            format!(
                r#"<span style="color:#16a34a;">● Активен · {}</span>"#,
                safe_moderation_status
            )
        } else {
            format!(
                r#"<span style="color:#dc2626;">● Скрыт · {}</span>"#,
                safe_moderation_status
            )
        };

        let close_button = if report_status == "pending" {
            format!(
                r#"
<form method="post"
      action="/app/admin/report/{report_id}/close{key_query}">
    <button type="submit"
            style="
                width:100%;
                min-height:44px;
                border-radius:12px;
                border:1px solid rgba(22,163,74,.35);
                background:rgba(22,163,74,.10);
                color:inherit;
                font-weight:800;
                cursor:pointer;
            ">
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
<article style="
    border:1px solid rgba(214,183,122,.22);
    border-radius:20px;
    padding:18px;
    margin-bottom:16px;
    background:rgba(255,255,255,.035);
">

    <div style="
        display:flex;
        justify-content:space-between;
        gap:12px;
        flex-wrap:wrap;
        align-items:flex-start;
    ">
        <div>
            <div style="
                font-size:11px;
                color:#8f96a3;
                margin-bottom:6px;
            ">
                Жалоба #{report_id} · ресурс #{resource_id}
            </div>

            <h2 style="margin:0 0 6px;font-size:20px;">
                {title}
            </h2>

            <div style="
                color:#9ca3af;
                font-size:13px;
            ">
                {category}
            </div>
        </div>

        <div style="
            display:flex;
            gap:10px;
            flex-wrap:wrap;
            font-size:12px;
        ">
            {report_badge}
            {resource_status}
        </div>
    </div>

    <div style="
        margin-top:16px;
        padding:14px;
        border-radius:14px;
        background:rgba(217,119,6,.07);
        border:1px solid rgba(217,119,6,.20);
        line-height:1.5;
    ">
        <strong>Причина:</strong><br>
        {reason}
    </div>

    <div style="
        margin-top:10px;
        font-size:11px;
        color:var(--muted);
    ">
        Reporter Telegram ID: {reporter_user_id}
        · created_at: {created_at}
    </div>

    <div style="
        display:grid;
        grid-template-columns:repeat(auto-fit,minmax(145px,1fr));
        gap:9px;
        margin-top:18px;
    ">

        {close_button}

        <form method="post"
              action="/app/admin/report/{report_id}/hide-resource{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.30);
                        background:rgba(220,38,38,.07);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                Скрыть ресурс
            </button>
        </form>

        <form method="post"
              action="/app/admin/report/{report_id}/reject-resource{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.40);
                        background:rgba(220,38,38,.12);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                ✕ Отклонить ресурс
            </button>
        </form>

        <a href="/app/resource/{resource_id}"
           target="_blank"
           style="
               min-height:42px;
               border-radius:12px;
               border:1px solid rgba(255,255,255,.12);
               display:flex;
               align-items:center;
               justify-content:center;
               text-decoration:none;
               color:inherit;
               font-weight:800;
           ">
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
<div class="card" style="display:block;">
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
            <div class="brand-sub">REPORT CENTER</div>
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

<div style="
    display:flex;
    gap:8px;
    flex-wrap:wrap;
    margin-bottom:20px;
">

    <a href="/app/admin/resources{key_query}"
       style="
           padding:9px 12px;
           border-radius:999px;
           text-decoration:none;
           border:1px solid rgba(255,255,255,.14);
           color:inherit;
           font-weight:700;
       ">
        Ресурсы
    </a>

    <a href="/app/admin/reports{key_query}"
       style="
           padding:9px 12px;
           border-radius:999px;
           text-decoration:none;
           border:1px solid rgba(217,119,6,.38);
           background:rgba(217,119,6,.08);
           color:inherit;
           font-weight:800;
       ">
        Жалобы
    </a>

</div>

<div style="
    display:grid;
    grid-template-columns:repeat(2,minmax(0,1fr));
    gap:10px;
    margin-bottom:24px;
">

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

    if !is_admin_session(&state, &headers) {
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

    if !is_admin_session(&state, &headers) {
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

    if !is_admin_session(&state, &headers) {
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
         ORDER BY
            is_verified ASC,
            is_active DESC,
            is_premium DESC,
            id DESC",
        filter_clause, search_clause
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

        let moderation_badge = match moderation_status.as_str() {
            "approved" => r#"<span style="color:#16a34a;font-weight:800;">✓ Одобрен</span>"#,
            "rejected" => r#"<span style="color:#dc2626;font-weight:800;">✕ Отклонён</span>"#,
            _ => r#"<span style="color:#d97706;font-weight:800;">● Ожидает проверки</span>"#,
        };

        let rejection_html =
            if moderation_status == "rejected" && !rejection_reason.trim().is_empty() {
                format!(
                    r#"<div style="
                        margin-top:12px;
                        padding:11px 13px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.22);
                        background:rgba(220,38,38,.07);
                        color:#dc2626;
                        font-size:13px;
                        line-height:1.45;
                    "><strong>Причина отказа:</strong> {}</div>"#,
                    safe_rejection_reason
                )
            } else {
                String::new()
            };

        let premium_badge = if premium == 1 {
            r#"<span style="color:#b88932;font-weight:800;">★ PREMIUM</span>"#
        } else {
            ""
        };

        let active_badge = if active == 1 {
            r#"<span style="color:#16a34a;">● Активен</span>"#
        } else {
            r#"<span style="color:#dc2626;">● Скрыт</span>"#
        };

        let premium_label = if premium == 1 {
            "Premium OFF"
        } else {
            "★ Premium ON"
        };

        let active_label = if active == 1 {
            "Скрыть"
        } else {
            "Вернуть"
        };

        cards.push_str(&format!(
            r#"
<article style="
    border:1px solid rgba(214,183,122,.22);
    border-radius:20px;
    padding:18px;
    margin:0 0 16px;
    background:rgba(255,255,255,.035);
    box-shadow:0 10px 30px rgba(0,0,0,.12);
">

    <div style="
        display:flex;
        justify-content:space-between;
        gap:14px;
        align-items:flex-start;
        flex-wrap:wrap;
    ">
        <div>
            <div style="font-size:11px;color:#8f96a3;margin-bottom:6px;">
                #{id} · {category}
            </div>

            <h2 style="margin:0 0 8px;font-size:20px;">
                {title}
            </h2>

            <div style="color:#9ca3af;line-height:1.5;max-width:620px;">
                {description}
            </div>
        </div>

        <div style="font-weight:800;white-space:nowrap;">
            ⭐ {rating:.1} · {votes}
        </div>
    </div>

    <div style="
        display:flex;
        gap:12px;
        flex-wrap:wrap;
        margin-top:14px;
        font-size:12px;
    ">
        {moderation_badge}
        {premium_badge}
        {active_badge}
    </div>

    {rejection_html}

    <div style="
        display:grid;
        grid-template-columns:repeat(auto-fit,minmax(145px,1fr));
        gap:9px;
        margin-top:18px;
    ">

        <form method="post"
              action="/app/admin/resource/{id}/approve{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(22,163,74,.35);
                        background:rgba(22,163,74,.10);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                ✓ Одобрить
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/reject{key_query}"
              style="
                  grid-column:1/-1;
                  display:grid;
                  grid-template-columns:minmax(0,1fr) auto;
                  gap:9px;
              ">

            <input
                type="text"
                name="reason"
                required
                maxlength="500"
                placeholder="Причина отклонения"
                style="
                    min-width:0;
                    min-height:44px;
                    box-sizing:border-box;
                    padding:0 13px;
                    border-radius:12px;
                    border:1px solid rgba(220,38,38,.30);
                    background:rgba(255,255,255,.04);
                    color:inherit;
                    font-size:14px;
                ">

            <button type="submit"
                    style="
                        min-height:44px;
                        padding:0 16px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.38);
                        background:rgba(220,38,38,.10);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                ✕ Отклонить
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/toggle-premium{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(214,183,122,.42);
                        background:rgba(214,183,122,.10);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                {premium_label}
            </button>
        </form>

        <form method="post"
              action="/app/admin/resource/{id}/toggle-active{key_query}">
            <button type="submit"
                    style="
                        width:100%;
                        min-height:44px;
                        border-radius:12px;
                        border:1px solid rgba(220,38,38,.28);
                        background:rgba(220,38,38,.07);
                        color:inherit;
                        font-weight:800;
                        cursor:pointer;
                    ">
                {active_label}
            </button>
        </form>

        <a href="/app/resource/{id}"
           target="_blank"
           style="
               min-height:42px;
               border-radius:12px;
               border:1px solid rgba(255,255,255,.12);
               display:flex;
               align-items:center;
               justify-content:center;
               text-decoration:none;
               color:inherit;
               font-weight:800;
           ">
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

    let safe_q = templates::escape_html(q);
    let safe_filter = templates::escape_html(filter);

    let main_html = format!(
        r####"<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">ResursMap</div>
            <div class="brand-sub">MODERATION CENTER</div>
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
        Проверка, Premium и видимость ресурсов.
    </p>

</section>

<form method="get"
      action="/app/admin/resources"
      style="display:flex;gap:8px;margin-bottom:16px;">

    <input type="hidden" name="filter" value="{filter}">

    <input
        type="search"
        name="q"
        value="{q}"
        placeholder="Поиск по названию, категории, описанию..."
        style="
            flex:1;
            min-width:0;
            padding:13px 14px;
            border-radius:14px;
            border:1px solid rgba(255,255,255,.14);
            background:rgba(255,255,255,.04);
            color:inherit;
            font-size:15px;
        "
    >

    <button type="submit"
            style="
                min-width:92px;
                border-radius:14px;
                border:1px solid rgba(214,183,122,.35);
                background:rgba(214,183,122,.10);
                color:inherit;
                font-weight:800;
                cursor:pointer;
            ">
        Найти
    </button>

</form>

<div style="
    display:flex;
    gap:8px;
    flex-wrap:wrap;
    margin-bottom:20px;
">

    <a href="/app/admin/reports{key_query}"
       style="
           padding:9px 12px;
           border-radius:999px;
           text-decoration:none;
           border:1px solid rgba(220,38,38,.38);
           background:rgba(220,38,38,.08);
           color:inherit;
           font-weight:800;
       ">
        🚩 Жалобы ({pending_reports_count})
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=all"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(255,255,255,.14);color:inherit;font-weight:700;">
        Все
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=pending"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(217,119,6,.35);color:inherit;font-weight:700;">
        Ожидают
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=verified"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(22,163,74,.35);color:inherit;font-weight:700;">
        Проверены
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=rejected"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(220,38,38,.35);color:inherit;font-weight:700;">
        Отклонённые
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=premium"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(214,183,122,.45);color:inherit;font-weight:700;">
        Premium
    </a>

    <a href="/app/admin/resources{key_query}{key_join}filter=hidden"
       style="padding:9px 12px;border-radius:999px;text-decoration:none;
              border:1px solid rgba(220,38,38,.32);color:inherit;font-weight:700;">
        Скрытые
    </a>

</div>

<div style="
    display:grid;
    grid-template-columns:repeat(auto-fit,minmax(140px,1fr));
    gap:10px;
    margin-bottom:24px;
">

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
            <div class="card-meta">Premium</div>
        </div>
    </div>

</div>

<section style="margin-bottom:20px;">

    <div style="
        display:flex;
        align-items:center;
        justify-content:space-between;
        gap:12px;
        flex-wrap:wrap;
        margin-bottom:12px;
    ">
        <strong>Найдено: {result_count}</strong>
        <span style="font-size:12px;color:var(--muted);">
            Массовые действия применяются к текущей выборке
        </span>
    </div>

    <div style="
        display:grid;
        grid-template-columns:repeat(auto-fit,minmax(150px,1fr));
        gap:8px;
    ">

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=verify">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(22,163,74,.35);
                           background:rgba(22,163,74,.10);
                           color:inherit;font-weight:800;cursor:pointer;">
                ✓ Одобрить найденные
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=unverify">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(217,119,6,.35);
                           background:rgba(217,119,6,.08);
                           color:inherit;font-weight:800;cursor:pointer;">
                Снять проверку
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=premium">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(214,183,122,.42);
                           background:rgba(214,183,122,.10);
                           color:inherit;font-weight:800;cursor:pointer;">
                ★ Premium ON
            </button>
        </form>

        <form method="post"
              action="/app/admin/resources/bulk{key_query}{key_join}filter={filter}&q={q}&action=hide">
            <button type="submit"
                    style="width:100%;min-height:44px;border-radius:12px;
                           border:1px solid rgba(220,38,38,.30);
                           background:rgba(220,38,38,.07);
                           color:inherit;font-weight:800;cursor:pointer;">
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
        r#"<section class="card" style="display:block;padding:20px;">
    <div class="card-title">
        Изменено ресурсов: {changed}
    </div>

    <div class="card-meta" style="margin-top:8px;">
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
            if let Some(user_id) = telegram_owner_user_id(&client_id) {
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
            if let Some(user_id) = telegram_owner_user_id(&client_id) {
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
