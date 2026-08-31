use super::admin_access::{
    create_admin_session, load_admin_context, record_denied_access, scope_is_authorized,
    verify_admin_session, AdminPermission,
};
use super::auth::verify_authenticated_user;
use crate::state::app_state::AppState;
use crate::web::templates::{admin_ops_page_themed, escape_html};
use axum::{
    extract::{Form, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;

const CITY_RESULT_LIMIT: i64 = 80;
const SEARCH_MAX_CHARS: usize = 80;

#[derive(Debug, Default, Deserialize)]
pub struct GeographyQuery {
    q: Option<String>,
}

#[derive(Debug)]
struct GeographyCityRow {
    id: i64,
    stable_key: String,
    city_name: String,
    native_name: String,
    country_name: String,
    iso2: String,
    continent_name: String,
    population: i64,
    timezone: String,
    target_name: Option<String>,
    telegram_chat_id: Option<i64>,
    telegram_url: Option<String>,
    target_active: Option<i64>,
}

fn normalize_search(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .trim()
        .chars()
        .take(SEARCH_MAX_CHARS)
        .collect()
}

fn telegram_status(row: &GeographyCityRow) -> (&'static str, &'static str) {
    match (
        row.telegram_chat_id.unwrap_or(0),
        row.target_active.unwrap_or(0),
    ) {
        (chat_id, 1) if chat_id < 0 => ("Подключена", "connected"),
        (chat_id, _) if chat_id < 0 => ("Отключена", "disabled"),
        _ => ("Не подключена", "missing"),
    }
}

fn render_city_card(row: &GeographyCityRow) -> String {
    let (status, status_class) = telegram_status(row);

    let native = if row.native_name.trim().is_empty() || row.native_name == row.city_name {
        String::new()
    } else {
        format!(
            r#"<span class="native">{}</span>"#,
            escape_html(&row.native_name),
        )
    };

    let target = row
        .target_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(escape_html)
        .unwrap_or_else(|| "Основная городская группа".to_string());

    let chat_id_value = row
        .telegram_chat_id
        .filter(|value| *value < 0)
        .map(|value| value.to_string())
        .unwrap_or_default();

    let telegram_url_value = row.telegram_url.as_deref().unwrap_or_default();

    let active_checked = if row.target_active == Some(1) {
        " checked"
    } else {
        ""
    };

    let telegram_link = row
        .telegram_url
        .as_deref()
        .filter(|value| value.starts_with("https://t.me/"))
        .map(|value| {
            format!(
                r#"<a class="telegram-link"
                       href="{}"
                       target="_blank"
                       rel="noopener noreferrer">
                        Открыть Telegram
                   </a>"#,
                escape_html(value),
            )
        })
        .unwrap_or_default();

    format!(
        r#"
<article class="city-card">
    <div class="city-main">
        <div class="city-title-row">
            <div>
                <h3>{city_name}</h3>
                {native}
            </div>
            <span class="group-status {status_class}">
                {status}
            </span>
        </div>

        <div class="location">
            {country_name} · {continent_name}
        </div>

        <div class="city-meta">
            <span>{stable_key}</span>
            <span>{iso2}</span>
            <span>{population} жителей</span>
            <span>{timezone}</span>
        </div>
    </div>

    <div class="group-panel">
        <strong>{target}</strong>
        <small>Telegram-группа города</small>
        {telegram_link}

        <details class="group-editor">
            <summary>Настроить группу</summary>

            <form class="group-form"
                  method="post"
                  action="/app/center/geography/group">
                <input type="hidden"
                       name="city_id"
                       value="{city_id}">

                <label>
                    Название группы
                    <input name="target_name"
                           minlength="3"
                           maxlength="80"
                           required
                           value="{target}">
                </label>

                <label>
                    Telegram Chat ID
                    <input name="telegram_chat_id"
                           type="number"
                           max="-1"
                           required
                           placeholder="-1001234567890"
                           value="{chat_id_value}">
                </label>

                <label>
                    Ссылка Telegram
                    <input name="telegram_url"
                           type="url"
                           maxlength="200"
                           placeholder="https://t.me/group"
                           value="{telegram_url_value}">
                </label>

                <label class="active-control">
                    <input name="is_active"
                           type="checkbox"
                           value="1"{active_checked}>
                    Группа активна
                </label>

                <button type="submit">
                    Сохранить группу
                </button>
            </form>
        </details>
    </div>
</article>
"#,
        city_name = escape_html(&row.city_name),
        native = native,
        status_class = status_class,
        status = status,
        country_name = escape_html(&row.country_name),
        continent_name = escape_html(&row.continent_name),
        stable_key = escape_html(&row.stable_key),
        iso2 = escape_html(&row.iso2),
        population = row.population,
        timezone = escape_html(&row.timezone),
        target = target,
        telegram_link = telegram_link,
        city_id = row.id,
        chat_id_value = escape_html(&chat_id_value),
        telegram_url_value = escape_html(telegram_url_value),
        active_checked = active_checked,
    )
}

fn render_page(
    query: &str,
    cities: &[GeographyCityRow],
    continents: i64,
    countries: i64,
    total_cities: i64,
    configured_groups: i64,
    active_groups: i64,
) -> String {
    let cards = if cities.is_empty() {
        r#"
<div class="empty-state">
    <div class="empty-icon">⌕</div>
    <h2>Города не найдены</h2>
    <p>Попробуйте название города, страны или стабильный ключ.</p>
</div>
"#
        .to_string()
    } else {
        cities
            .iter()
            .map(render_city_card)
            .collect::<Vec<_>>()
            .join("")
    };

    let result_caption = if query.is_empty() {
        format!("Крупнейшие города мира · показано {}", cities.len())
    } else {
        format!("Результаты поиска · найдено до {}", cities.len())
    };

    let content = format!(
        r#"
    <div class="topbar">
        <a class="back" href="/app/center">← Центр управления</a>
        <span class="protected">ТЕРРИТОРИИ · ЗАЩИЩЕНО</span>
    </div>

    <section class="hero">
        <div class="kicker">ResursMap · Глобальная география</div>
        <h1>География и группы</h1>
        <p>
            Единый центр территорий ResursMap:
            континенты, страны, города и подключённые
            Telegram-группы.
        </p>
    </section>

    <section class="metrics">
        <div class="metric">
            <strong>{continents}</strong>
            <span>континентов</span>
        </div>
        <div class="metric">
            <strong>{countries}</strong>
            <span>стран и территорий</span>
        </div>
        <div class="metric">
            <strong>{total_cities}</strong>
            <span>городов</span>
        </div>
        <div class="metric">
            <strong>{configured_groups}</strong>
            <span>групп в каталоге</span>
        </div>
        <div class="metric">
            <strong>{active_groups}</strong>
            <span>активных Telegram-групп</span>
        </div>
    </section>

    <form class="search"
          method="get"
          action="/app/center/geography">
        <input type="search"
               name="q"
               value="{query}"
               maxlength="80"
               autocomplete="off"
               placeholder="Ницца, France, FR-NICE…"
               aria-label="Поиск города">
        <button type="submit">Найти</button>
    </form>

    <div class="section-head">
        <h2>Города</h2>
        <span>{result_caption}</span>
    </div>

    <section class="city-list">
        {cards}
    </section>
"#,
        continents = continents,
        countries = countries,
        total_cities = total_cities,
        configured_groups = configured_groups,
        active_groups = active_groups,
        query = escape_html(query),
        result_caption = escape_html(&result_caption),
        cards = cards,
    );

    admin_ops_page_themed(
        "География и группы · ResursMap",
        "rm-admin-ops--geo",
        &content,
    )
}

pub async fn admin_geography_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GeographyQuery>,
) -> Response {
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
                "admin_geography_access_denied",
                "Нет активного административного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if !context.has_permission(AdminPermission::GroupsManage) {
        record_denied_access(
            &state,
            authenticated_user.user_id,
            "admin_geography_permission_denied",
            "Недостаточно прав управления географией",
        );

        return (StatusCode::FORBIDDEN, "Управление географией недоступно").into_response();
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
            HeaderValue::from_static("/app/center/geography"),
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
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "База данных временно недоступна",
            )
                .into_response();
        }
    };

    let search = normalize_search(query.q.as_deref());
    let search_pattern = format!("%{}%", search.to_lowercase());

    let mut statement = match connection.prepare(
        "SELECT
             city.id,
             city.stable_key,
             city.name_ru,
             city.name_native,
             country.name_ru,
             country.iso2,
             continent.name_ru,
             city.population,
             city.timezone,
             target.target_name,
             target.telegram_chat_id,
             target.telegram_url,
             target.is_active
         FROM geo_cities AS city
         JOIN geo_countries AS country
           ON country.id = city.country_id
         JOIN geo_continents AS continent
           ON continent.id = country.continent_id
         JOIN geographic_scopes AS city_scope
           ON city_scope.scope_type = 'city'
          AND city_scope.external_key =
              'city:' || city.stable_key
          AND city_scope.is_active = 1
         LEFT JOIN city_publication_targets AS target
           ON target.id = (
               SELECT candidate.id
               FROM city_publication_targets AS candidate
               WHERE candidate.city_id = city.id
                 AND candidate.target_kind = 'group'
               ORDER BY candidate.is_active DESC, candidate.id ASC
               LIMIT 1
           )
         WHERE city.is_active = 1
           AND (
               ?1 = ''
               OR lower(city.name_ru) LIKE ?2
               OR lower(city.name_native) LIKE ?2
               OR lower(city.name_ascii) LIKE ?2
               OR lower(city.stable_key) LIKE ?2
               OR lower(country.name_ru) LIKE ?2
               OR lower(country.iso2) = lower(?1)
           )
           AND (
               ?3 = 1
               OR city_scope.id IN (
                   WITH RECURSIVE authorized_scopes(id) AS (
                       SELECT ?4

                       UNION

                       SELECT additional.scope_id
                       FROM admin_additional_scopes AS additional
                       WHERE additional.assignment_id = ?5

                       UNION

                       SELECT child.id
                       FROM geographic_scopes AS child
                       JOIN authorized_scopes AS parent
                         ON child.parent_scope_id = parent.id
                       WHERE child.is_active = 1
                   )
                   SELECT id
                   FROM authorized_scopes
               )
           )
         ORDER BY
             CASE WHEN lower(city.stable_key) = lower(?1) THEN 0 ELSE 1 END,
             city.population DESC,
             city.name_ru ASC
         LIMIT ?6",
    ) {
        Ok(statement) => statement,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось подготовить каталог городов",
            )
                .into_response();
        }
    };

    let cities = match statement.query_map(
        params![
            search,
            search_pattern,
            i64::from(context.is_owner()),
            context.scope_id,
            context.assignment_id,
            CITY_RESULT_LIMIT,
        ],
        |row| {
            Ok(GeographyCityRow {
                id: row.get(0)?,
                stable_key: row.get(1)?,
                city_name: row.get(2)?,
                native_name: row.get(3)?,
                country_name: row.get(4)?,
                iso2: row.get(5)?,
                continent_name: row.get(6)?,
                population: row.get(7)?,
                timezone: row.get(8)?,
                target_name: row.get(9)?,
                telegram_chat_id: row.get(10)?,
                telegram_url: row.get(11)?,
                target_active: row.get(12)?,
            })
        },
    ) {
        Ok(rows) => match rows.collect::<rusqlite::Result<Vec<_>>>() {
            Ok(rows) => rows,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Не удалось прочитать каталог городов",
                )
                    .into_response();
            }
        },

        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось выполнить поиск",
            )
                .into_response();
        }
    };

    let scalar =
        |sql: &str| -> i64 { connection.query_row(sql, [], |row| row.get(0)).unwrap_or(0) };

    let html = render_page(
        &search,
        &cities,
        scalar(
            "SELECT COUNT(*)
             FROM geo_continents
             WHERE is_active = 1",
        ),
        scalar(
            "SELECT COUNT(*)
             FROM geo_countries
             WHERE is_active = 1",
        ),
        scalar(
            "SELECT COUNT(*)
             FROM geo_cities
             WHERE is_active = 1",
        ),
        scalar(
            "SELECT COUNT(*)
             FROM city_publication_targets
             WHERE target_kind = 'group'",
        ),
        scalar(
            "SELECT COUNT(*)
             FROM city_publication_targets
             WHERE target_kind = 'group'
               AND telegram_chat_id < 0
               AND is_active = 1",
        ),
    );

    let mut response = Html(html).into_response();

    response.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate, private"
            .parse()
            .expect("valid cache policy"),
    );

    response
}

#[derive(Debug, Deserialize)]
pub struct GeographyGroupForm {
    city_id: i64,
    target_name: String,
    telegram_chat_id: i64,
    telegram_url: String,
    is_active: Option<String>,
}

fn normalize_target_name(value: &str) -> Option<String> {
    let value = value.trim().chars().take(80).collect::<String>();

    (value.chars().count() >= 3).then_some(value)
}

fn normalize_telegram_url(value: &str) -> Option<String> {
    let value = value.trim();

    if value.is_empty() {
        return Some(String::new());
    }

    if value.len() > 200
        || !value.starts_with("https://t.me/")
        || value.contains(char::is_whitespace)
    {
        return None;
    }

    Some(value.to_string())
}

pub async fn admin_geography_group_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<GeographyGroupForm>,
) -> Response {
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
                "admin_group_update_denied",
                "Нет активного назначения",
            );

            return (StatusCode::NOT_FOUND, "404").into_response();
        }
    };

    if !context.has_permission(AdminPermission::GroupsManage) {
        record_denied_access(
            &state,
            user.user_id,
            "admin_group_permission_denied",
            "Нет права GroupsManage",
        );

        return (StatusCode::FORBIDDEN, "Управление группами недоступно").into_response();
    }

    if !verify_admin_session(&state, &headers, context.user_id, context.assignment_id) {
        return (
            StatusCode::UNAUTHORIZED,
            "Административная сессия недействительна",
        )
            .into_response();
    }

    if form.city_id <= 0 || form.telegram_chat_id >= 0 {
        return (
            StatusCode::BAD_REQUEST,
            "Telegram Chat ID должен быть отрицательным",
        )
            .into_response();
    }

    let target_name = match normalize_target_name(&form.target_name) {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Название должно содержать от 3 до 80 символов",
            )
                .into_response();
        }
    };

    let telegram_url = match normalize_telegram_url(&form.telegram_url) {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Допустима только ссылка https://t.me/...",
            )
                .into_response();
        }
    };

    let is_active = i64::from(form.is_active.as_deref() == Some("1"));

    let mut connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "База данных недоступна").into_response();
        }
    };

    let city = connection
        .query_row(
            "SELECT
                 city.stable_key,
                 city.name_ru,
                 country.id,
                 continent.id,
                 city_scope.id
             FROM geo_cities AS city
             JOIN geo_countries AS country
               ON country.id = city.country_id
             JOIN geo_continents AS continent
               ON continent.id = country.continent_id
             JOIN geographic_scopes AS city_scope
               ON city_scope.scope_type = 'city'
              AND city_scope.external_key =
                  'city:' || city.stable_key
              AND city_scope.is_active = 1
             WHERE city.id = ?1
               AND city.is_active = 1",
            params![form.city_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional();

    let (stable_key, city_name, country_id, continent_id, city_scope_id) = match city {
        Ok(Some(city)) => city,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Город не найден").into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось проверить город",
            )
                .into_response();
        }
    };

    if !scope_is_authorized(&state, &context, city_scope_id) {
        record_denied_access(
            &state,
            context.user_id,
            "admin_group_scope_denied",
            &format!("Попытка изменить город вне территории: {}", stable_key),
        );

        return (
            StatusCode::FORBIDDEN,
            "Город находится вне назначенной территории",
        )
            .into_response();
    }

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

    let existing_id = transaction
        .query_row(
            "SELECT id
             FROM city_publication_targets
             WHERE city_id = ?1
               AND target_kind = 'group'
             ORDER BY is_active DESC, id ASC
             LIMIT 1",
            params![form.city_id],
            |row| row.get::<_, i64>(0),
        )
        .optional();

    let existing_id = match existing_id {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось проверить текущую группу",
            )
                .into_response();
        }
    };

    let write = if let Some(target_id) = existing_id {
        transaction.execute(
            "UPDATE city_publication_targets
             SET target_name = ?1,
                 city_name = ?2,
                 telegram_chat_id = ?3,
                 telegram_url = ?4,
                 platform = 'telegram',
                 external_target_id = CAST(?3 AS TEXT),
                 external_url = ?4,
                 capabilities =
                     '[\"publish\",\"moderate\",\"audit\"]',
                 provider_status = CASE
                     WHEN ?5 = 1 THEN 'active'
                     ELSE 'disabled'
                 END,
                 is_active = ?5,
                 updated_at = strftime('%s','now')
             WHERE id = ?6",
            params![
                target_name,
                city_name,
                form.telegram_chat_id,
                telegram_url,
                is_active,
                target_id,
            ],
        )
    } else {
        transaction.execute(
            "INSERT INTO city_publication_targets (
                 continent_index,
                 country_index,
                 city_index,
                 city_id,
                 city_name,
                 target_name,
                 telegram_chat_id,
                 telegram_url,
                 target_kind,
                 platform,
                 external_target_id,
                 external_url,
                 capabilities,
                 provider_status,
                 is_active
             )
             VALUES (
                 ?1, ?2, ?3, ?4, ?5,
                 ?6, ?7, ?8, 'group',
                 'telegram',
                 CAST(?7 AS TEXT),
                 ?8,
                 '[\"publish\",\"moderate\",\"audit\"]',
                 CASE
                     WHEN ?9 = 1 THEN 'active'
                     ELSE 'disabled'
                 END,
                 ?9
             )",
            params![
                1_000_000_i64 + continent_id,
                1_000_000_i64 + country_id,
                1_000_000_i64 + form.city_id,
                form.city_id,
                city_name,
                target_name,
                form.telegram_chat_id,
                telegram_url,
                is_active,
            ],
        )
    };

    if write.is_err() {
        return (
            StatusCode::CONFLICT,
            "Этот Chat ID уже подключён к другому городу",
        )
            .into_response();
    }

    if transaction
        .execute(
            "INSERT INTO geographic_scopes (
                 scope_type,
                 parent_scope_id,
                 external_key,
                 display_name,
                 is_active
             )
             VALUES (
                 'group',
                 ?1,
                 'group:' || ?2 || ':main',
                 ?3 || ' · ' || ?4,
                 ?5
             )
             ON CONFLICT(scope_type, external_key) DO UPDATE SET
                 parent_scope_id = excluded.parent_scope_id,
                 display_name = excluded.display_name,
                 is_active = excluded.is_active,
                 updated_at = strftime('%s','now')",
            params![city_scope_id, stable_key, target_name, city_name, is_active,],
        )
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось обновить область группы",
        )
            .into_response();
    }

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
             'admin_geography_group_updated',
             'info',
             ?3
         )",
        params![
            context.user_id,
            context.assignment_id,
            format!("city={stable_key}; active={is_active}"),
        ],
    );

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось сохранить группу",
        )
            .into_response();
    }

    (
        StatusCode::SEE_OTHER,
        [(
            header::LOCATION,
            format!("/app/center/geography?q={stable_key}&saved=1"),
        )],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geography_search_is_trimmed_and_bounded() {
        assert_eq!(normalize_search(Some("  FR-NICE  ")), "FR-NICE");

        assert_eq!(
            normalize_search(Some(&"x".repeat(100))).chars().count(),
            SEARCH_MAX_CHARS
        );
    }

    #[test]
    fn territorial_protection_is_present() {
        let source = include_str!("admin_geography.rs");

        assert!(source.contains("WITH RECURSIVE authorized_scopes"));
        assert!(source.contains("scope_is_authorized("));
        assert!(source.contains("admin_group_scope_denied"));
    }

    #[test]
    fn telegram_group_input_policy_is_strict() {
        assert_eq!(
            normalize_target_name("  Основная группа  "),
            Some("Основная группа".to_string())
        );

        assert!(normalize_target_name("x").is_none());

        assert_eq!(
            normalize_telegram_url("https://t.me/resursmap"),
            Some("https://t.me/resursmap".to_string())
        );

        assert!(normalize_telegram_url("https://example.com/group").is_none());

        assert!(normalize_telegram_url("https://t.me/bad link").is_none());
    }

    #[test]
    fn telegram_status_requires_negative_chat_and_active_target() {
        let mut row = GeographyCityRow {
            id: 1,
            stable_key: "FR-NICE".to_string(),
            city_name: "Ницца".to_string(),
            native_name: "Nice".to_string(),
            country_name: "Франция".to_string(),
            iso2: "FR".to_string(),
            continent_name: "Европа".to_string(),
            population: 342_669,
            timezone: "Europe/Paris".to_string(),
            target_name: None,
            telegram_chat_id: Some(-1001),
            telegram_url: None,
            target_active: Some(1),
        };

        assert_eq!(telegram_status(&row).0, "Подключена");

        row.target_active = Some(0);

        assert_eq!(telegram_status(&row).0, "Отключена");
    }
}
