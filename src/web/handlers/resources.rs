use super::auth::{verify_authenticated_user, verify_user_session};
use super::common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
};
use super::types::{AddResourceForm, EditResourceForm, ReportResourcePayload};
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use rusqlite::OptionalExtension;
use serde_json::json;
use std::collections::BTreeMap;

pub async fn app_cat(
    State(state): State<AppState>,
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Html<String> {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let listing_type = match params.get("type").map(|value| value.trim()) {
        Some("offer") => Some("offer"),
        Some("seeker") => Some("seeker"),
        _ => None,
    };
    let sort = match params.get("sort").map(|value| value.trim()) {
        Some("new") => "new",
        _ => "rating",
    };
    let rubric_id = params
        .get("rubric")
        .and_then(|value| crate::catalog::by_id(value.trim()))
        .map(|rubric| rubric.id);

    let city_id: Option<i64> = db
        .query_row(
            "SELECT city.id
             FROM geo_cities AS city
             WHERE city.legacy_continent_index = ?1
               AND city.legacy_country_index = ?2
               AND city.legacy_city_index = ?3
               AND city.is_active = 1
             LIMIT 1",
            rusqlite::params![ci as i64, si as i64, zi as i64],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten();

    let order_sql = if sort == "new" {
        "ORDER BY id DESC"
    } else {
        "ORDER BY is_verified DESC, rating DESC, votes DESC, id DESC"
    };
    let list_sql = format!(
        "SELECT id, title, description, contact, address, rating, votes, is_verified, is_premium,
                COALESCE(listing_type, 'general'), COALESCE(rubric, '')
         FROM resources
         WHERE (
                (continent_index = ?1 AND country_index = ?2 AND city_index = ?3)
                OR (?6 IS NOT NULL AND city_id = ?6)
              )
           AND (
                LOWER(?4) = 'all'
                OR (LOWER(?4) = 'business' AND category IN ('business', 'services'))
                OR category = ?4
           )
           AND (?5 IS NULL OR listing_type = ?5)
           AND (?7 IS NULL OR rubric = ?7)
           AND is_active = 1
           AND moderation_status = 'approved'
         {order_sql}"
    );
    let resources: Vec<crate::web::view_models::CategoryResourceRow> = db
        .prepare(&list_sql)
        .and_then(|mut stmt| {
            stmt.query_map(
                rusqlite::params![ci, si, zi, k, listing_type, city_id, rubric_id],
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
                        row.get(9)?,
                        row.get(10)?,
                    ))
                },
            )?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let include_people = listing_type == Some("seeker") || rubric_id.is_some();
    let people: Vec<crate::web::view_models::SearchPersonRow> = if include_people {
        db.prepare(
            "SELECT
                    p.public_id,
                    p.username,
                    p.first_name,
                    p.last_name,
                    p.category,
                    p.open_contact,
                    p.intent_text,
                    p.intent_until,
                    p.last_seen_at
                 FROM profiles p
                 JOIN users u
                   ON u.id = p.user_id
                  AND u.is_active = 1
                 WHERE p.public_id <> ''
                   AND p.home_continent_index = ?1
                   AND p.home_country_index = ?2
                   AND p.home_city_index = ?3
                   AND (?4 IS NULL OR p.category = ?4)
                   AND (
                        trim(p.category) <> ''
                        OR (
                            trim(p.intent_text) <> ''
                            AND (
                                p.intent_until = 0
                                OR p.intent_until >= strftime('%s','now')
                            )
                        )
                   )
                 ORDER BY
                    CASE
                        WHEN p.intent_text <> ''
                         AND (
                              p.intent_until = 0
                              OR p.intent_until >= strftime('%s','now')
                         )
                        THEN 0
                        ELSE 1
                    END,
                    CASE
                        WHEN trim(p.category) <> '' THEN 0
                        ELSE 1
                    END,
                    p.updated_at DESC
                 LIMIT 50",
        )
        .and_then(|mut stmt| {
            stmt.query_map(
                rusqlite::params![ci as i64, si as i64, zi as i64, rubric_id],
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
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    drop(db);

    let mut resources = resources;
    if sort == "new" {
        resources.sort_by_key(|a| std::cmp::Reverse(a.0));
    } else {
        resources.sort_by(|a, b| {
            b.8.cmp(&a.8)
                .then_with(|| b.7.cmp(&a.7))
                .then_with(|| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| b.6.cmp(&a.6))
        });
    }

    Html(templates::render_category(
        templates::RenderCategoryParams {
            ci,
            si,
            zi,
            category: &k,
            listing_type,
            active_rubric: rubric_id,
            sort,
            resources,
            people,
        },
    ))
}

pub async fn app_city_all(
    state: State<AppState>,
    Path((ci, si, zi)): Path<(usize, usize, usize)>,
    params: Query<BTreeMap<String, String>>,
) -> Html<String> {
    app_cat(state, Path((ci, si, zi, "all".to_string())), params).await
}

pub async fn resource_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Html<String> {
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
                COALESCE(p.public_id, ''),
                COALESCE(r.listing_type, 'general'),
                r.continent_index,
                r.country_index,
                r.city_index,
                r.client_id,
                r.moderation_status,
                r.is_active,
                COALESCE(r.rubric, '')
         FROM resources r
         LEFT JOIN profiles p
           ON p.client_id = r.client_id
         WHERE r.id = ?1",
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
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, i64>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, String>(16)?,
                    row.get::<_, i64>(17)?,
                    row.get::<_, String>(18)?,
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
            listing_type,
            continent_index,
            country_index,
            city_index,
            owner_client_id,
            moderation_status,
            is_active,
            rubric,
        )) => {
            let is_public = is_active != 0 && moderation_status == "approved";
            let is_owner = verify_authenticated_user(&state, &headers)
                .is_some_and(|user| !owner_client_id.is_empty() && user.client_id == owner_client_id);

            if !is_public && !is_owner {
                return Html(templates::status_page(
                    "Ресурс не найден · ResursMap",
                    "⚠ ResursMap",
                    "Ресурс не найден",
                    "Этот ресурс ещё на проверке, скрыт или был удалён.",
                    &templates::navigation_card(
                        "/app",
                        "map",
                        "Вернуться к городам",
                        "Открыть ResursMap",
                    ),
                ));
            }

            Html(templates::render_resource_profile(
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
                    listing_type: &listing_type,
                    continent_index,
                    country_index,
                    city_index,
                    _created_at: created_at,
                    owner_public_id: &owner_public_id,
                    rubric: &rubric,
                    owner_preview: !is_public,
                    moderation_status: &moderation_status,
                    is_active,
                },
            ))
        }

        None => Html(templates::status_page(
            "Ресурс не найден · ResursMap",
            "⚠ ResursMap",
            "Ресурс не найден",
            "Этот ресурс больше недоступен или был удалён.",
            &templates::navigation_card("/app", "map", "Вернуться к городам", "Открыть ResursMap"),
        )),
    }
}

pub async fn add_resource_page(
    State(state): State<AppState>,
    Path((ci, si, zi, k)): Path<(usize, usize, usize, String)>,
    Query(params): Query<BTreeMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    if verify_authenticated_user(&state, &headers).is_none() {
        let category = k.trim();
        let category_url = urlencoding::encode(category);
        let mut next = format!("/app/{ci}/{si}/{zi}/cat/{category_url}/add");
        if let Some(query) = listing_add_query(&params) {
            next.push('?');
            next.push_str(&query);
        }

        return Redirect::temporary(&format!("/login?next={}", urlencoding::encode(&next)))
            .into_response();
    }

    let listing_type = match params.get("type").map(|value| value.trim()) {
        Some("seeker") => Some("seeker"),
        Some("offer") => Some("offer"),
        _ => None,
    };
    let rubric = params
        .get("rubric")
        .and_then(|value| crate::catalog::by_id(value.trim()));

    if let Some(rubric) = rubric {
        return Html(templates::render_add_resource(
            ci,
            si,
            zi,
            k.trim(),
            listing_type,
            rubric,
            None,
            None,
        ))
        .into_response();
    }

    Html(templates::render_add_rubric_picker(
        ci,
        si,
        zi,
        k.trim(),
        listing_type,
    ))
    .into_response()
}

fn listing_add_query(params: &BTreeMap<String, String>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(kind) = params.get("type").map(|value| value.trim()) {
        if matches!(kind, "offer" | "seeker") {
            parts.push(format!("type={}", urlencoding::encode(kind)));
        }
    }
    if let Some(rubric) = params
        .get("rubric")
        .and_then(|value| crate::catalog::by_id(value.trim()))
    {
        parts.push(format!("rubric={}", urlencoding::encode(rubric.id)));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("&"))
    }
}

#[allow(clippy::too_many_arguments)]
fn add_form_error(
    status: StatusCode,
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
    form: &AddResourceForm,
    rubric: &crate::catalog::Rubric,
    message: &str,
) -> Response {
    let listing_type = match form.listing_type.as_deref().map(str::trim) {
        Some("seeker") => Some("seeker"),
        Some("offer") => Some("offer"),
        _ => None,
    };
    (
        status,
        Html(templates::render_add_resource(
            ci,
            si,
            zi,
            category,
            listing_type,
            rubric,
            Some(templates::AddResourceDraft {
                title: form.title.trim(),
                description: form.description.trim(),
                contact: form.contact.trim(),
                address: form.address.trim(),
            }),
            Some(message),
        )),
    )
        .into_response()
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

    let requested_rubric = form
        .rubric
        .as_deref()
        .and_then(|value| crate::catalog::by_id(value.trim()));
    let rubric = match requested_rubric {
        Some(rubric) => rubric,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Html("<h1>400</h1><p>Выберите рубрику из списка.</p>".to_string()),
            )
                .into_response();
        }
    };

    let category = crate::catalog::resource_category_for(rubric);
    let section_ok = matches!(
        (k.trim().to_ascii_lowercase().as_str(), rubric.kind),
        ("work", crate::catalog::RubricKind::Work)
            | ("business" | "services", crate::catalog::RubricKind::Business)
    );
    if !section_ok {
        return add_form_error(
            StatusCode::BAD_REQUEST,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Рубрика не подходит к этому разделу.",
        );
    }

    let title = form.title.trim();
    let description = form.description.trim();
    let contact = form.contact.trim();
    let address = form.address.trim();

    if !input_text_is_valid(category, 1, 100) {
        return add_form_error(
            StatusCode::BAD_REQUEST,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Некорректная категория.",
        );
    }

    let category_url = urlencoding::encode(category);

    let (owner_user_id, owner_client_id) =
        if let Some(user) = verify_authenticated_user(&state, &headers) {
            (user.user_id, user.client_id)
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

    if !input_text_is_valid(title, 1, 120) {
        return add_form_error(
            StatusCode::BAD_REQUEST,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Название должно содержать от 1 до 120 символов.",
        );
    }

    if !input_text_is_valid(description, 1, 1000) {
        return add_form_error(
            StatusCode::BAD_REQUEST,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Описание должно содержать от 1 до 1000 символов.",
        );
    }

    if contact.is_empty() || !input_text_is_valid(contact, 1, 120) {
        return add_form_error(
            StatusCode::BAD_REQUEST,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Укажите телефон, Telegram или другой контакт.",
        );
    }

    if !input_text_is_valid(address, 0, 250) {
        return add_form_error(
            StatusCode::BAD_REQUEST,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Адрес слишком длинный или содержит недопустимые символы.",
        );
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, owner_user_id, "resource_add", 10, 3600).await
    {
        let mut response = add_form_error(
            StatusCode::TOO_MANY_REQUESTS,
            ci,
            si,
            zi,
            k.trim(),
            &form,
            rubric,
            "Слишком много добавлений. Попробуйте позже.",
        );
        response.headers_mut().insert(
            header::RETRY_AFTER,
            header::HeaderValue::from_str(&retry_after.to_string())
                .unwrap_or_else(|_| header::HeaderValue::from_static("3600")),
        );
        return response;
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

    let listing_type = crate::catalog::listing_type_for_intent(
        rubric.kind,
        form.listing_type.as_deref(),
    );

    let city_id: Option<i64> = db
        .query_row(
            "SELECT city.id
             FROM geo_cities AS city
             WHERE city.legacy_continent_index = ?1
               AND city.legacy_country_index = ?2
               AND city.legacy_city_index = ?3
               AND city.is_active = 1
             LIMIT 1",
            rusqlite::params![ci, si, zi],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten();

    let result = db.execute(
        "INSERT INTO resources
        (client_id, continent_index, country_index, city_index, city_id,
         category, title, description, contact, address,
         rating, votes, is_premium, is_verified, is_active,
         moderation_status, rejection_reason, listing_type, rubric,
         created_at, updated_at)
        VALUES
        (?9, ?1, ?2, ?3, ?11, ?4, ?5, ?6, ?7, ?8,
         0, 0, 0, 0, 1,
         'pending', '', ?10, ?12,
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
            listing_type,
            city_id,
            rubric.id,
        ],
    );

    let inserted_id = match &result {
        Ok(_) => db.last_insert_rowid(),
        Err(_) => 0,
    };

    drop(db);

    match result {
        Ok(_) => {
            let preview_href = if inserted_id > 0 {
                format!("/app/resource/{inserted_id}")
            } else {
                "/app/my-resources".to_string()
            };
            let actions = format!(
                "{}{}",
                templates::navigation_card(
                    &preview_href,
                    "map-pin",
                    "Открыть моё объявление",
                    "Пока видите только вы",
                ),
                templates::navigation_card(
                    "/app/my-resources",
                    "user",
                    "Мои ресурсы",
                    "Статус проверки",
                ),
            );

            Html(templates::status_page(
                "На проверке · ResursMap",
                "Проверка",
                "Объявление на проверке",
                "Другие участники его пока не видят. После одобрения оно появится в поиске и в городе.",
                &actions,
            ))
            .into_response()
        }
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
                is_active,
                COALESCE(listing_type, 'general'),
                COALESCE(rubric, '')
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
                    row.get(11)?,
                    row.get(12)?,
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
            "SELECT title, description, contact, address, category, client_id,
                    COALESCE(listing_type, 'general'), COALESCE(rubric, '')
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
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .ok();

    drop(db);

    match resource {
        Some((title, description, contact, address, category, owner, listing_type, rubric))
            if !client_id.is_empty() && owner == client_id =>
        {
            Html(templates::render_edit_resource(
                templates::RenderEditResourceParams {
                    id,
                    title: &title,
                    description: &description,
                    contact: &contact,
                    address: &address,
                    category: &category,
                    listing_type: &listing_type,
                    rubric: &rubric,
                },
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

    let current_rubric: String = db
        .query_row(
            "SELECT COALESCE(rubric, '')
             FROM resources
             WHERE id = ?1
               AND client_id = ?2",
            rusqlite::params![id, &owner_client_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    let rubric = form
        .rubric
        .as_deref()
        .and_then(|value| crate::catalog::by_id(value.trim()))
        .or_else(|| crate::catalog::by_id(&current_rubric));

    let Some(rubric) = rubric else {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>400</h1><p>Выберите рубрику из списка.</p>".to_string()),
        )
            .into_response();
    };

    let listing_type =
        crate::catalog::listing_type_for_intent(rubric.kind, form.listing_type.as_deref());
    let category = crate::catalog::resource_category_for(rubric);

    let changed = db.execute(
            "UPDATE resources
         SET title = ?1,
             description = ?2,
             contact = ?3,
             address = ?4,
             listing_type = ?7,
             rubric = ?8,
             category = ?9,
             is_verified = 0,
             is_active = 1,
             moderation_status = 'pending',
             rejection_reason = '',
             city_id = COALESCE(
                (SELECT city.id
                 FROM geo_cities AS city
                 WHERE city.legacy_continent_index = resources.continent_index
                   AND city.legacy_country_index = resources.country_index
                   AND city.legacy_city_index = resources.city_index
                   AND city.is_active = 1
                 LIMIT 1),
                city_id
             ),
             updated_at = strftime('%s','now')
         WHERE id = ?5
           AND client_id = ?6",
            rusqlite::params![
                title,
                description,
                contact,
                address,
                id,
                &owner_client_id,
                listing_type,
                rubric.id,
                category,
            ],
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
