use super::auth::verify_user_session;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn notifications_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_notifications(vec![], false));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let notifications: Vec<crate::web::view_models::NotificationRow> = db
        .prepare(
            "SELECT
                id,
                resource_id,
                kind,
                title,
                message,
                is_read,
                created_at
             FROM user_notifications
             WHERE user_id = ?1
             ORDER BY
                is_read ASC,
                created_at DESC,
                id DESC
             LIMIT 100",
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
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let _ = db.execute(
        "UPDATE user_notifications
         SET is_read = 1
         WHERE user_id = ?1
           AND is_read = 0",
        rusqlite::params![user_id],
    );

    drop(db);

    Html(templates::render_notifications(notifications, true))
}
