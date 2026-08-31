use super::auth::{verify_authenticated_user, verify_telegram_init_data, verify_user_session};
use super::common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
};
use super::types::{AddResourceForm, EditResourceForm, ReportResourcePayload};
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

pub async fn app_cat(
    State(state): State<AppState>,
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
) -> Html<String> {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resources: Vec<crate::web::view_models::CategoryResourceRow> = db
        .prepare(
            "SELECT id, title, description, contact, address, rating, votes, is_verified, is_premium
             FROM resources
             WHERE continent_index = ?1
               AND country_index = ?2
               AND city_index = ?3
               AND category = ?4
               AND is_active = 1
               AND moderation_status = 'approved'
             ORDER BY is_verified DESC, rating DESC, votes DESC, id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(
                rusqlite::params![ci, si, zi, k],
                |row| {
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
                    ))
                },
            )?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    // Premium → проверенные → рейтинг → голоса
    let mut resources = resources;
    resources.sort_by(|a, b| {
        b.8.cmp(&a.8)
            .then_with(|| b.7.cmp(&a.7))
            .then_with(|| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| b.6.cmp(&a.6))
    });

    Html(templates::render_category(ci, si, zi, &k, resources))
}

pub async fn resource_profile(State(state): State<AppState>, Path(id): Path<i64>) -> Html<String> {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resource = db
        .query_row(
            "SELECT
                r.title,
                r.description,
                r.contact,
                r.address,
                r.rating,
                r.votes,
                r.is_premium,
                r.is_verified,
                r.category,
                r.created_at,
                COALESCE(p.public_id, '')
         FROM resources r
         LEFT JOIN profiles p
           ON p.client_id = r.client_id
         WHERE r.id = ?1
           AND r.is_active = 1
           AND r.moderation_status = 'approved'",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, String>(10)?,
                ))
            },
        )
        .ok();

    drop(db);

    match resource {
        Some((
            title,
            description,
            contact,
            address,
            rating,
            votes,
            premium,
            verified,
            category,
            created_at,
            owner_public_id,
        )) => Html(templates::render_resource_profile(
            templates::RenderResourceProfileParams {
                id,
                title: &title,
                description: &description,
                contact: &contact,
                address: &address,
                rating,
                votes,
                premium,
                verified,
                category: &category,
                _created_at: created_at,
                owner_public_id: &owner_public_id,
            },
        )),

        None => Html(templates::status_page(
            "Ресурс не найден · ResursMap",
            "⚠ ResursMap",
            "Ресурс не найден",
            "Этот ресурс больше недоступен или был удалён.",
            &templates::navigation_card("/app", "map", "Вернуться на карту", "Открыть ResursMap"),
        )),
    }
}

pub async fn add_resource_page(
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
) -> Html<String> {
    Html(templates::render_add_resource(ci, si, zi, &k))
}

pub async fn add_resource(
    State(state): State<AppState>,
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
    headers: HeaderMap,
    Form(form): Form<AddResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    let category = k.trim();
    let title = form.title.trim();
    let description = form.description.trim();
    let contact = form.contact.trim();
    let address = form.address.trim();
    let init_data = form.init_data.trim();

    if !input_text_is_valid(category, 1, 100) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Некорректная категория.</p>".to_string()),
        )
            .into_response();
    }

    if !input_text_is_valid(title, 1, 120) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Название должно содержать от 1 до 120 символов.</p>".to_string()),
        )
            .into_response();
    }

    if !input_text_is_valid(description, 1, 1000) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Описание должно содержать от 1 до 1000 символов.</p>".to_string()),
        )
            .into_response();
    }

    if !input_text_is_valid(contact, 0, 120) {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>400</h1><p>Контакт слишком длинный или содержит недопустимые символы.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    if !input_text_is_valid(address, 0, 250) {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>400</h1><p>Адрес слишком длинный или содержит недопустимые символы.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    // Telegram initData обычно намного меньше.
    // Ограничение защищает endpoint от бессмысленно огромного значения.
    if !input_text_is_valid(init_data, 0, 8192) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Некорректные данные Telegram.</p>".to_string()),
        )
            .into_response();
    }

    let category_url = urlencoding::encode(category);

    let (owner_user_id, owner_client_id) =
        if let Some(user) = verify_authenticated_user(&state, &headers) {
            (user.user_id, user.client_id)
        } else if !init_data.is_empty() {
            match verify_telegram_init_data(init_data, &state.bot_token) {
                Some(user_id) => (user_id, format!("tg:{}", user_id)),
                None => (0, String::new()),
            }
        } else {
            (0, String::new())
        };

    if owner_user_id <= 0 || owner_client_id.is_empty() {
        return Html(templates::status_page(
            "Требуется вход · ResursMap",
            "⚠ Авторизация",
            "Не удалось подтвердить пользователя",
            "Войдите в аккаунт и попробуйте добавить ресурс снова.",
            &templates::navigation_card("/login", "user", "Войти в аккаунт", ""),
        ))
        .into_response();
    }

    if owner_user_id <= 0 {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<h1>401</h1><p>Не удалось определить пользователя.</p>".to_string()),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, owner_user_id, "resource_add", 10, 3600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Html(
                "<h1>429</h1><p>Слишком много добавлений ресурсов. Попробуйте позже.</p>"
                    .to_string(),
            ),
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
        "INSERT INTO resources
        (client_id, continent_index, country_index, city_index,
         category, title, description, contact, address,
         rating, votes, is_premium, is_verified, is_active,
         moderation_status, rejection_reason,
         created_at, updated_at)
        VALUES
        (?9, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
         0, 0, 0, 0, 1,
         'pending', '',
         strftime('%s','now'), strftime('%s','now'))",
        rusqlite::params![
            ci,
            si,
            zi,
            category,
            title,
            description,
            contact,
            address,
            owner_client_id,
        ],
    );

    drop(db);

    match result {
        Ok(_) => Html(templates::status_page(
            "Ресурс добавлен · ResursMap",
            "✓ Готово",
            "Ресурс добавлен",
            "Спасибо! Ресурс появился в категории и ожидает проверки.",
            &templates::navigation_card(
                &format!("/app/{}/{}/{}/cat/{}", ci, si, zi, category_url),
                "map",
                "Вернуться к ресурсам",
                "Открыть категорию",
            ),
        ))
        .into_response(),
        Err(_) => Html(templates::status_page(
            "Ошибка · ResursMap",
            "⚠ Ошибка",
            "Не удалось добавить ресурс",
            "Попробуйте ещё раз.",
            &format!(
                r#"<a href="/app/{}/{}/{}/cat/{}">Назад</a>"#,
                ci, si, zi, category_url,
            ),
        ))
        .into_response(),
    }
}

pub async fn my_resources(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let client_id = match verify_authenticated_user(&state, &headers) {
        Some(user) => user.client_id,
        None => String::new(),
    };

    if client_id.is_empty() {
        return Html(templates::render_my_resources("", vec![]));
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resources: Vec<crate::web::view_models::MyResourceRow> = db
        .prepare(
            "SELECT
                id,
                title,
                category,
                description,
                rating,
                votes,
                is_verified,
                is_premium,
                moderation_status,
                rejection_reason,
                is_active
             FROM resources
             WHERE client_id = ?1
             ORDER BY is_active DESC,
                      is_premium DESC,
                      updated_at DESC,
                      id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![&client_id], |row| {
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
        .unwrap_or_default();

    drop(db);

    Html(templates::render_my_resources(&client_id, resources))
}

pub async fn edit_resource_page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Html<String> {
    let client_id = match verify_authenticated_user(&state, &headers) {
        Some(user) => user.client_id,
        None => String::new(),
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resource = db
        .query_row(
            "SELECT title, description, contact, address, category, client_id
         FROM resources
         WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .ok();

    drop(db);

    match resource {
        Some((title, description, contact, address, category, owner))
            if !client_id.is_empty() && owner == client_id =>
        {
            Html(templates::render_edit_resource(
                id,
                &title,
                &description,
                &contact,
                &address,
                &category,
            ))
        }

        _ => Html(templates::status_page(
            "Нет доступа · ResursMap",
            "⚠ Доступ",
            "Редактирование недоступно",
            "Этот ресурс не принадлежит текущему пользователю.",
            &templates::navigation_card("/app/me", "user", "Вернуться в профиль", ""),
        )),
    }
}

pub async fn edit_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<EditResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"<h1>403</h1><p>Запрос отклонён.</p>"#.to_string()),
        )
            .into_response();
    }

    let title = form.title.trim();
    let description = form.description.trim();
    let contact = form.contact.trim();
    let address = form.address.trim();

    if !input_text_is_valid(title, 1, 120)
        || !input_text_is_valid(description, 1, 1000)
        || !input_text_is_valid(contact, 0, 120)
        || !input_text_is_valid(address, 0, 250)
    {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Проверьте длину и содержимое полей ресурса.</p>".to_string()),
        )
            .into_response();
    }

    let owner = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return Html(templates::status_page(
                "Нет доступа · ResursMap",
                "⚠ Доступ",
                "Не удалось подтвердить владельца",
                "Войдите в аккаунт и попробуйте снова.",
                &templates::navigation_card("/login", "user", "Войти в аккаунт", ""),
            ))
            .into_response();
        }
    };

    let owner_user_id = owner.user_id;
    let owner_client_id = owner.client_id;

    if owner_user_id <= 0 {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<h1>401</h1><p>Не удалось определить пользователя.</p>".to_string()),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, owner_user_id, "resource_edit", 30, 3600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Html(
                "<h1>429</h1><p>Слишком много изменений ресурсов. Попробуйте позже.</p>"
                    .to_string(),
            ),
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

    let changed = db
        .execute(
            "UPDATE resources
         SET title = ?1,
             description = ?2,
             contact = ?3,
             address = ?4,
             is_verified = 0,
             is_active = 1,
             moderation_status = 'pending',
             rejection_reason = '',
             updated_at = strftime('%s','now')
         WHERE id = ?5
           AND client_id = ?6",
            rusqlite::params![title, description, contact, address, id, &owner_client_id,],
        )
        .unwrap_or(0);

    drop(db);

    if changed == 1 {
        Html(templates::status_page(
            "Изменения сохранены · ResursMap",
            "✓ Готово",
            "Изменения сохранены",
            "После изменения ресурс снова ожидает проверки.",
            &templates::navigation_card("/app/my-resources", "map", "Вернуться в мои ресурсы", ""),
        ))
        .into_response()
    } else {
        Html(templates::status_page(
            "Нет доступа · ResursMap",
            "⚠ Ошибка",
            "Не удалось сохранить",
            "Проверьте владельца ресурса.",
            "",
        ))
        .into_response()
    }
}

pub async fn api_report_resource(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ReportResourcePayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let reason = payload.reason.trim();

    if !input_text_is_valid(reason, 3, 500) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_reason"
            })),
        )
            .into_response();
    }

    let resource_exists: bool = {
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

        db.query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM resources
                WHERE id = ?1
                  AND is_active = 1
                  AND moderation_status = 'approved'
            )",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .unwrap_or(false)
    };

    if !resource_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "resource_not_found"
            })),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "resource_report", 6, 3600).await
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
        "INSERT INTO resource_reports (
            reporter_user_id,
            resource_id,
            reason,
            status,
            created_at,
            updated_at
        )
        VALUES (
            ?1,
            ?2,
            ?3,
            'pending',
            strftime('%s','now'),
            strftime('%s','now')
        )

        ON CONFLICT(reporter_user_id, resource_id)
        DO UPDATE SET
            reason = excluded.reason,
            status = 'pending',
            updated_at = strftime('%s','now')",
        rusqlite::params![user_id, id, reason,],
    );

    drop(db);

    match result {
        Ok(_) => Json(json!({
            "ok": true,
            "status": "pending"
        }))
        .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "database_error",
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn api_resource_vote(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    let user_id = user.user_id;
    let client_id = user.client_id;

    let score = payload.get("score").and_then(|v| v.as_i64()).unwrap_or(0);

    if !(1..=5).contains(&score) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_vote"
            })),
        )
            .into_response();
    }

    let allowed: bool = {
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

        db.query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM resources
                WHERE id = ?1
                  AND is_active = 1
                  AND moderation_status = 'approved'
            )",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .unwrap_or(false)
    };

    if !allowed {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "resource_not_found"
            })),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "resource_rating", 60, 60).await
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

    let transaction_result = (|| -> rusqlite::Result<(f64, i64)> {
        let tx = db.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO resource_votes
                (resource_id, client_id, score, updated_at)
             VALUES
                (?1, ?2, ?3, strftime('%s','now'))
             ON CONFLICT(resource_id, client_id)
             DO UPDATE SET
                score = excluded.score,
                updated_at = excluded.updated_at",
            rusqlite::params![id, client_id, score],
        )?;

        let stats: (f64, i64) = tx.query_row(
            "SELECT
                COALESCE(AVG(score), 0),
                COUNT(*)
             FROM resource_votes
             WHERE resource_id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        tx.execute(
            "UPDATE resources
             SET rating = ?1,
                 votes = ?2,
                 updated_at = strftime('%s','now')
             WHERE id = ?3",
            rusqlite::params![stats.0, stats.1, id],
        )?;

        tx.commit()?;
        Ok(stats)
    })();

    let stats = match transaction_result {
        Ok(stats) => stats,

        Err(err) => {
            drop(db);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": "vote_write_failed",
                    "message": err.to_string()
                })),
            )
                .into_response();
        }
    };

    drop(db);

    Json(json!({
        "ok": true,
        "resource_id": id,
        "rating": stats.0,
        "votes": stats.1
    }))
    .into_response()
}
