use super::auth::verify_user_session;
use super::common::{input_text_is_valid, rate_limit_retry_after, request_is_cross_site};
use super::user_blocks::users_are_blocked;
use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rusqlite::OptionalExtension;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct StartDirectChatPayload {
    public_id: String,
    message: String,
}

fn public_id_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn first_message_is_valid(value: &str) -> bool {
    input_text_is_valid(value, 1, 2000)
}

fn normalized_pair(sender_user_id: i64, receiver_user_id: i64) -> Option<(i64, i64)> {
    if sender_user_id <= 0 || receiver_user_id <= 0 || sender_user_id == receiver_user_id {
        return None;
    }

    if sender_user_id < receiver_user_id {
        Some((sender_user_id, receiver_user_id))
    } else {
        Some((receiver_user_id, sender_user_id))
    }
}

fn contact_request_is_required(conversation_exists: bool) -> bool {
    !conversation_exists
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

pub async fn api_start_direct_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StartDirectChatPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }

    let sender_user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return json_error(StatusCode::UNAUTHORIZED, "login_required");
        }
    };

    let public_id = payload.public_id.trim();
    let message = payload.message.trim();

    if !public_id_is_valid(public_id) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_public_id");
    }

    if !first_message_is_valid(message) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_message");
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, sender_user_id, "direct_chat_start", 6, 600).await
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

    let sender_is_verified: i64 = connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM auth_identities
                WHERE user_id = ?1 AND verified_at > 0
            )",
            rusqlite::params![sender_user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if sender_is_verified != 1 {
        return json_error(StatusCode::FORBIDDEN, "verification_required");
    }

    let receiver: Option<i64> = connection
        .query_row(
            "SELECT profile.user_id
             FROM profiles AS profile
             JOIN users AS user
               ON user.id = profile.user_id
              AND user.is_active = 1
             WHERE profile.public_id = ?1
             LIMIT 1",
            rusqlite::params![public_id],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None);

    let Some(receiver_user_id) = receiver else {
        return json_error(StatusCode::NOT_FOUND, "user_not_found");
    };

    let Some((user1_id, user2_id)) = normalized_pair(sender_user_id, receiver_user_id) else {
        return json_error(StatusCode::BAD_REQUEST, "cannot_message_self");
    };

    if users_are_blocked(&connection, sender_user_id, receiver_user_id) {
        return json_error(StatusCode::FORBIDDEN, "user_blocked");
    }

    let existing_conversation: Option<i64> = connection
        .query_row(
            "SELECT id
             FROM conversations
             WHERE user1_id = ?1
               AND user2_id = ?2
             LIMIT 1",
            rusqlite::params![user1_id, user2_id],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None);

    let request_pending = contact_request_is_required(existing_conversation.is_some());

    if !request_pending {
        let gate_status: Option<String> = connection
            .query_row(
                "SELECT status
                 FROM contact_requests
                 WHERE (
                     sender_user_id = ?1 AND receiver_user_id = ?2
                 ) OR (
                     sender_user_id = ?2 AND receiver_user_id = ?1
                 )
                 ORDER BY id DESC
                 LIMIT 1",
                rusqlite::params![sender_user_id, receiver_user_id],
                |row| row.get(0),
            )
            .optional()
            .unwrap_or(None);

        match gate_status.as_deref() {
            Some("pending") => {
                return json_error(StatusCode::CONFLICT, "request_pending");
            }
            Some("rejected") => {
                return json_error(StatusCode::FORBIDDEN, "request_rejected");
            }
            _ => {}
        }
    }

    if request_pending {
        let existing_status: Option<String> = connection
            .query_row(
                "SELECT status
                 FROM contact_requests
                 WHERE sender_user_id = ?1
                   AND receiver_user_id = ?2
                 LIMIT 1",
                rusqlite::params![sender_user_id, receiver_user_id],
                |row| row.get(0),
            )
            .optional()
            .unwrap_or(None);

        match existing_status.as_deref() {
            Some("pending") => {
                return json_error(StatusCode::CONFLICT, "request_already_pending");
            }
            Some("accepted") => {
                return json_error(StatusCode::CONFLICT, "already_connected");
            }
            _ => {}
        }

        let now = crate::web::handlers::common::unix_now();

        if connection
            .execute(
                "INSERT INTO contact_requests (
                    sender_user_id,
                    receiver_user_id,
                    message,
                    status,
                    created_at,
                    updated_at
                 )
                 VALUES (?1, ?2, ?3, 'pending', ?4, ?4)
                 ON CONFLICT(sender_user_id, receiver_user_id)
                 DO UPDATE SET
                    message = excluded.message,
                    status = 'pending',
                    updated_at = excluded.updated_at",
                rusqlite::params![sender_user_id, receiver_user_id, message, now],
            )
            .is_err()
        {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "request_store_failed");
        }

        let _ = connection.execute(
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
                ?1, ?2,
                'contact_request',
                'Новый запрос на связь',
                'Участник хочет связаться через ResursMap.',
                0, ?3
             )",
            rusqlite::params![receiver_user_id, sender_user_id, now],
        );
    }

    let now = crate::web::handlers::common::unix_now();

    let transaction =
        match connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
            Ok(transaction) => transaction,
            Err(_) => {
                return json_error(StatusCode::CONFLICT, "chat_busy");
            }
        };

    if existing_conversation.is_none()
        && transaction
            .execute(
                "INSERT OR IGNORE INTO conversations (
                    user1_id,
                    user2_id,
                    created_at,
                    updated_at
                 )
                 VALUES (?1, ?2, ?3, ?3)",
                rusqlite::params![user1_id, user2_id, now],
            )
            .is_err()
    {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "conversation_create_failed",
        );
    }

    let conversation_id: i64 = match transaction.query_row(
        "SELECT id
             FROM conversations
             WHERE user1_id = ?1
               AND user2_id = ?2
             LIMIT 1",
        rusqlite::params![user1_id, user2_id],
        |row| row.get(0),
    ) {
        Ok(value) => value,
        Err(_) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "conversation_lookup_failed",
            );
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
                created_at,
                reply_to_message_id,
                edited_at,
                deleted_at
             )
             VALUES (
                ?1, ?2, ?3,
                0, 0, 0, ?4,
                NULL, 0, 0
             )",
            rusqlite::params![conversation_id, sender_user_id, message, now],
        )
        .unwrap_or(0)
        != 1
    {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_store_failed");
    }

    let message_id = transaction.last_insert_rowid();

    let _ = transaction.execute(
        "UPDATE conversations
         SET updated_at = ?2
         WHERE id = ?1",
        rusqlite::params![conversation_id, now],
    );

    if !request_pending {
        let _ = transaction.execute(
            "UPDATE contact_requests
             SET status = 'accepted',
                 updated_at = ?3
             WHERE (
                 sender_user_id = ?1
                 AND receiver_user_id = ?2
             )
             OR (
                 sender_user_id = ?2
                 AND receiver_user_id = ?1
             )",
            rusqlite::params![sender_user_id, receiver_user_id, now],
        );
    }

    let updated_notification = transaction
        .execute(
            "UPDATE user_notifications
             SET created_at = ?2
             WHERE user_id = ?1
               AND kind = 'chat_message'
               AND is_read = 0",
            rusqlite::params![receiver_user_id, now],
        )
        .unwrap_or(0);

    if updated_notification == 0 {
        let _ = transaction.execute(
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
                ?1, ?2,
                'chat_message',
                'Новое сообщение',
                'Вам написал участник ResursMap.',
                0, ?3
             )",
            rusqlite::params![receiver_user_id, sender_user_id, now],
        );
    }

    if transaction.commit().is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "transaction_failed");
    }

    state.publish_chat_event(
        "message.created",
        conversation_id,
        message_id,
        sender_user_id,
        receiver_user_id,
    );

    (
        StatusCode::CREATED,
        Json(json!({
            "ok": true,
            "conversation_id": conversation_id,
            "message_id": message_id,
            "existing_conversation":
                existing_conversation.is_some(),
            "status": if request_pending {
                "pending"
            } else {
                "accepted"
            },
            "chat_url": format!(
                "/app/chat/{}#chat-end",
                receiver_user_id
            )
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_chat_pair_is_normalized() {
        assert_eq!(normalized_pair(9, 4), Some((4, 9)));
        assert_eq!(normalized_pair(4, 4), None);
        assert_eq!(normalized_pair(0, 4), None);
    }

    #[test]
    fn direct_chat_input_policy_is_strict() {
        assert!(public_id_is_valid("f57ceb83b834b7aa4e0d694c11894014"));
        assert!(!public_id_is_valid("../owner"));
        assert!(first_message_is_valid("Здравствуйте"));
        assert!(!first_message_is_valid(""));
    }

    #[test]
    fn closed_profile_requires_contact_acceptance() {
        assert!(contact_request_is_required(false));
        assert!(!contact_request_is_required(true));
    }
}
