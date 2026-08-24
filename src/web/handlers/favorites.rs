use super::*;

pub async fn favorites_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return Html(templates::render_favorites(vec![], false));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let resources: Vec<(i64, String, String, String, String, f64, i64, i64, i64)> = db
        .prepare(
            "SELECT
                r.id,
                r.title,
                r.category,
                r.description,
                r.address,
                r.rating,
                r.votes,
                r.is_verified,
                r.is_premium
             FROM favorites f
             JOIN resources r
               ON r.id = f.resource_id
             WHERE f.user_id = ?1
               AND r.is_active = 1
               AND r.moderation_status = 'approved'
             ORDER BY
                r.is_premium DESC,
                r.is_verified DESC,
                f.created_at DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![user_id], |row| {
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
        .unwrap_or_default();

    drop(db);

    Html(templates::render_favorites(resources, true))
}

pub async fn api_favorite_toggle(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
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

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "favorite_toggle", 60, 60).await
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

    let allowed: bool = db
        .query_row(
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
        .unwrap_or(false);

    if !allowed {
        drop(db);

        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "resource_not_found"
            })),
        )
            .into_response();
    }

    let exists: bool = db
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM favorites
                WHERE user_id = ?1
                  AND resource_id = ?2
            )",
            rusqlite::params![user_id, id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    let favorite = if exists {
        let _ = db.execute(
            "DELETE FROM favorites
             WHERE user_id = ?1
               AND resource_id = ?2",
            rusqlite::params![user_id, id],
        );

        false
    } else {
        let _ = db.execute(
            "INSERT OR IGNORE INTO favorites
             (user_id, resource_id, created_at)
             VALUES (?1, ?2, strftime('%s','now'))",
            rusqlite::params![user_id, id],
        );

        true
    };

    drop(db);

    Json(json!({
        "ok": true,
        "favorite": favorite
    }))
    .into_response()
}

pub async fn api_favorite_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return Json(json!({
                "ok": true,
                "favorite": false,
                "authenticated": false
            }))
            .into_response();
        }
    };

    let favorite: bool = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db
            .query_row(
                "SELECT EXISTS(
                    SELECT 1
                    FROM favorites
                    WHERE user_id = ?1
                      AND resource_id = ?2
                )",
                rusqlite::params![user_id, id],
                |row| row.get(0),
            )
            .unwrap_or(false),
        Err(_) => false,
    };

    Json(json!({
        "ok": true,
        "favorite": favorite,
        "authenticated": true
    }))
    .into_response()
}
