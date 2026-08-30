use super::admin_access::{
    load_admin_context, record_denied_access, verify_admin_session, AdminPermission,
};
use super::auth::verify_authenticated_user;
use crate::state::app_state::AppState;
use crate::web::templates::escape_html;
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rusqlite::params;
use serde::Deserialize;

const CITY_RESULT_LIMIT: i64 = 80;
const SEARCH_MAX_CHARS: usize = 80;

#[derive(Debug, Default, Deserialize)]
pub struct GeographyQuery {
    q: Option<String>,
}

#[derive(Debug)]
struct GeographyCityRow {
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

    format!(
        r#"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width,initial-scale=1,viewport-fit=cover">
<meta name="robots" content="noindex,nofollow">
<title>География и группы · ResursMap</title>
<style>
:root {{
    color-scheme:dark;
    --bg:#07090d;
    --panel:#11151c;
    --panel-2:#161b24;
    --line:rgba(255,255,255,.10);
    --text:#f7f2e8;
    --muted:#9ea7b5;
    --gold:#d6b77a;
    --green:#46d39a;
    --red:#ff7d7d;
    --blue:#72aaff;
}}
* {{ box-sizing:border-box; }}
body {{
    margin:0;
    min-height:100vh;
    color:var(--text);
    background:
        radial-gradient(circle at 15% 0%,rgba(214,183,122,.13),transparent 32%),
        radial-gradient(circle at 100% 20%,rgba(94,120,255,.10),transparent 28%),
        var(--bg);
    font-family:Inter,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;
}}
.page {{
    width:min(1180px,100%);
    margin:0 auto;
    padding:
        max(18px,env(safe-area-inset-top))
        16px
        calc(80px + env(safe-area-inset-bottom));
}}
.topbar {{
    display:flex;
    align-items:center;
    justify-content:space-between;
    gap:12px;
    margin-bottom:18px;
}}
.back {{
    color:var(--text);
    text-decoration:none;
    font-weight:800;
}}
.protected {{
    padding:8px 11px;
    border:1px solid rgba(214,183,122,.35);
    border-radius:999px;
    color:var(--gold);
    font-size:11px;
    font-weight:900;
    letter-spacing:.08em;
}}
.hero {{
    position:relative;
    overflow:hidden;
    padding:28px;
    border:1px solid rgba(214,183,122,.25);
    border-radius:26px;
    background:
        linear-gradient(135deg,rgba(214,183,122,.14),rgba(17,21,28,.94) 48%),
        var(--panel);
    box-shadow:0 24px 70px rgba(0,0,0,.32);
}}
.hero::after {{
    content:"";
    position:absolute;
    width:230px;
    height:230px;
    right:-80px;
    top:-100px;
    border-radius:50%;
    border:1px solid rgba(214,183,122,.20);
    box-shadow:0 0 80px rgba(214,183,122,.11);
}}
.kicker {{
    color:var(--gold);
    font-size:12px;
    font-weight:900;
    letter-spacing:.12em;
    text-transform:uppercase;
}}
h1 {{
    max-width:760px;
    margin:10px 0;
    font-size:clamp(30px,6vw,58px);
    line-height:.98;
    letter-spacing:-.045em;
}}
.hero p {{
    max-width:720px;
    margin:0;
    color:#c5cad2;
    line-height:1.6;
}}
.metrics {{
    display:grid;
    grid-template-columns:repeat(5,minmax(0,1fr));
    gap:10px;
    margin:16px 0 22px;
}}
.metric {{
    padding:16px;
    border:1px solid var(--line);
    border-radius:18px;
    background:rgba(17,21,28,.84);
}}
.metric strong {{
    display:block;
    font-size:24px;
    color:var(--gold);
}}
.metric span {{
    display:block;
    margin-top:5px;
    color:var(--muted);
    font-size:12px;
}}
.search {{
    position:sticky;
    top:8px;
    z-index:10;
    display:flex;
    gap:10px;
    padding:12px;
    margin:0 0 18px;
    border:1px solid var(--line);
    border-radius:18px;
    background:rgba(7,9,13,.86);
    backdrop-filter:blur(18px);
}}
.search input {{
    flex:1;
    min-width:0;
    min-height:48px;
    padding:0 16px;
    border:1px solid var(--line);
    border-radius:14px;
    color:var(--text);
    background:var(--panel-2);
    font:inherit;
}}
.search button {{
    min-height:48px;
    padding:0 20px;
    border:0;
    border-radius:14px;
    color:#17120a;
    background:linear-gradient(135deg,#ead29f,var(--gold));
    font-weight:900;
}}
.section-head {{
    display:flex;
    align-items:end;
    justify-content:space-between;
    gap:12px;
    margin:22px 2px 12px;
}}
.section-head h2 {{
    margin:0;
    font-size:20px;
}}
.section-head span {{
    color:var(--muted);
    font-size:12px;
}}
.city-list {{
    display:grid;
    gap:12px;
}}
.city-card {{
    display:grid;
    grid-template-columns:minmax(0,1fr) 260px;
    gap:16px;
    padding:18px;
    border:1px solid var(--line);
    border-radius:20px;
    background:linear-gradient(145deg,rgba(22,27,36,.95),rgba(13,17,23,.96));
}}
.city-title-row {{
    display:flex;
    align-items:start;
    justify-content:space-between;
    gap:12px;
}}
.city-card h3 {{
    margin:0;
    font-size:20px;
}}
.native {{
    display:block;
    margin-top:3px;
    color:var(--muted);
    font-size:13px;
}}
.location {{
    margin-top:8px;
    color:#c7ccd5;
}}
.city-meta {{
    display:flex;
    flex-wrap:wrap;
    gap:7px;
    margin-top:12px;
}}
.city-meta span {{
    padding:5px 8px;
    border:1px solid var(--line);
    border-radius:999px;
    color:var(--muted);
    font-size:11px;
}}
.group-status {{
    white-space:nowrap;
    padding:6px 9px;
    border-radius:999px;
    font-size:11px;
    font-weight:900;
}}
.group-status.connected {{
    color:var(--green);
    background:rgba(70,211,154,.10);
    border:1px solid rgba(70,211,154,.30);
}}
.group-status.disabled {{
    color:var(--red);
    background:rgba(255,125,125,.09);
    border:1px solid rgba(255,125,125,.28);
}}
.group-status.missing {{
    color:var(--muted);
    background:rgba(255,255,255,.04);
    border:1px solid var(--line);
}}
.group-panel {{
    display:flex;
    flex-direction:column;
    justify-content:center;
    padding:14px;
    border:1px solid var(--line);
    border-radius:15px;
    background:rgba(0,0,0,.18);
}}
.group-panel small {{
    margin-top:5px;
    color:var(--muted);
}}
.telegram-link {{
    margin-top:12px;
    color:var(--blue);
    text-decoration:none;
    font-size:13px;
    font-weight:800;
}}
.empty-state {{
    padding:42px 20px;
    border:1px dashed var(--line);
    border-radius:22px;
    text-align:center;
    color:var(--muted);
}}
.empty-state h2 {{
    margin:8px 0;
    color:var(--text);
}}
.empty-state p {{ margin:0; }}
.empty-icon {{ font-size:38px; }}
@media (max-width:760px) {{
    .page {{ padding-left:12px;padding-right:12px; }}
    .hero {{ padding:22px 18px; }}
    .metrics {{ grid-template-columns:repeat(2,minmax(0,1fr)); }}
    .metric:last-child {{ grid-column:1/-1; }}
    .search {{ align-items:stretch; }}
    .search button {{ padding:0 14px; }}
    .city-card {{ grid-template-columns:1fr; }}
    .section-head {{ align-items:start;flex-direction:column; }}
}}
</style>
</head>
<body>
<main class="page">
    <div class="topbar">
        <a class="back" href="/app/center">← Центр управления</a>
        <span class="protected">OWNER · PROTECTED</span>
    </div>

    <section class="hero">
        <div class="kicker">ResursMap · Global Geography</div>
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
</main>
</body>
</html>"#,
        continents = continents,
        countries = countries,
        total_cities = total_cities,
        configured_groups = configured_groups,
        active_groups = active_groups,
        query = escape_html(query),
        result_caption = escape_html(&result_caption),
        cards = cards,
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

    if !context.is_owner()
        || !context.has_permission(AdminPermission::CitiesManage)
        || !context.has_permission(AdminPermission::GroupsManage)
    {
        record_denied_access(
            &state,
            authenticated_user.user_id,
            "admin_geography_permission_denied",
            "Недостаточно прав управления географией",
        );

        return (StatusCode::FORBIDDEN, "Управление географией недоступно").into_response();
    }

    if !verify_admin_session(&state, &headers, context.user_id, context.assignment_id) {
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/app/center")]).into_response();
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
         ORDER BY
             CASE WHEN lower(city.stable_key) = lower(?1) THEN 0 ELSE 1 END,
             city.population DESC,
             city.name_ru ASC
         LIMIT ?3",
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

    let cities =
        match statement.query_map(params![search, search_pattern, CITY_RESULT_LIMIT], |row| {
            Ok(GeographyCityRow {
                stable_key: row.get(0)?,
                city_name: row.get(1)?,
                native_name: row.get(2)?,
                country_name: row.get(3)?,
                iso2: row.get(4)?,
                continent_name: row.get(5)?,
                population: row.get(6)?,
                timezone: row.get(7)?,
                target_name: row.get(8)?,
                telegram_chat_id: row.get(9)?,
                telegram_url: row.get(10)?,
                target_active: row.get(11)?,
            })
        }) {
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
    fn telegram_status_requires_negative_chat_and_active_target() {
        let mut row = GeographyCityRow {
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
