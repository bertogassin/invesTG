use super::auth::verify_authenticated_user;
use super::common::{input_text_is_valid, rate_limit_retry_after, request_is_cross_site};
use super::types::AddResourceForm;
use crate::resource_screening::screen_listing_content;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use rusqlite::OptionalExtension;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct CreateCityQuery {
    pub q: Option<String>,
    pub kind: Option<String>,
    pub intent: Option<String>,
    pub rubric: Option<String>,
}

#[allow(clippy::result_large_err)]
fn authenticated_or_login(
    state: &AppState,
    headers: &HeaderMap,
    next: &str,
) -> Result<crate::web::handlers::auth::AuthenticatedUser, Response> {
    verify_authenticated_user(state, headers).ok_or_else(|| {
        Redirect::temporary(&format!("/login?next={}", urlencoding::encode(next))).into_response()
    })
}

fn create_page(
    title: &str,
    eyebrow: &str,
    description: &str,
    back_href: &str,
    back_label: &str,
    body: &str,
) -> String {
    let back = if back_href.is_empty() {
        String::new()
    } else {
        templates::back_navigation_card(back_href, back_label, "Вернуться назад")
    };

    let main = format!(
        r#"
<section class="hero">
    <div class="eyebrow">{eyebrow}</div>
    <h1>{title}</h1>
    <p>{description}</p>
</section>
{back}
<section>
{body}
</section>
"#,
        eyebrow = templates::escape_html(eyebrow),
        title = templates::escape_html(title),
        description = templates::escape_html(description),
        back = back,
        body = body,
    );

    templates::page_document(&format!("{title} · ResursMap"), "", "", &main, "", "")
}

fn kind_from_query(value: Option<&str>) -> Option<crate::catalog::RubricKind> {
    match value.map(str::trim) {
        Some("work") => Some(crate::catalog::RubricKind::Work),
        Some("business") => Some(crate::catalog::RubricKind::Business),
        _ => None,
    }
}

fn intent_from_query(value: Option<&str>) -> Option<&'static str> {
    match value.map(str::trim) {
        Some("offer") => Some("offer"),
        Some("seeker") => Some("seeker"),
        Some("general") => Some("general"),
        _ => None,
    }
}

fn selection_is_valid(kind: Option<crate::catalog::RubricKind>, intent: Option<&str>) -> bool {
    matches!(
        (kind, intent),
        (
            Some(crate::catalog::RubricKind::Work),
            Some("offer" | "seeker")
        ) | (Some(crate::catalog::RubricKind::Business), Some("general"))
    )
}

fn create_city_href(city_id: i64, kind: &str, intent: &str, rubric: Option<&str>) -> String {
    let mut href = format!(
        "/app/add/city/{city_id}?kind={}&intent={}",
        urlencoding::encode(kind),
        urlencoding::encode(intent),
    );

    if let Some(rubric) = rubric {
        href.push_str("&rubric=");
        href.push_str(&urlencoding::encode(rubric));
    }

    href
}

pub async fn resource_create_start(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = authenticated_or_login(&state, &headers, "/app/add") {
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

    let continents = db
        .prepare(
            "SELECT continent.id,
                    continent.name_ru,
                    COUNT(country.id)
             FROM geo_continents AS continent
             LEFT JOIN geo_countries AS country
               ON country.continent_id = continent.id
              AND country.is_active = 1
             WHERE continent.is_active = 1
             GROUP BY continent.id, continent.name_ru
             ORDER BY continent.name_ru COLLATE NOCASE",
        )
        .and_then(|mut statement| {
            statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let cards = continents
        .iter()
        .map(|(id, name, count)| {
            templates::navigation_card(
                &format!("/app/add/continent/{id}"),
                "globe",
                name,
                &format!("{count} стран"),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    Html(create_page(
        "Где публикуем?",
        "Новое объявление",
        "Выберите континент, страну и город.",
        "/app/me",
        "Профиль",
        &cards,
    ))
    .into_response()
}

pub async fn resource_create_continent(
    State(state): State<AppState>,
    Path(continent_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let next = format!("/app/add/continent/{continent_id}");
    if let Err(response) = authenticated_or_login(&state, &headers, &next) {
        return response;
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let continent = db
        .query_row(
            "SELECT name_ru
             FROM geo_continents
             WHERE id = ?1 AND is_active = 1",
            [continent_id],
            |row| row.get::<_, String>(0),
        )
        .ok();

    let Some(continent) = continent else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let countries = db
        .prepare(
            "SELECT country.id,
                    country.name_ru,
                    COUNT(city.id)
             FROM geo_countries AS country
             LEFT JOIN geo_cities AS city
               ON city.country_id = country.id
              AND city.place_kind = 'city'
              AND city.is_active = 1
             WHERE country.continent_id = ?1
               AND country.is_active = 1
             GROUP BY country.id, country.name_ru
             ORDER BY country.name_ru COLLATE NOCASE",
        )
        .and_then(|mut statement| {
            statement
                .query_map([continent_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let cards = countries
        .iter()
        .map(|(id, name, count)| {
            templates::navigation_card(
                &format!("/app/add/country/{id}"),
                "building",
                name,
                &format!("{count} городов"),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    Html(create_page(
        "Выберите страну",
        &continent,
        "После страны выберите город публикации.",
        "/app/add",
        "Все континенты",
        &cards,
    ))
    .into_response()
}

pub async fn resource_create_country(
    State(state): State<AppState>,
    Path(country_id): Path<i64>,
    Query(query): Query<CreateCityQuery>,
    headers: HeaderMap,
) -> Response {
    let next = format!("/app/add/country/{country_id}");
    if let Err(response) = authenticated_or_login(&state, &headers, &next) {
        return response;
    }

    let search = query.q.unwrap_or_default();
    let search = search.trim();

    if search.chars().count() > 80 || search.chars().any(char::is_control) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let location = db
        .query_row(
            "SELECT country.name_ru,
                    continent.id,
                    continent.name_ru
             FROM geo_countries AS country
             JOIN geo_continents AS continent
               ON continent.id = country.continent_id
             WHERE country.id = ?1
               AND country.is_active = 1
               AND continent.is_active = 1",
            [country_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .ok();

    let Some((country, continent_id, continent)) = location else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let pattern = format!("%{}%", search.to_lowercase());

    let cities = db
        .prepare(
            "SELECT id, name_ru
             FROM geo_cities
             WHERE country_id = ?1
               AND place_kind = 'city'
               AND is_active = 1
               AND (
                    ?2 = ''
                    OR lower(name_ru) LIKE ?3
                    OR lower(name_native) LIKE ?3
                    OR lower(name_ascii) LIKE ?3
               )
             ORDER BY
                CASE WHEN lower(name_ru) = lower(?2) THEN 0 ELSE 1 END,
                population DESC,
                name_ru COLLATE NOCASE
             LIMIT 250",
        )
        .and_then(|mut statement| {
            statement
                .query_map(rusqlite::params![country_id, search, pattern], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let search_form = format!(
        r#"
<form method="get"
      action="/app/add/country/{country_id}"
      class="ui-form">
    <label class="ui-field">
        <span class="ui-field-label">Найти город</span>
        <input class="ui-input"
               type="search"
               name="q"
               maxlength="80"
               value="{value}"
               placeholder="Например: Ницца">
    </label>
    <button class="ui-button" type="submit">Найти</button>
</form>
"#,
        value = templates::escape_html(search),
    );

    let city_cards = cities
        .iter()
        .map(|(id, name)| {
            templates::navigation_card(&format!("/app/add/city/{id}"), "map-pin", name, &country)
        })
        .collect::<Vec<_>>()
        .join("");

    let status = if cities.is_empty() {
        r#"<div class="ui-status">Город не найден.</div>"#
    } else if search.is_empty() {
        r#"<div class="ui-status">Показаны крупнейшие города. Используйте поиск для полного каталога.</div>"#
    } else {
        r#"<div class="ui-status">Выберите найденный город.</div>"#
    };

    let body = format!("{search_form}{status}{city_cards}");

    Html(create_page(
        "Выберите город",
        &format!("{continent} · {country}"),
        "Поиск работает по полному каталогу городов.",
        &format!("/app/add/continent/{continent_id}"),
        "Назад к странам",
        &body,
    ))
    .into_response()
}

pub async fn resource_create_city_page(
    State(state): State<AppState>,
    Path(city_id): Path<i64>,
    Query(query): Query<CreateCityQuery>,
    headers: HeaderMap,
) -> Response {
    let next = format!("/app/add/city/{city_id}");
    if let Err(response) = authenticated_or_login(&state, &headers, &next) {
        return response;
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let location = db
        .query_row(
            "SELECT city.name_ru,
                    country.id,
                    country.name_ru,
                    continent.name_ru
             FROM geo_cities AS city
             JOIN geo_countries AS country
               ON country.id = city.country_id
             JOIN geo_continents AS continent
               ON continent.id = country.continent_id
             WHERE city.id = ?1
               AND city.place_kind = 'city'
               AND city.is_active = 1",
            [city_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .ok();

    let Some((city, country_id, country, continent)) = location else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let kind = kind_from_query(query.kind.as_deref());
    let intent = intent_from_query(query.intent.as_deref());

    if !selection_is_valid(kind, intent) {
        let cards = [
            (
                "work",
                "seeker",
                "Ищу работу",
                "Разместить анкету соискателя",
                "user",
            ),
            (
                "work",
                "offer",
                "Предлагаю работу",
                "Опубликовать вакансию",
                "briefcase",
            ),
            (
                "business",
                "general",
                "Услуга или бизнес",
                "Услуги, жильё, транспорт и организации",
                "building",
            ),
        ]
        .iter()
        .map(|(kind, intent, title, meta, icon)| {
            templates::navigation_card(
                &create_city_href(city_id, kind, intent, None),
                icon,
                title,
                meta,
            )
        })
        .collect::<Vec<_>>()
        .join("");

        return Html(create_page(
            "Что вы хотите создать?",
            &city,
            "Выберите назначение публикации.",
            &format!("/app/add/country/{country_id}"),
            "Выбрать другой город",
            &cards,
        ))
        .into_response();
    }

    let kind = kind.expect("validated kind");
    let intent = intent.expect("validated intent");
    let kind_key = match kind {
        crate::catalog::RubricKind::Work => "work",
        crate::catalog::RubricKind::Business => "business",
    };

    let rubric = query
        .rubric
        .as_deref()
        .and_then(crate::catalog::by_id)
        .filter(|rubric| rubric.kind == kind);

    if rubric.is_none() {
        let cards = crate::catalog::by_kind(kind)
            .map(|rubric| {
                templates::navigation_card(
                    &create_city_href(city_id, kind_key, intent, Some(rubric.id)),
                    if kind == crate::catalog::RubricKind::Work {
                        "briefcase"
                    } else {
                        "building"
                    },
                    rubric.label,
                    "Выбрать рубрику",
                )
            })
            .collect::<Vec<_>>()
            .join("");

        return Html(create_page(
            "Выберите рубрику",
            &format!("{continent} · {country} · {city}"),
            "Рубрика определяет место публикации в каталоге.",
            &format!("/app/add/city/{city_id}"),
            "Изменить назначение",
            &cards,
        ))
        .into_response();
    }

    let rubric = rubric.expect("validated rubric");
    let action = create_city_href(city_id, kind_key, intent, Some(rubric.id));

    let form = format!(
        r#"
<form method="post"
      action="{action}"
      class="ui-form ui-form-stack">
    <input type="hidden"
           name="listing_type"
           value="{intent}">
    <input type="hidden"
           name="rubric"
           value="{rubric_id}">

    <div class="ui-field">
        <span class="ui-field-label">Город</span>
        <div class="card-title">{city}</div>
        <div class="card-meta">{country}</div>
    </div>

    <div class="ui-field">
        <span class="ui-field-label">Рубрика</span>
        <div class="card-title">{rubric_label}</div>
    </div>

    <label class="ui-field">
        <span class="ui-field-label">Название</span>
        <input class="ui-input"
               name="title"
               required
               maxlength="120">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Описание</span>
        <textarea class="ui-textarea"
                  name="description"
                  required
                  maxlength="1000"
                  rows="6"></textarea>
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Телефон или Telegram</span>
        <input class="ui-input"
               name="contact"
               required
               maxlength="120">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Адрес или район</span>
        <input class="ui-input"
               name="address"
               maxlength="250">
    </label>

    <button class="ui-button rm-auth-button"
            type="submit">
        Отправить на проверку
    </button>
</form>
"#,
        action = templates::escape_html(&action),
        intent = templates::escape_html(intent),
        rubric_id = templates::escape_html(rubric.id),
        rubric_label = templates::escape_html(rubric.label),
        city = templates::escape_html(&city),
        country = templates::escape_html(&country),
    );

    Html(create_page(
        "Новое объявление",
        rubric.label,
        "Заполните данные публикации.",
        &create_city_href(city_id, kind_key, intent, None),
        "Изменить рубрику",
        &form,
    ))
    .into_response()
}

pub async fn resource_create_city_submit(
    State(state): State<AppState>,
    Path(city_id): Path<i64>,
    Query(query): Query<CreateCityQuery>,
    headers: HeaderMap,
    Form(form): Form<AddResourceForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Html("<h1>403</h1><p>Запрос отклонён.</p>".to_string()),
        )
            .into_response();
    }

    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return Redirect::temporary(&format!(
                "/login?next={}",
                urlencoding::encode(&format!("/app/add/city/{city_id}"))
            ))
            .into_response();
        }
    };

    let kind = match kind_from_query(query.kind.as_deref()) {
        Some(kind) => kind,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let intent = match intent_from_query(query.intent.as_deref()) {
        Some(intent) => intent,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    if !selection_is_valid(Some(kind), Some(intent))
        || form.listing_type.as_deref().map(str::trim) != Some(intent)
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let rubric = match form
        .rubric
        .as_deref()
        .and_then(crate::catalog::by_id)
        .filter(|rubric| rubric.kind == kind)
    {
        Some(rubric) => rubric,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let title = form.title.trim();
    let description = form.description.trim();
    let contact = form.contact.trim();
    let address = form.address.trim();

    if !input_text_is_valid(title, 1, 120)
        || !input_text_is_valid(description, 1, 1000)
        || !input_text_is_valid(contact, 1, 120)
        || !input_text_is_valid(address, 0, 250)
    {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<h1>Проверьте поля</h1><p>Название, описание и контакт обязательны.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    let screening = screen_listing_content(title, description, contact);
    if !screening.passed {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "<h1>Объявление требует исправления</h1><p>{}</p>",
                templates::escape_html(&screening.reason)
            )),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user.user_id, "resource_add", 10, 3600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Html("<h1>Слишком много публикаций</h1><p>Попробуйте позже.</p>".to_string()),
        )
            .into_response();
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let location = db
        .query_row(
            "SELECT
                COALESCE(legacy_continent_index, -1),
                COALESCE(legacy_country_index, -1),
                COALESCE(legacy_city_index, -1)
             FROM geo_cities
             WHERE id = ?1
               AND place_kind = 'city'
               AND is_active = 1",
            [city_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional();

    let (ci, si, zi) = match location {
        Ok(Some(location)) => location,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let listing_type = crate::catalog::listing_type_for_intent(kind, Some(intent));
    let category = crate::catalog::resource_category_for(rubric);

    let result = db.execute(
        "INSERT INTO resources (
            client_id,
            continent_index,
            country_index,
            city_index,
            city_id,
            category,
            title,
            description,
            contact,
            address,
            rating,
            votes,
            is_premium,
            is_verified,
            is_active,
            moderation_status,
            rejection_reason,
            listing_type,
            rubric,
            created_at,
            updated_at
         )
         VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10,
            0, 0, 0, 0, 1,
            'pending', '', ?11, ?12,
            strftime('%s','now'),
            strftime('%s','now')
         )",
        rusqlite::params![
            user.client_id,
            ci,
            si,
            zi,
            city_id,
            category,
            title,
            description,
            contact,
            address,
            listing_type,
            rubric.id,
        ],
    );

    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            Html(templates::status_page(
                "Объявление на проверке · ResursMap",
                "Проверка",
                "Объявление создано",
                "Оно сохранено в выбранном городе и отправлено на модерацию.",
                &format!(
                    "{}{}",
                    templates::navigation_card(
                        &format!("/app/resource/{id}"),
                        "map-pin",
                        "Открыть объявление",
                        "Предварительный просмотр",
                    ),
                    templates::navigation_card(
                        "/app/my-resources",
                        "menu",
                        "Мои ресурсы",
                        "Управление публикациями",
                    ),
                ),
            ))
            .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Ошибка</h1><p>Не удалось сохранить объявление.</p>".to_string()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_href_preserves_city_and_intent() {
        let href = create_city_href(31_700, "work", "seeker", Some("security"));

        assert_eq!(
            href,
            "/app/add/city/31700?kind=work&intent=seeker&rubric=security"
        );
    }

    #[test]
    fn create_kind_and_intent_pairs_are_strict() {
        use crate::catalog::RubricKind;

        assert!(selection_is_valid(Some(RubricKind::Work), Some("seeker")));
        assert!(selection_is_valid(Some(RubricKind::Work), Some("offer")));
        assert!(selection_is_valid(
            Some(RubricKind::Business),
            Some("general")
        ));

        assert!(!selection_is_valid(
            Some(RubricKind::Business),
            Some("seeker")
        ));
        assert!(!selection_is_valid(Some(RubricKind::Work), Some("general")));
        assert!(!selection_is_valid(None, Some("offer")));
    }

    #[test]
    fn create_intents_are_strict() {
        assert_eq!(intent_from_query(Some("offer")), Some("offer"));
        assert_eq!(intent_from_query(Some("seeker")), Some("seeker"));
        assert_eq!(intent_from_query(Some("general")), Some("general"));
        assert_eq!(intent_from_query(Some("invalid")), None);
    }
}
