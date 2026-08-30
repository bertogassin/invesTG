use super::auth::verify_user_session;
use super::common::{rate_limit_retry_after, request_is_cross_site};
use crate::state::app_state::AppState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rusqlite::Connection;
use serde_json::json;

pub(super) fn users_are_blocked(
    connection: &Connection,
    first_user_id: i64,
    second_user_id: i64,
) -> bool {
    if first_user_id <= 0 || second_user_id <= 0 || first_user_id == second_user_id {
        return true;
    }

    connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM user_blocks
                WHERE (
                    blocker_user_id = ?1
                    AND blocked_user_id = ?2
                )
                OR (
                    blocker_user_id = ?2
                    AND blocked_user_id = ?1
                )
            )",
            rusqlite::params![first_user_id, second_user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(1)
        == 1
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (
        status,
        Json(json!({
            "ok": false,
            "error": error
        })),
    )
        .into_response()
}

fn authenticated_user(state: &AppState, headers: &HeaderMap) -> Option<i64> {
    verify_user_session(state, headers)
}

pub async fn api_chat_block_status(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let user_id = match authenticated_user(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return json_error(StatusCode::UNAUTHORIZED, "login_required");
        }
    };

    if other_user_id <= 0 || user_id == other_user_id {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    let blocked_by_me: bool = connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM user_blocks
                WHERE blocker_user_id = ?1
                  AND blocked_user_id = ?2
            )",
            rusqlite::params![user_id, other_user_id],
            |row| Ok(row.get::<_, i64>(0)? == 1),
        )
        .unwrap_or(false);

    let blocked_by_peer: bool = connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM user_blocks
                WHERE blocker_user_id = ?1
                  AND blocked_user_id = ?2
            )",
            rusqlite::params![other_user_id, user_id],
            |row| Ok(row.get::<_, i64>(0)? == 1),
        )
        .unwrap_or(false);

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "blocked": blocked_by_me || blocked_by_peer,
            "blocked_by_me": blocked_by_me
        })),
    )
        .into_response()
}

pub async fn api_chat_block(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }

    let user_id = match authenticated_user(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return json_error(StatusCode::UNAUTHORIZED, "login_required");
        }
    };

    if other_user_id <= 0 || user_id == other_user_id {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "user_block", 12, 60).await {
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

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    let target_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM users
             WHERE id = ?1
               AND is_active = 1",
            rusqlite::params![other_user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if target_exists != 1 {
        return json_error(StatusCode::NOT_FOUND, "user_not_found");
    }

    if connection
        .execute(
            "INSERT OR IGNORE INTO user_blocks (
                blocker_user_id,
                blocked_user_id,
                created_at
             )
             VALUES (?1, ?2, strftime('%s','now'))",
            rusqlite::params![user_id, other_user_id],
        )
        .is_err()
    {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "block_failed");
    }

    // Старые сообщения сохраняются, но непрочитанное уведомление
    // от заблокированного пользователя больше не показываем.
    let _ = connection.execute(
        "DELETE FROM user_notifications
         WHERE user_id = ?1
           AND resource_id = ?2
           AND kind = 'chat_message'
           AND is_read = 0",
        rusqlite::params![user_id, other_user_id],
    );

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "blocked": true,
            "blocked_by_me": true
        })),
    )
        .into_response()
}

pub async fn api_chat_unblock(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }

    let user_id = match authenticated_user(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return json_error(StatusCode::UNAUTHORIZED, "login_required");
        }
    };

    if other_user_id <= 0 || user_id == other_user_id {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    if connection
        .execute(
            "DELETE FROM user_blocks
             WHERE blocker_user_id = ?1
               AND blocked_user_id = ?2",
            rusqlite::params![user_id, other_user_id],
        )
        .is_err()
    {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "unblock_failed");
    }

    let still_blocked = users_are_blocked(&connection, user_id, other_user_id);

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "blocked": still_blocked,
            "blocked_by_me": false
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory database");

        connection
            .execute_batch(
                "CREATE TABLE user_blocks (
                    blocker_user_id INTEGER NOT NULL,
                    blocked_user_id INTEGER NOT NULL,
                    created_at INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (blocker_user_id, blocked_user_id)
                );",
            )
            .expect("block schema");

        connection
    }

    #[test]
    fn block_is_bidirectional_for_messaging() {
        let connection = connection();

        connection
            .execute(
                "INSERT INTO user_blocks (
                    blocker_user_id,
                    blocked_user_id
                 )
                 VALUES (10, 20)",
                [],
            )
            .expect("seed block");

        assert!(users_are_blocked(&connection, 10, 20));
        assert!(users_are_blocked(&connection, 20, 10));
        assert!(!users_are_blocked(&connection, 10, 30));
    }

    #[test]
    fn invalid_pair_is_rejected() {
        let connection = connection();

        assert!(users_are_blocked(&connection, 0, 20));
        assert!(users_are_blocked(&connection, 10, 10));
    }
}
