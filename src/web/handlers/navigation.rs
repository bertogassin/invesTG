use super::{templates, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::collections::BTreeMap;

pub async fn home() -> Html<String> {
    Html("<h1>ResursMap</h1><p>Сервер работает.</p>".to_string())
}

pub async fn app_root() -> Html<String> {
    Html(templates::render_continents())
}

pub async fn app_search(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Response {
    let q = params.get("q").map(|s| s.trim()).unwrap_or("");

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

    let mut resources: Vec<(
        i64,
        String,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        usize,
        usize,
        usize,
    )> = Vec::new();

    let mut people: Vec<(String, String, String, String, i64, String, i64)> = Vec::new();

    if !q.is_empty() {
        let query_lower = q.to_lowercase();

        let mut location_matches: Vec<(usize, usize, usize)> = Vec::new();

        let world_data = templates::world();

        for (ci, (_, countries)) in world_data.iter().enumerate() {
            for (si, (country, cities)) in countries.iter().enumerate() {
                let country_match = country.to_lowercase().contains(&query_lower);

                for (zi, city) in cities.iter().enumerate() {
                    let city_match = city.to_lowercase().contains(&query_lower);

                    if city_match || country_match {
                        location_matches.push((ci, si, zi));
                    }
                }
            }
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

        // FTS5 query:
        // каждое слово ищем как prefix, чтобы запрос
        // "elect" находил "electrician".
        //
        // Спецсимволы FTS не передаём напрямую:
        // оставляем только буквы/цифры и собираем безопасный MATCH.
        let fts_terms: Vec<String> = q
            .split_whitespace()
            .filter_map(|term| {
                let clean: String = term.chars().filter(|c| c.is_alphanumeric()).collect();

                if clean.is_empty() {
                    None
                } else {
                    Some(format!("{}*", clean))
                }
            })
            .collect();

        let fts_query = fts_terms.join(" ");

        let mut sql = String::from(
            "SELECT
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
                r.city_index
             FROM resources_fts f
             JOIN resources r
               ON r.id = f.rowid
             WHERE r.is_active = 1
               AND r.moderation_status = 'approved'
               AND resources_fts MATCH ?1",
        );

        let mut values: Vec<rusqlite::types::Value> = Vec::new();

        values.push(fts_query.clone().into());

        if !location_matches.is_empty() {
            sql.push_str(" AND (");

            for (index, _) in location_matches.iter().enumerate() {
                if index > 0 {
                    sql.push_str(" OR ");
                }

                let base = 2 + index * 3;

                sql.push_str(&format!(
                    "(
                        r.continent_index = ?{}
                        AND r.country_index = ?{}
                        AND r.city_index = ?{}
                    )",
                    base,
                    base + 1,
                    base + 2
                ));
            }

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

        for (ci, si, zi) in location_matches {
            values.push((ci as i64).into());
            values.push((si as i64).into());
            values.push((zi as i64).into());
        }

        if !fts_terms.is_empty() {
            resources = db
                .prepare(&sql)
                .and_then(|mut stmt| {
                    stmt.query_map(rusqlite::params_from_iter(values.iter()), |row| {
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
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()
                })
                .unwrap_or_default();
        }

        if !fts_terms.is_empty() {
            people = db
                .prepare(
                    "SELECT
                        p.public_id,
                        p.username,
                        p.first_name,
                        p.last_name,
                        p.open_contact,
                        p.intent_text,
                        p.intent_until
                     FROM profiles_fts
                     JOIN profiles p
                       ON p.rowid = profiles_fts.rowid
                     WHERE profiles_fts MATCH ?1
                       AND p.client_id LIKE 'tg:%'
                       AND p.public_id <> ''
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
                        p.updated_at DESC
                     LIMIT 50",
                )
                .and_then(|mut stmt| {
                    stmt.query_map(rusqlite::params![&fts_query], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()
                })
                .unwrap_or_default();
        }

        drop(db);
    }

    Html(templates::render_search(q, resources, people)).into_response()
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
