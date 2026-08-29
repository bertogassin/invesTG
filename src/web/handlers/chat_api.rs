use super::auth::verify_user_session;
use super::common::{input_text_is_valid, rate_limit_retry_after, request_is_cross_site};
use crate::state::app_state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use teloxide::prelude::*;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Deserialize)]
pub struct ChatMessagesQuery {
    before_id: Option<i64>,
    after_id: Option<i64>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ChatSendPayload {
    message: String,
}

#[derive(Debug, Serialize)]
struct ChatApiMessage {
    id: i64,
    sender_user_id: i64,
    message: String,
    is_mine: bool,
    delivered_at: i64,
    read_at: i64,
    created_at: i64,
}

fn normalized_pair(current_user_id: i64, other_user_id: i64) -> Option<(i64, i64)> {
    if current_user_id <= 0 || other_user_id <= 0 || current_user_id == other_user_id {
        return None;
    }

    if current_user_id < other_user_id {
        Some((current_user_id, other_user_id))
    } else {
        Some((other_user_id, current_user_id))
    }
}

fn normalized_limit(value: Option<i64>) -> i64 {
    value.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
}

fn message_is_valid(message: &str) -> bool {
    input_text_is_valid(message, 1, 2000)
}

fn conversation_id(
    connection: &rusqlite::Connection,
    current_user_id: i64,
    other_user_id: i64,
) -> Option<i64> {
    let (user1_id, user2_id) = normalized_pair(current_user_id, other_user_id)?;

    connection
        .query_row(
            "SELECT id
             FROM conversations
             WHERE user1_id = ?1
               AND user2_id = ?2
             LIMIT 1",
            rusqlite::params![user1_id, user2_id],
            |row| row.get(0),
        )
        .ok()
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

pub async fn api_chat_messages(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
    Query(query): Query<ChatMessagesQuery>,
) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return json_error(StatusCode::UNAUTHORIZED, "login_required");
        }
    };

    if normalized_pair(user_id, other_user_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }

    if query.before_id.is_some() && query.after_id.is_some() {
        return json_error(StatusCode::BAD_REQUEST, "conflicting_cursor");
    }

    if query.before_id.is_some_and(|value| value <= 0)
        || query.after_id.is_some_and(|value| value < 0)
    {
        return json_error(StatusCode::BAD_REQUEST, "invalid_cursor");
    }

    let limit = normalized_limit(query.limit);

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(conversation_id) => conversation_id,
        None => {
            return json_error(StatusCode::FORBIDDEN, "conversation_not_open");
        }
    };

    let fetch_limit = limit.saturating_add(1);

    let rows_result = if let Some(before_id) = query.before_id {
        connection
            .prepare(
                "SELECT
                    id,
                    sender_user_id,
                    message,
                    delivered_at,
                    read_at,
                    created_at
                 FROM messages
                 WHERE conversation_id = ?1
                   AND id < ?2
                 ORDER BY id DESC
                 LIMIT ?3",
            )
            .and_then(|mut statement| {
                statement
                    .query_map(
                        rusqlite::params![conversation_id, before_id, fetch_limit],
                        |row| {
                            Ok(ChatApiMessage {
                                id: row.get(0)?,
                                sender_user_id: row.get(1)?,
                                message: row.get(2)?,
                                is_mine: row.get::<_, i64>(1)? == user_id,
                                delivered_at: row.get(3)?,
                                read_at: row.get(4)?,
                                created_at: row.get(5)?,
                            })
                        },
                    )?
                    .collect::<Result<Vec<_>, _>>()
            })
    } else if let Some(after_id) = query.after_id {
        connection
            .prepare(
                "SELECT
                    id,
                    sender_user_id,
                    message,
                    delivered_at,
                    read_at,
                    created_at
                 FROM messages
                 WHERE conversation_id = ?1
                   AND id > ?2
                 ORDER BY id ASC
                 LIMIT ?3",
            )
            .and_then(|mut statement| {
                statement
                    .query_map(
                        rusqlite::params![conversation_id, after_id, fetch_limit],
                        |row| {
                            Ok(ChatApiMessage {
                                id: row.get(0)?,
                                sender_user_id: row.get(1)?,
                                message: row.get(2)?,
                                is_mine: row.get::<_, i64>(1)? == user_id,
                                delivered_at: row.get(3)?,
                                read_at: row.get(4)?,
                                created_at: row.get(5)?,
                            })
                        },
                    )?
                    .collect::<Result<Vec<_>, _>>()
            })
    } else {
        connection
            .prepare(
                "SELECT
                    id,
                    sender_user_id,
                    message,
                    delivered_at,
                    read_at,
                    created_at
                 FROM messages
                 WHERE conversation_id = ?1
                 ORDER BY id DESC
                 LIMIT ?2",
            )
            .and_then(|mut statement| {
                statement
                    .query_map(rusqlite::params![conversation_id, fetch_limit], |row| {
                        Ok(ChatApiMessage {
                            id: row.get(0)?,
                            sender_user_id: row.get(1)?,
                            message: row.get(2)?,
                            is_mine: row.get::<_, i64>(1)? == user_id,
                            delivered_at: row.get(3)?,
                            read_at: row.get(4)?,
                            created_at: row.get(5)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()
            })
    };

    let mut messages = match rows_result {
        Ok(messages) => messages,
        Err(_) => {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_query_failed");
        }
    };

    let has_more = messages.len() as i64 > limit;

    if has_more {
        messages.truncate(limit as usize);
    }

    if query.after_id.is_none() {
        messages.reverse();
    }

    let latest_visible_id = messages.iter().map(|message| message.id).max().unwrap_or(0);

    if latest_visible_id > 0 {
        let now = crate::web::handlers::common::unix_now();

        let _ = connection.execute(
            "UPDATE messages
             SET is_read = 1,
                 delivered_at = CASE
                     WHEN delivered_at = 0 THEN ?4
                     ELSE delivered_at
                 END,
                 read_at = CASE
                     WHEN read_at = 0 THEN ?4
                     ELSE read_at
                 END
             WHERE conversation_id = ?1
               AND sender_user_id = ?2
               AND id <= ?3
               AND read_at = 0",
            rusqlite::params![conversation_id, other_user_id, latest_visible_id, now],
        );
    }

    let peer_read_through_id: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(id), 0)
             FROM messages
             WHERE conversation_id = ?1
               AND sender_user_id = ?2
               AND read_at > 0",
            rusqlite::params![conversation_id, user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "messages": messages,
            "has_more": has_more,
            "peer_read_through_id": peer_read_through_id
        })),
    )
        .into_response()
}

pub async fn api_chat_send(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ChatSendPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return json_error(StatusCode::UNAUTHORIZED, "login_required");
        }
    };

    if normalized_pair(user_id, other_user_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }

    let message = payload.message.trim();

    if !message_is_valid(message) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_message");
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "chat_api_send", 30, 60).await
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

    let mut connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(conversation_id) => conversation_id,
        None => {
            return json_error(StatusCode::FORBIDDEN, "conversation_not_open");
        }
    };

    let now = crate::web::handlers::common::unix_now();

    let transaction =
        match connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
            Ok(transaction) => transaction,
            Err(_) => {
                return json_error(StatusCode::CONFLICT, "chat_busy");
            }
        };

    if transaction
        .execute(
            "INSERT INTO messages (
                conversation_id,
                sender_user_id,
                message,
                is_read,
                delivered_at,
                read_at,
                created_at
             )
             VALUES (?1, ?2, ?3, 0, 0, 0, ?4)",
            rusqlite::params![conversation_id, user_id, message, now],
        )
        .unwrap_or(0)
        != 1
    {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_store_failed");
    }

    let message_id = transaction.last_insert_rowid();

    if transaction
        .execute(
            "UPDATE conversations
             SET updated_at = ?2
             WHERE id = ?1",
            rusqlite::params![conversation_id, now],
        )
        .unwrap_or(0)
        != 1
    {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "conversation_update_failed",
        );
    }

    let existing_notification = transaction
        .execute(
            "UPDATE user_notifications
             SET created_at = ?2
             WHERE user_id = ?1
               AND kind = 'chat_message'
               AND is_read = 0",
            rusqlite::params![other_user_id, now],
        )
        .unwrap_or(0);

    let should_notify_telegram = existing_notification == 0;

    if should_notify_telegram
        && transaction
            .execute(
                "INSERT INTO user_notifications (
                    user_id,
                    resource_id,
                    kind,
                    title,
                    message,
                    is_read,
                    created_at
                 )
                 VALUES (
                    ?1, NULL, 'chat_message',
                    'Новое сообщение',
                    'У вас новое сообщение в ResursMap.',
                    0, ?2
                 )",
                rusqlite::params![other_user_id, now],
            )
            .unwrap_or(0)
            != 1
    {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "notification_store_failed",
        );
    }

    if transaction.commit().is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "transaction_failed");
    }

    let telegram_id: Option<i64> = if should_notify_telegram {
        connection
            .query_row(
                "SELECT telegram_id
                 FROM users
                 WHERE id = ?1
                   AND is_active = 1",
                rusqlite::params![other_user_id],
                |row| row.get(0),
            )
            .ok()
    } else {
        None
    };

    drop(connection);

    if let Some(telegram_id) = telegram_id {
        if telegram_id > 0 {
            let bot_token = state.bot_token.clone();

            std::mem::drop(tokio::spawn(async move {
                let bot = Bot::new(bot_token);

                let _ =
                    crate::bot::handler::send_notification(
                        &bot,
                        telegram_id,
                        "📩 У вас новое сообщение в ResursMap!\n\nОткройте чат: https://resursmap.de/app/messages",
                    )
                    .await;
            }));
        }
    }

    (
        StatusCode::CREATED,
        Json(json!({
            "ok": true,
            "message": {
                "id": message_id,
                "sender_user_id": user_id,
                "message": message,
                "is_mine": true,
                "delivered_at": 0,
                "read_at": 0,
                "created_at": now
            }
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_pair_is_normalized() {
        assert_eq!(normalized_pair(7, 3), Some((3, 7)));
        assert_eq!(normalized_pair(3, 7), Some((3, 7)));
        assert_eq!(normalized_pair(3, 3), None);
        assert_eq!(normalized_pair(0, 3), None);
    }

    #[test]
    fn chat_page_size_is_bounded() {
        assert_eq!(normalized_limit(None), 50);
        assert_eq!(normalized_limit(Some(0)), 1);
        assert_eq!(normalized_limit(Some(10)), 10);
        assert_eq!(normalized_limit(Some(999)), 100);
    }

    #[test]
    fn chat_message_policy_is_enforced() {
        assert!(message_is_valid("Привет"));
        assert!(!message_is_valid(""));
        assert!(!message_is_valid("Строка\u{0000}с управлением"));

        let long_message = "я".repeat(2001);

        assert!(!message_is_valid(&long_message));
    }

    #[test]
    fn lifecycle_cursor_rules_are_stable() {
        let query = ChatMessagesQuery {
            before_id: Some(10),
            after_id: None,
            limit: Some(50),
        };

        assert_eq!(query.before_id, Some(10));
        assert_eq!(query.after_id, None);
        assert_eq!(normalized_limit(query.limit), 50);
    }
}
