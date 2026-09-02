use super::auth::verify_authenticated_user;
use super::auth::verify_user_session;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub async fn home() -> Redirect {
    Redirect::permanent("/app")
}

pub async fn app_menu() -> Html<String> {
    Html(templates::render_menu())
}

pub async fn app_root(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    // Обновим last_seen_at для авторизованного пользователя
    if let Some(user) = verify_authenticated_user(&state, &headers) {
        if let Ok(db) = crate::db::pool::get_connection(&state.db_pool) {
            let _ = db.execute(
                "UPDATE profiles SET last_seen_at = strftime('%s','now') WHERE user_id = ?1",
                rusqlite::params![user.user_id],
            );
        }
    }
    let users_count = state
        .db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM users WHERE is_active = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        })
        .unwrap_or(0);

    let online_count = state
        .db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM profiles WHERE last_seen_at > strftime('%s','now') - 300",
                [],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        })
        .unwrap_or(0);

    let resources_count = state
        .db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM resources WHERE is_active = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        })
        .unwrap_or(0);

    let guest_mode = verify_user_session(&state, &headers).is_none();

    let continents = state
        .db_pool
        .get()
        .ok()
        .and_then(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT continent.id,continent.name_ru,COUNT(country.id)
                     FROM geo_continents continent
                     LEFT JOIN geo_countries country
                       ON country.continent_id=continent.id AND country.is_active=1
                     WHERE continent.is_active=1
                     GROUP BY continent.id,continent.name_ru
                     ORDER BY continent.name_ru COLLATE NOCASE",
                )
                .ok()?;
            let rows = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .ok()?;
            rows.collect::<Result<Vec<_>, _>>().ok()
        })
        .unwrap_or_default();

    Html(templates::render_geo_root(
        users_count,
        online_count,
        resources_count,
        continents,
        guest_mode,
    ))
}

pub async fn app_geo_continent(
    State(state): State<AppState>,
    Path(continent_id): Path<i64>,
) -> Response {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let name = db
        .query_row(
            "SELECT name_ru FROM geo_continents WHERE id=?1 AND is_active=1",
            [continent_id],
            |row| row.get::<_, String>(0),
        )
        .ok();
    let Some(name) = name else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let countries = db
        .prepare(
            "SELECT country.id,country.name_ru,COUNT(city.id)
             FROM geo_countries country
             LEFT JOIN geo_cities city ON city.country_id=country.id AND city.place_kind='city'
             WHERE country.continent_id=?1 AND country.is_active=1
             GROUP BY country.id,country.name_ru
             ORDER BY country.name_ru COLLATE NOCASE",
        )
        .and_then(|mut stmt| {
            stmt.query_map([continent_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();
    Html(templates::render_geo_continent(
        continent_id,
        &name,
        countries,
    ))
    .into_response()
}

pub async fn app_geo_country(
    State(state): State<AppState>,
    Path(country_id): Path<i64>,
) -> Response {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let location = db
        .query_row(
            "SELECT country.name_ru,continent.id,continent.name_ru
             FROM geo_countries country JOIN geo_continents continent ON continent.id=country.continent_id
             WHERE country.id=?1 AND country.is_active=1 AND continent.is_active=1",
            [country_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
        )
        .ok();
    let Some((country, continent_id, continent)) = location else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cities = load_country_cities(&db, country_id, "", 0, 80);
    let total = db
        .query_row(
            "SELECT COUNT(*) FROM geo_cities WHERE country_id=?1 AND place_kind='city'",
            [country_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    Html(templates::render_geo_country(
        country_id,
        &country,
        continent_id,
        &continent,
        cities,
        total,
    ))
    .into_response()
}

fn load_country_cities(
    db: &rusqlite::Connection,
    country_id: i64,
    query: &str,
    offset: i64,
    limit: i64,
) -> Vec<(i64, String)> {
    let normalized = query.trim().to_lowercase();
    db.prepare(
        "SELECT id,name_ru,name_native,name_ascii FROM geo_cities
         WHERE country_id=?1 AND place_kind='city'
         ORDER BY name_ru COLLATE NOCASE,id",
    )
    .and_then(|mut stmt| {
        let rows = stmt
            .query_map([country_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows
            .into_iter()
            .filter(|(_, name_ru, name_native, name_ascii)| {
                normalized.is_empty()
                    || name_ru.to_lowercase().contains(&normalized)
                    || name_native.to_lowercase().contains(&normalized)
                    || name_ascii.to_lowercase().contains(&normalized)
            })
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(id, name_ru, _, _)| (id, name_ru))
            .collect())
    })
    .unwrap_or_default()
}

#[derive(serde::Deserialize)]
pub struct CityPageQuery {
    offset: Option<i64>,
    q: Option<String>,
}

pub async fn api_map_country_cities(
    State(state): State<AppState>,
    Path(country_id): Path<i64>,
    Query(query): Query<CityPageQuery>,
) -> axum::Json<Value> {
    let offset = query.offset.unwrap_or(0).max(0);
    let search = query.q.unwrap_or_default();
    if search.chars().count() > 80 || search.chars().any(char::is_control) {
        return axum::Json(json!({"ok":false,"error":"invalid_query"}));
    }
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return axum::Json(json!({"ok":false,"error":"database_unavailable"})),
    };
    let exists = db
        .query_row(
            "SELECT 1 FROM geo_countries WHERE id=?1 AND is_active=1",
            [country_id],
            |_| Ok(()),
        )
        .is_ok();
    if !exists {
        return axum::Json(json!({"ok":false,"error":"country_not_found"}));
    }
    let items = load_country_cities(&db, country_id, &search, offset, 80)
        .into_iter()
        .map(|(id, name)| json!({"id":id,"name":name,"href":format!("/app/map/city/{id}")}))
        .collect::<Vec<_>>();
    axum::Json(
        json!({"ok":true,"next_offset":offset + items.len() as i64,"has_more":items.len()==80,"items":items}),
    )
}

pub async fn app_geo_city(State(state): State<AppState>, Path(city_id): Path<i64>) -> Response {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let location = db
        .query_row(
            "SELECT city.name_ru,country.id,country.name_ru,continent.id,continent.name_ru
             FROM geo_cities city
             JOIN geo_countries country ON country.id=city.country_id
             JOIN geo_continents continent ON continent.id=country.continent_id
             WHERE city.id=?1 AND city.place_kind='city'",
            [city_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .ok();
    let Some((city, country_id, country, continent_id, continent)) = location else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let sectors = db
        .prepare(
            "SELECT sector.stable_key,sector.name_ru,COUNT(profession.id)
             FROM profession_sectors sector
             LEFT JOIN professions profession ON profession.sector_key=sector.stable_key AND profession.is_active=1
             GROUP BY sector.stable_key,sector.name_ru,sector.position
             ORDER BY sector.position",
        )
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)))?
                .collect::<Result<Vec<_>,_>>()
        })
        .unwrap_or_default();
    Html(templates::render_geo_city(
        city_id,
        &city,
        country_id,
        &country,
        continent_id,
        &continent,
        sectors,
    ))
    .into_response()
}

pub async fn app_geo_professions(
    State(state): State<AppState>,
    Path((city_id, sector_key)): Path<(i64, String)>,
) -> Response {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let city = db.query_row(
        "SELECT city.name_ru,country.name_ru FROM geo_cities city JOIN geo_countries country ON country.id=city.country_id WHERE city.id=?1 AND city.place_kind='city'",
        [city_id],
        |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)),
    ).ok();
    let sector = db
        .query_row(
            "SELECT name_ru FROM profession_sectors WHERE stable_key=?1",
            [&sector_key],
            |row| row.get::<_, String>(0),
        )
        .ok();
    let (Some((city, country)), Some(sector)) = (city, sector) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let professions = db.prepare(
        "SELECT stable_key,name_ru FROM professions WHERE sector_key=?1 AND is_active=1 ORDER BY name_ru COLLATE NOCASE",
    ).and_then(|mut stmt| {
        stmt.query_map([&sector_key], |row| Ok((row.get(0)?,row.get(1)?)))?.collect::<Result<Vec<_>,_>>()
    }).unwrap_or_default();
    Html(templates::render_geo_professions(
        city_id,
        &city,
        &country,
        &sector,
        professions,
    ))
    .into_response()
}

fn parsed_search_kind(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "work" | "job" | "jobs" => Some("work"),
        "workers" | "people" => Some("workers"),
        "business" => Some("business"),
        _ => None,
    }
}

fn search_kind_clause(kind: Option<&str>) -> &'static str {
    match kind {
        Some("work") => {
            " AND r.category = 'work' AND COALESCE(r.listing_type, 'offer') IN ('offer', 'general')"
        }
        Some("workers") => " AND r.category = 'work' AND r.listing_type = 'seeker'",
        Some("business") => " AND r.category IN ('business', 'services')",
        _ => "",
    }
}

fn search_fts_ready(db: &rusqlite::Connection) -> bool {
    db.prepare("SELECT 1 FROM resources_fts LIMIT 1").is_ok()
}

fn push_location_or(
    sql: &mut String,
    values: &mut Vec<rusqlite::types::Value>,
    location_matches: &[(usize, usize, usize)],
    continent_expr: &str,
    country_expr: &str,
    city_expr: &str,
) {
    if location_matches.is_empty() {
        return;
    }

    sql.push('(');

    for (index, (ci, si, zi)) in location_matches.iter().enumerate() {
        if index > 0 {
            sql.push_str(" OR ");
        }

        let base = values.len() + 1;
        sql.push_str(&format!(
            "({continent_expr} = ?{base} AND {country_expr} = ?{country} AND {city_expr} = ?{city})",
            continent_expr = continent_expr,
            country_expr = country_expr,
            city_expr = city_expr,
            base = base,
            country = base + 1,
            city = base + 2,
        ));
        values.push((*ci as i64).into());
        values.push((*si as i64).into());
        values.push((*zi as i64).into());
    }

    sql.push(')');
}

fn people_fts_ready(db: &rusqlite::Connection) -> bool {
    db.prepare("SELECT 1 FROM profiles_fts LIMIT 1").is_ok()
}

fn discoverable_profile_clause() -> &'static str {
    " AND (
            trim(p.category) <> ''
            OR (
                trim(p.intent_text) <> ''
                AND (
                    p.intent_until = 0
                    OR p.intent_until >= strftime('%s','now')
                )
            )
       )"
}

fn place_prefix_match(place: &str, token: &str) -> bool {
    let place = crate::catalog::normalize(place);
    let token = crate::catalog::normalize(token);
    if place.is_empty() || token.is_empty() {
        return false;
    }
    if place == token || place.starts_with(&token) || token.starts_with(&place) {
        return true;
    }
    if token.chars().count() >= 4 && place.chars().count() >= 4 {
        let token_prefix: String = token.chars().take(4).collect();
        let place_prefix: String = place.chars().take(4).collect();
        return token_prefix == place_prefix;
    }
    false
}

fn location_token_score(token: &str, city: &str, country: &str, continent: &str) -> i32 {
    let token = token.trim();
    if token.chars().count() < 3 || crate::catalog::resolve(token).is_some() {
        return 0;
    }
    if place_prefix_match(city, token) {
        return 500;
    }
    if place_prefix_match(country, token) {
        return 250;
    }
    if token.chars().count() >= 4 && place_prefix_match(continent, token) {
        return 120;
    }
    0
}

fn token_matches_locations(token: &str, locations: &[(usize, usize, usize)]) -> bool {
    let world_data = crate::geography::world();
    locations.iter().any(|(ci, si, zi)| {
        world_data
            .iter()
            .nth(*ci)
            .is_some_and(|(continent, countries)| {
                countries.iter().nth(*si).is_some_and(|(country, cities)| {
                    cities.get(*zi).is_some_and(|city| {
                        location_token_score(token, city, country, continent) > 0
                    })
                })
            })
    })
}

fn search_text_terms(q: &str, locations: &[(usize, usize, usize)]) -> Vec<String> {
    q.split_whitespace()
        .filter_map(|term| {
            let clean: String = term.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.chars().count() < 2 {
                return None;
            }
            if !locations.is_empty() && token_matches_locations(&clean, locations) {
                return None;
            }
            Some(clean)
        })
        .collect()
}

fn location_match_score(
    query: &str,
    query_chars: usize,
    city: &str,
    country: &str,
    continent: &str,
) -> i32 {
    let city_l = city.to_lowercase();
    let country_l = country.to_lowercase();
    let continent_l = continent.to_lowercase();

    if city_l == query {
        return 1000;
    }
    if country_l == query {
        return 900;
    }
    if continent_l == query {
        return 800;
    }
    if city_l.starts_with(query) {
        return 700;
    }
    if country_l.starts_with(query) {
        return 600;
    }
    if query_chars >= 3 && city_l.contains(query) {
        return 400;
    }
    if query_chars >= 3 && country_l.contains(query) {
        return 300;
    }
    if query_chars >= 4 && continent_l.contains(query) {
        return 200;
    }

    0
}

fn collect_location_matches(q: &str) -> Vec<(usize, usize, usize)> {
    let query = q.trim().to_lowercase();

    if query.is_empty() {
        return Vec::new();
    }

    let query_chars = query.chars().count();
    let world_data = crate::geography::world();
    let mut best: BTreeMap<(usize, usize, usize), i32> = BTreeMap::new();

    for (ci, (continent, countries)) in world_data.iter().enumerate() {
        for (si, (country, cities)) in countries.iter().enumerate() {
            for (zi, city) in cities.iter().enumerate() {
                let mut score = location_match_score(&query, query_chars, city, country, continent);
                for token in query.split(|ch: char| !ch.is_alphanumeric()) {
                    score = score.max(location_token_score(token, city, country, continent));
                }
                if score > 0 {
                    best.entry((ci, si, zi))
                        .and_modify(|current| *current = (*current).max(score))
                        .or_insert(score);
                }
            }
        }
    }

    let mut scored: Vec<(i32, usize, usize, usize)> = best
        .into_iter()
        .map(|((ci, si, zi), score)| (score, ci, si, zi))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scored.truncate(20);
    scored
        .into_iter()
        .map(|(_, ci, si, zi)| (ci, si, zi))
        .collect()
}

fn map_search_resource_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<crate::web::view_models::SearchResourceRow> {
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
        row.get(13)?,
    ))
}

fn map_search_person_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<crate::web::view_models::SearchPersonRow> {
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
}

pub async fn app_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<BTreeMap<String, String>>,
) -> Response {
    let q = params.get("q").map(|s| s.trim()).unwrap_or("");
    let kind = parsed_search_kind(params.get("kind").map(|s| s.as_str()).unwrap_or(""));
    let explicit_rubric = params
        .get("rubric")
        .and_then(|value| crate::catalog::by_id(value.trim()));
    let implied_rubric = if explicit_rubric.is_none() {
        crate::catalog::resolve(q)
    } else {
        None
    };
    let rubric_filter = explicit_rubric.or(implied_rubric);
    let city_id = params
        .get("city_id")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0);

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

    let mut resources: Vec<crate::web::view_models::SearchResourceRow> = Vec::new();
    let mut people: Vec<crate::web::view_models::SearchPersonRow> = Vec::new();

    if !q.is_empty() || kind.is_some() || explicit_rubric.is_some() || city_id.is_some() {
        let location_matches = collect_location_matches(q);

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

        let text_terms = search_text_terms(q, &location_matches);
        let fts_terms: Vec<String> = text_terms.iter().map(|term| format!("{}*", term)).collect();

        let fts_source = if text_terms.is_empty() {
            q.to_string()
        } else {
            text_terms.join(" ")
        };
        let fts_query = crate::db::professions::build_fts_query(&db, &fts_source);
        let like_pattern = if let Some(term) = text_terms.first() {
            format!("%{}%", term.to_lowercase())
        } else {
            format!("%{}%", q.to_lowercase())
        };
        let kind_clause = search_kind_clause(kind);
        let fts_ready = search_fts_ready(&db);
        let use_fts = !fts_terms.is_empty() && fts_ready;
        let use_like = q.chars().count() >= 2 && !use_fts;

        let resource_select = "SELECT
                    r.id,
                    r.title,
                    r.category,
                    r.description,
                    r.address,
                    r.rating,
                    r.votes,
                    r.is_verified,
                    r.is_premium,
                    r.continent_index,
                    r.country_index,
                    r.city_index,
                    COALESCE(r.listing_type, 'general'),
                    COALESCE(r.rubric, '')";

        let mut sql = if use_fts {
            format!(
                "{resource_select}
                 FROM resources_fts f
                 JOIN resources r
                   ON r.id = f.rowid
                 WHERE r.is_active = 1
                   AND r.moderation_status = 'approved'"
            )
        } else {
            format!(
                "{resource_select}
                 FROM resources r
                 WHERE r.is_active = 1
                   AND r.moderation_status = 'approved'"
            )
        };

        sql.push_str(kind_clause);

        let mut values: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(rubric) = explicit_rubric {
            values.push(rubric.id.to_string().into());
            sql.push_str(&format!(" AND r.rubric = ?{}", values.len()));
        }

        let mut match_parts: Vec<String> = Vec::new();

        if use_fts {
            values.push(fts_query.clone().into());
            match_parts.push(format!("resources_fts MATCH ?{}", values.len()));
        } else if use_like && !text_terms.is_empty() {
            values.push(like_pattern.clone().into());
            match_parts.push(format!(
                "(LOWER(r.title) LIKE ?{n}
                  OR LOWER(r.description) LIKE ?{n}
                  OR LOWER(r.category) LIKE ?{n}
                  OR LOWER(r.address) LIKE ?{n}
                  OR LOWER(r.rubric) LIKE ?{n})",
                n = values.len()
            ));
        }

        if let Some(rubric) = implied_rubric {
            values.push(rubric.id.to_string().into());
            match_parts.push(format!("r.rubric = ?{}", values.len()));
        }

        if !match_parts.is_empty() {
            sql.push_str(" AND (");
            sql.push_str(&match_parts.join(" OR "));
            sql.push(')');
        }

        if !location_matches.is_empty() {
            let mut location_sql = String::new();
            push_location_or(
                &mut location_sql,
                &mut values,
                &location_matches,
                "r.continent_index",
                "r.country_index",
                "r.city_index",
            );
            sql.push_str(" AND ");
            sql.push_str(&location_sql);
        }

        if let Some(city_id) = city_id {
            values.push(city_id.into());
            sql.push_str(&format!(" AND r.city_id = ?{}", values.len()));
        }

        sql.push_str(
            " ORDER BY
                r.is_premium DESC,
                r.is_verified DESC,
                r.rating DESC,
                r.votes DESC,
                r.id DESC
              LIMIT 100",
        );

        resources = db
            .prepare(&sql)
            .and_then(|mut stmt| {
                stmt.query_map(
                    rusqlite::params_from_iter(values.iter()),
                    map_search_resource_row,
                )?
                .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default();

        let include_people = kind.is_none() || kind == Some("workers");
        let people_fts = people_fts_ready(&db);
        let use_people_fts = !fts_terms.is_empty() && people_fts;
        let use_people_like = q.chars().count() >= 2 && !use_people_fts;

        if include_people
            && (use_people_fts
                || use_people_like
                || kind == Some("workers")
                || !location_matches.is_empty()
                || rubric_filter.is_some()
                || city_id.is_some())
        {
            let mut people_sql = String::from(
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
                 FROM profiles p",
            );

            let mut people_values: Vec<rusqlite::types::Value> = Vec::new();
            let mut people_parts: Vec<String> = Vec::new();

            if use_people_fts {
                people_sql.push_str(
                    "
                 JOIN profiles_fts
                   ON p.rowid = profiles_fts.rowid",
                );
                people_values.push(fts_query.clone().into());
                people_parts.push(format!("profiles_fts MATCH ?{}", people_values.len()));
            } else if use_people_like && !text_terms.is_empty() {
                people_values.push(like_pattern.clone().into());
                people_parts.push(format!(
                    "(LOWER(p.category) LIKE ?{n}
                      OR LOWER(p.intent_text) LIKE ?{n})",
                    n = people_values.len()
                ));
            }

            if let Some(rubric) = implied_rubric {
                people_values.push(rubric.id.to_string().into());
                people_parts.push(format!("p.category = ?{}", people_values.len()));
            }

            people_sql.push_str(
                "
                 JOIN users u
                   ON u.id = p.user_id
                  AND u.is_active = 1
                 WHERE p.public_id <> ''",
            );
            people_sql.push_str(discoverable_profile_clause());

            if let Some(rubric) = explicit_rubric {
                people_values.push(rubric.id.to_string().into());
                people_sql.push_str(&format!(" AND p.category = ?{}", people_values.len()));
            }

            if !people_parts.is_empty() {
                people_sql.push_str(" AND (");
                people_sql.push_str(&people_parts.join(" OR "));
                people_sql.push(')');
            }

            if !location_matches.is_empty() {
                let mut location_sql = String::new();
                push_location_or(
                    &mut location_sql,
                    &mut people_values,
                    &location_matches,
                    "p.home_continent_index",
                    "p.home_country_index",
                    "p.home_city_index",
                );
                people_sql.push_str(" AND ");
                people_sql.push_str(&location_sql);
            }

            if let Some(city_id) = city_id {
                people_values.push(city_id.into());
                people_sql.push_str(&format!(" AND p.home_city_id = ?{}", people_values.len()));
            }

            people_sql.push_str(
                "
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
            );

            people = db
                .prepare(&people_sql)
                .and_then(|mut stmt| {
                    stmt.query_map(
                        rusqlite::params_from_iter(people_values.iter()),
                        map_search_person_row,
                    )?
                    .collect::<Result<Vec<_>, _>>()
                })
                .unwrap_or_default();
        }

        drop(db);
    }

    Html(templates::render_search(
        q,
        kind.unwrap_or(""),
        explicit_rubric.map(|rubric| rubric.id).unwrap_or(""),
        resources,
        people,
        verify_user_session(&state, &headers).is_none(),
        city_id,
    ))
    .into_response()
}

pub async fn app_continent(Path(ci): Path<usize>) -> Html<String> {
    Html(templates::render_continent(ci))
}

pub async fn app_country(Path((ci, si)): Path<(usize, usize)>) -> Html<String> {
    Html(templates::render_country(ci, si))
}

pub async fn app_city(Path((ci, si, zi)): Path<(usize, usize, usize)>) -> Html<String> {
    Html(templates::render_city(ci, si, zi))
}

#[cfg(test)]
mod search_query_tests {
    use super::*;

    #[test]
    fn city_query_still_finds_nice() {
        let matches = collect_location_matches("Ницца");
        assert!(!matches.is_empty());
    }

    #[test]
    fn trade_plus_city_keeps_profession_and_pins_nice() {
        let matches = collect_location_matches("сантехник в Ницце");
        assert!(
            !matches.is_empty(),
            "город из фразы должен находиться по падежу"
        );

        let terms = search_text_terms("сантехник в Ницце", &matches);
        assert_eq!(terms, vec!["сантехник".to_string()]);
        assert!(crate::catalog::resolve("сантехник в Ницце").is_some());
    }
}
