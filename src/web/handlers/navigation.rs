use super::auth::verify_authenticated_user;
use super::auth::verify_user_session;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use std::collections::BTreeMap;

fn grouped_category_counts(state: &AppState, sql: &str) -> Vec<(String, i64)> {
    state
        .db_pool
        .get()
        .ok()
        .map(|conn| {
            let mut result = Vec::new();
            let mut stmt = match conn.prepare(sql) {
                Ok(stmt) => stmt,
                Err(_) => return result,
            };
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            });
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    if !row.0.is_empty() {
                        result.push(row);
                    }
                }
            }
            result
        })
        .unwrap_or_default()
}

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

    let categories = grouped_category_counts(
        &state,
        "SELECT trim(category) AS category, COUNT(*) AS cnt
         FROM resources
         WHERE is_active = 1
           AND trim(category) <> ''
         GROUP BY trim(category)
         ORDER BY cnt DESC
         LIMIT 10",
    );

    let people_by_category = grouped_category_counts(
        &state,
        "SELECT trim(p.category) AS category, COUNT(*) AS cnt
         FROM profiles p
         JOIN users u ON u.id = p.user_id
         WHERE u.is_active = 1
           AND p.public_id <> ''
           AND trim(p.category) <> ''
         GROUP BY trim(p.category)
         ORDER BY cnt DESC
         LIMIT 10",
    );

    let guest_mode = verify_user_session(&state, &headers).is_none();

    Html(templates::render_continents(
        users_count,
        online_count,
        resources_count,
        categories,
        people_by_category,
        guest_mode,
    ))
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
    let mut scored: Vec<(i32, usize, usize, usize)> = Vec::new();

    for (ci, (continent, countries)) in world_data.iter().enumerate() {
        for (si, (country, cities)) in countries.iter().enumerate() {
            for (zi, city) in cities.iter().enumerate() {
                let score = location_match_score(&query, query_chars, city, country, continent);

                if score > 0 {
                    scored.push((score, ci, si, zi));
                }
            }
        }
    }

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

    if !q.is_empty() || kind.is_some() {
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

        let fts_terms: Vec<String> = q
            .split_whitespace()
            .filter_map(|term| {
                let clean: String = term.chars().filter(|c| c.is_alphanumeric()).collect();

                if clean.chars().count() < 2 {
                    None
                } else {
                    Some(format!("{}*", clean))
                }
            })
            .collect();

        let fts_query = fts_terms.join(" ");
        let like_pattern = format!("%{}%", q.to_lowercase());
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
                    COALESCE(r.listing_type, 'general')";

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
        let mut match_parts: Vec<String> = Vec::new();

        if use_fts {
            values.push(fts_query.clone().into());
            match_parts.push("resources_fts MATCH ?1".to_string());
        } else if use_like {
            values.push(like_pattern.clone().into());
            match_parts.push(
                "(LOWER(r.title) LIKE ?1
                  OR LOWER(r.description) LIKE ?1
                  OR LOWER(r.category) LIKE ?1
                  OR LOWER(r.address) LIKE ?1)"
                    .to_string(),
            );
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
            match_parts.push(location_sql);
        }

        if !match_parts.is_empty() {
            sql.push_str(" AND (");
            sql.push_str(&match_parts.join(" OR "));
            sql.push(')');
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
                || !location_matches.is_empty())
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
                people_parts.push("profiles_fts MATCH ?1".to_string());
            } else if use_people_like {
                people_values.push(like_pattern.clone().into());
                people_parts.push(
                    "(LOWER(p.category) LIKE ?1
                      OR LOWER(p.intent_text) LIKE ?1)"
                        .to_string(),
                );
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
                people_parts.push(location_sql);
            }

            people_sql.push_str(
                "
                 JOIN users u
                   ON u.id = p.user_id
                  AND u.is_active = 1
                 WHERE p.public_id <> ''",
            );
            people_sql.push_str(discoverable_profile_clause());

            if !people_parts.is_empty() {
                people_sql.push_str(" AND (");
                people_sql.push_str(&people_parts.join(" OR "));
                people_sql.push(')');
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
        resources,
        people,
        verify_user_session(&state, &headers).is_none(),
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
