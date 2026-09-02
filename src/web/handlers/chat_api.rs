use super::auth::verify_user_session;
use super::chat::load_user_conversations;
use super::common::{input_text_is_valid, rate_limit_retry_after, request_is_cross_site};
use super::user_blocks::users_are_blocked;
use crate::state::app_state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Deserialize)]
pub struct ChatMessagesQuery {
    before_id: Option<i64>,
    after_id: Option<i64>,
    limit: Option<i64>,
    mark_read: Option<bool>,
    read_through_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ChatSendPayload {
    message: String,
    reply_to_message_id: Option<i64>,
    client_message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatEditPayload {
    message: String,
}

fn message_can_be_edited(created_at: i64, deleted_at: i64, now: i64) -> bool {
    deleted_at == 0
        && created_at > 0
        && now >= created_at
        && now.saturating_sub(created_at) <= 86_400
}

pub(crate) fn chat_media_attachment_url(
    id: i64,
    deleted_at: i64,
    kind: &str,
    path: &str,
) -> String {
    if deleted_at == 0 && !path.is_empty() && (kind == "image" || kind == "voice") {
        format!("/api/chat/media/{id}")
    } else {
        String::new()
    }
}

#[derive(Debug, Clone, Serialize)]
struct MessageReaction {
    emoji: String,
    count: i64,
    mine: bool,
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
    reply_to_message_id: Option<i64>,
    reply_sender_user_id: Option<i64>,
    reply_message: String,
    edited_at: i64,
    deleted_at: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    client_message_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    attachment_kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    attachment_mime: String,
    attachment_size: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    attachment_url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reactions: Vec<MessageReaction>,
}

const CHAT_REACTION_EMOJIS: &[&str] = &["❤️", "👍", "😂", "😮", "😢", "🙏"];

fn reaction_emoji_is_allowed(value: &str) -> bool {
    CHAT_REACTION_EMOJIS.contains(&value)
}

fn attach_reactions(
    connection: &rusqlite::Connection,
    messages: &mut [ChatApiMessage],
    viewer_user_id: i64,
) {
    if messages.is_empty() {
        return;
    }

    let message_ids: Vec<i64> = messages.iter().map(|message| message.id).collect();
    let reactions_by_message = load_message_reactions(connection, &message_ids, viewer_user_id);

    for message in messages.iter_mut() {
        if let Some(reactions) = reactions_by_message.get(&message.id) {
            message.reactions = reactions.clone();
        }
    }
}

fn load_message_reactions(
    connection: &rusqlite::Connection,
    message_ids: &[i64],
    viewer_user_id: i64,
) -> std::collections::HashMap<i64, Vec<MessageReaction>> {
    let mut reactions_by_message = std::collections::HashMap::new();

    if message_ids.is_empty() {
        return reactions_by_message;
    }

    let mut placeholders = Vec::with_capacity(message_ids.len());
    let mut params: Vec<rusqlite::types::Value> = Vec::with_capacity(message_ids.len() + 1);
    params.push(rusqlite::types::Value::from(viewer_user_id));

    for (index, message_id) in message_ids.iter().enumerate() {
        placeholders.push(format!("?{}", index + 2));
        params.push(rusqlite::types::Value::from(*message_id));
    }

    let sql = format!(
        "SELECT
            message_id,
            emoji,
            COUNT(*) AS reaction_count,
            SUM(CASE WHEN user_id = ?1 THEN 1 ELSE 0 END) AS mine_count
         FROM message_reactions
         WHERE message_id IN ({})
         GROUP BY message_id, emoji
         ORDER BY message_id ASC, reaction_count DESC, emoji ASC",
        placeholders.join(", ")
    );

    let mut statement = match connection.prepare(&sql) {
        Ok(statement) => statement,
        Err(_) => return reactions_by_message,
    };

    let rows = match statement.query_map(rusqlite::params_from_iter(params), |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    }) {
        Ok(rows) => rows,
        Err(_) => return reactions_by_message,
    };

    for row in rows.flatten() {
        let (message_id, emoji, count, mine_count) = row;
        reactions_by_message
            .entry(message_id)
            .or_default()
            .push(MessageReaction {
                emoji,
                count,
                mine: mine_count > 0,
            });
    }

    reactions_by_message
}

pub fn reactions_for_view(
    connection: &rusqlite::Connection,
    message_ids: &[i64],
    viewer_user_id: i64,
) -> std::collections::HashMap<i64, Vec<crate::web::view_models::ChatReactionRow>> {
    load_message_reactions(connection, message_ids, viewer_user_id)
        .into_iter()
        .map(|(message_id, reactions)| {
            (
                message_id,
                reactions
                    .into_iter()
                    .map(|reaction| crate::web::view_models::ChatReactionRow {
                        emoji: reaction.emoji,
                        count: reaction.count,
                        mine: reaction.mine,
                    })
                    .collect(),
            )
        })
        .collect()
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

fn client_message_id_is_valid(value: &str) -> bool {
    (16..=80).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
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

fn ensure_conversation_for_outgoing(
    connection: &rusqlite::Connection,
    current_user_id: i64,
    other_user_id: i64,
) -> Result<i64, &'static str> {
    if let Some(existing_id) = conversation_id(connection, current_user_id, other_user_id) {
        return Ok(existing_id);
    }

    let Some((user1_id, user2_id)) = normalized_pair(current_user_id, other_user_id) else {
        return Err("conversation_not_open");
    };

    let other_is_active: i64 = connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE id = ?1
                  AND is_active = 1
            )",
            rusqlite::params![other_user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if other_is_active != 1 {
        return Err("user_not_found");
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
             VALUES (?1, ?2, '', 'pending', ?3, ?3)
             ON CONFLICT(sender_user_id, receiver_user_id)
             DO NOTHING",
            rusqlite::params![current_user_id, other_user_id, now],
        )
        .is_err()
    {
        return Err("request_store_failed");
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
        rusqlite::params![other_user_id, current_user_id, now],
    );

    if connection
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
        return Err("conversation_create_failed");
    }

    conversation_id(connection, current_user_id, other_user_id).ok_or("conversation_lookup_failed")
}

pub(super) fn chat_contact_gate_error(
    connection: &rusqlite::Connection,
    current_user_id: i64,
    other_user_id: i64,
) -> Option<&'static str> {
    let status: Option<String> = connection
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
            rusqlite::params![current_user_id, other_user_id],
            |row| row.get(0),
        )
        .ok();

    match status.as_deref() {
        Some("pending") => Some("request_pending"),
        Some("rejected") => Some("request_rejected"),
        _ => None,
    }
}

pub(super) fn user_has_verified_identity(connection: &rusqlite::Connection, user_id: i64) -> bool {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM auth_identities
                WHERE user_id = ?1 AND verified_at > 0
            )",
            rusqlite::params![user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        == 1
}

fn touch_profile_last_seen(connection: &rusqlite::Connection, user_id: i64) {
    let now = crate::web::handlers::common::unix_now();

    let _ = connection.execute(
        "UPDATE profiles
         SET last_seen_at = ?1
         WHERE user_id = ?2",
        rusqlite::params![now, user_id],
    );
}

fn load_api_message_by_id(
    connection: &rusqlite::Connection,
    conversation_id: i64,
    message_id: i64,
    user_id: i64,
) -> Option<ChatApiMessage> {
    let mut message = connection
        .query_row(
            "SELECT
                messages.id,
                messages.sender_user_id,
                messages.message,
                messages.delivered_at,
                messages.read_at,
                messages.created_at,
                messages.reply_to_message_id,
                (
                    SELECT reply.sender_user_id
                    FROM messages AS reply
                    WHERE reply.id =
                        messages.reply_to_message_id
                      AND reply.conversation_id =
                        messages.conversation_id
                ),
                COALESCE((
                    SELECT CASE
                        WHEN reply.deleted_at > 0
                        THEN 'Сообщение удалено'
                        ELSE reply.message
                    END
                    FROM messages AS reply
                    WHERE reply.id =
                        messages.reply_to_message_id
                      AND reply.conversation_id =
                        messages.conversation_id
                ), ''),
                messages.edited_at,
                messages.deleted_at,
                COALESCE(messages.client_message_id, ''),
                COALESCE(messages.attachment_kind, ''),
                COALESCE(messages.attachment_mime, ''),
                COALESCE(messages.attachment_size, 0),
                COALESCE(messages.attachment_path, '')
             FROM messages
             WHERE messages.id = ?1
               AND messages.conversation_id = ?2
             LIMIT 1",
            rusqlite::params![message_id, conversation_id],
            |row| {
                Ok(ChatApiMessage {
                    id: row.get(0)?,
                    sender_user_id: row.get(1)?,
                    message: row.get(2)?,
                    is_mine: row.get::<_, i64>(1)? == user_id,
                    delivered_at: row.get(3)?,
                    read_at: row.get(4)?,
                    created_at: row.get(5)?,
                    reply_to_message_id: row.get(6)?,
                    reply_sender_user_id: row.get(7)?,
                    reply_message: row.get(8)?,
                    edited_at: row.get(9)?,
                    deleted_at: row.get(10)?,
                    client_message_id: row.get(11)?,
                    attachment_kind: {
                        let kind: String = row.get(12)?;
                        kind
                    },
                    attachment_mime: row.get(13)?,
                    attachment_size: row.get(14)?,
                    attachment_url: {
                        let deleted_at: i64 = row.get(10)?;
                        let kind: String = row.get(12)?;
                        let path: String = row.get(15)?;
                        let id: i64 = row.get(0)?;
                        chat_media_attachment_url(id, deleted_at, &kind, &path)
                    },
                    reactions: Vec::new(),
                })
            },
        )
        .ok()?;

    attach_reactions(connection, std::slice::from_mut(&mut message), user_id);

    Some(message)
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

    touch_profile_last_seen(&connection, user_id);

    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(conversation_id) => conversation_id,
        None => {
            return (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "messages": [],
                    "has_more": false,
                    "peer_read_through_id": 0
                })),
            )
                .into_response();
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
                    messages.delivered_at,
                    messages.read_at,
                    messages.created_at,
                    messages.reply_to_message_id,
                    (
                        SELECT reply.sender_user_id
                        FROM messages AS reply
                        WHERE reply.id =
                            messages.reply_to_message_id
                          AND reply.conversation_id =
                            messages.conversation_id
                    ),
                    COALESCE((
                        SELECT CASE
                            WHEN reply.deleted_at > 0
                            THEN 'Сообщение удалено'
                            ELSE reply.message
                        END
                        FROM messages AS reply
                        WHERE reply.id =
                            messages.reply_to_message_id
                          AND reply.conversation_id =
                            messages.conversation_id
                    ), ''),
                    messages.edited_at,
                    messages.deleted_at,
                    COALESCE(messages.client_message_id, ''),
                COALESCE(messages.attachment_kind, ''),
                COALESCE(messages.attachment_mime, ''),
                COALESCE(messages.attachment_size, 0),
                COALESCE(messages.attachment_path, '')
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
                                reply_to_message_id: row.get(6)?,
                                reply_sender_user_id: row.get(7)?,
                                reply_message: row.get(8)?,
                                edited_at: row.get(9)?,
                                deleted_at: row.get(10)?,
                                client_message_id: row.get(11)?,
                                attachment_kind: {
                                    let kind: String = row.get(12)?;
                                    kind
                                },
                                attachment_mime: row.get(13)?,
                                attachment_size: row.get(14)?,
                                attachment_url: {
                                    let deleted_at: i64 = row.get(10)?;
                                    let kind: String = row.get(12)?;
                                    let path: String = row.get(15)?;
                                    let id: i64 = row.get(0)?;
                                    chat_media_attachment_url(id, deleted_at, &kind, &path)
                                },
                                reactions: Vec::new(),
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
                    messages.delivered_at,
                    messages.read_at,
                    messages.created_at,
                    messages.reply_to_message_id,
                    (
                        SELECT reply.sender_user_id
                        FROM messages AS reply
                        WHERE reply.id =
                            messages.reply_to_message_id
                          AND reply.conversation_id =
                            messages.conversation_id
                    ),
                    COALESCE((
                        SELECT CASE
                            WHEN reply.deleted_at > 0
                            THEN 'Сообщение удалено'
                            ELSE reply.message
                        END
                        FROM messages AS reply
                        WHERE reply.id =
                            messages.reply_to_message_id
                          AND reply.conversation_id =
                            messages.conversation_id
                    ), ''),
                    messages.edited_at,
                    messages.deleted_at,
                    COALESCE(messages.client_message_id, ''),
                COALESCE(messages.attachment_kind, ''),
                COALESCE(messages.attachment_mime, ''),
                COALESCE(messages.attachment_size, 0),
                COALESCE(messages.attachment_path, '')
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
                                reply_to_message_id: row.get(6)?,
                                reply_sender_user_id: row.get(7)?,
                                reply_message: row.get(8)?,
                                edited_at: row.get(9)?,
                                deleted_at: row.get(10)?,
                                client_message_id: row.get(11)?,
                                attachment_kind: {
                                    let kind: String = row.get(12)?;
                                    kind
                                },
                                attachment_mime: row.get(13)?,
                                attachment_size: row.get(14)?,
                                attachment_url: {
                                    let deleted_at: i64 = row.get(10)?;
                                    let kind: String = row.get(12)?;
                                    let path: String = row.get(15)?;
                                    let id: i64 = row.get(0)?;
                                    chat_media_attachment_url(id, deleted_at, &kind, &path)
                                },
                                reactions: Vec::new(),
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
                    messages.delivered_at,
                    messages.read_at,
                    messages.created_at,
                    messages.reply_to_message_id,
                    (
                        SELECT reply.sender_user_id
                        FROM messages AS reply
                        WHERE reply.id =
                            messages.reply_to_message_id
                          AND reply.conversation_id =
                            messages.conversation_id
                    ),
                    COALESCE((
                        SELECT CASE
                            WHEN reply.deleted_at > 0
                            THEN 'Сообщение удалено'
                            ELSE reply.message
                        END
                        FROM messages AS reply
                        WHERE reply.id =
                            messages.reply_to_message_id
                          AND reply.conversation_id =
                            messages.conversation_id
                    ), ''),
                    messages.edited_at,
                    messages.deleted_at,
                    COALESCE(messages.client_message_id, ''),
                COALESCE(messages.attachment_kind, ''),
                COALESCE(messages.attachment_mime, ''),
                COALESCE(messages.attachment_size, 0),
                COALESCE(messages.attachment_path, '')
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
                            reply_to_message_id: row.get(6)?,
                            reply_sender_user_id: row.get(7)?,
                            reply_message: row.get(8)?,
                            edited_at: row.get(9)?,
                            deleted_at: row.get(10)?,
                            client_message_id: row.get(11)?,
                            attachment_kind: {
                                let kind: String = row.get(12)?;
                                kind
                            },
                            attachment_mime: row.get(13)?,
                            attachment_size: row.get(14)?,
                            attachment_url: {
                                let deleted_at: i64 = row.get(10)?;
                                let kind: String = row.get(12)?;
                                let path: String = row.get(15)?;
                                let id: i64 = row.get(0)?;
                                chat_media_attachment_url(id, deleted_at, &kind, &path)
                            },
                            reactions: Vec::new(),
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

    attach_reactions(&connection, &mut messages, user_id);

    let latest_visible_id = messages.iter().map(|message| message.id).max().unwrap_or(0);

    if latest_visible_id > 0 {
        let now = crate::web::handlers::common::unix_now();

        let _ = connection.execute(
            "UPDATE messages
             SET delivered_at = ?4
             WHERE conversation_id = ?1
               AND sender_user_id = ?2
               AND id <= ?3
               AND delivered_at = 0",
            rusqlite::params![conversation_id, other_user_id, latest_visible_id, now],
        );
    }

    let should_mark_read = query.mark_read.unwrap_or(false);

    if should_mark_read {
        let read_through_id = query
            .read_through_id
            .filter(|value| *value > 0)
            .unwrap_or(latest_visible_id);

        if read_through_id > 0 {
            let now = crate::web::handlers::common::unix_now();

            let read_changed = connection
                .execute(
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
                    rusqlite::params![conversation_id, other_user_id, read_through_id, now],
                )
                .unwrap_or(0);

            if read_changed > 0 {
                state.publish_chat_event(
                    "message.read",
                    conversation_id,
                    read_through_id,
                    user_id,
                    other_user_id,
                );
            }
        }
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

    let client_message_id = payload.client_message_id.as_deref().unwrap_or("").trim();

    if !client_message_id.is_empty() && !client_message_id_is_valid(client_message_id) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_client_message_id");
    }

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

    if users_are_blocked(&connection, user_id, other_user_id) {
        return json_error(StatusCode::FORBIDDEN, "user_blocked");
    }

    if !user_has_verified_identity(&connection, user_id) {
        return json_error(StatusCode::FORBIDDEN, "verification_required");
    }

    if let Some(error) = chat_contact_gate_error(&connection, user_id, other_user_id) {
        return json_error(StatusCode::FORBIDDEN, error);
    }

    let conversation_id =
        match ensure_conversation_for_outgoing(&connection, user_id, other_user_id) {
            Ok(conversation_id) => conversation_id,
            Err(error) => {
                return json_error(StatusCode::FORBIDDEN, error);
            }
        };

    if !client_message_id.is_empty() {
        let existing_id: Option<i64> = connection
            .query_row(
                "SELECT id
                 FROM messages
                 WHERE sender_user_id = ?1
                   AND client_message_id = ?2
                 LIMIT 1",
                rusqlite::params![user_id, client_message_id],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_id) = existing_id {
            if let Some(message) =
                load_api_message_by_id(&connection, conversation_id, existing_id, user_id)
            {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "ok": true,
                        "duplicate": true,
                        "message": message
                    })),
                )
                    .into_response();
            }
        }
    }

    let reply_to_message_id = match payload.reply_to_message_id {
        Some(reply_id) if reply_id > 0 => {
            let exists: i64 = connection
                .query_row(
                    "SELECT COUNT(*)
                         FROM messages
                         WHERE id = ?1
                           AND conversation_id = ?2
                           AND deleted_at = 0",
                    rusqlite::params![reply_id, conversation_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if exists != 1 {
                return json_error(StatusCode::BAD_REQUEST, "invalid_reply");
            }

            Some(reply_id)
        }
        Some(_) => {
            return json_error(StatusCode::BAD_REQUEST, "invalid_reply");
        }
        None => None,
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
                created_at,
                reply_to_message_id,
                client_message_id
             )
             VALUES (
                ?1, ?2, ?3, 0, 0, 0, ?4, ?5, ?6
             )",
            rusqlite::params![
                conversation_id,
                user_id,
                message,
                now,
                reply_to_message_id,
                client_message_id
            ],
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
               AND is_read = 0
               AND (resource_id = ?3 OR resource_id IS NULL)",
            rusqlite::params![other_user_id, now, user_id],
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
                    ?1, ?2, 'chat_message',
                    'Новое сообщение',
                    'У вас новое сообщение в ResursMap.',
                    0, ?3
                 )",
                rusqlite::params![other_user_id, user_id, now],
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

    state.publish_chat_event(
        "message.created",
        conversation_id,
        message_id,
        user_id,
        other_user_id,
    );

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
            crate::telegram_notify::notify_telegram_user(
                state.bot_token.as_deref(),
                telegram_id,
                &format!(
                    "📩 У вас новое сообщение в ResursMap!\n\nОткройте чат: https://resursmap.de/app/chat/{user_id}"
                ),
            );
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
                "created_at": now,
                "reply_to_message_id": reply_to_message_id,
                "reply_sender_user_id": null,
                "reply_message": "",
                "edited_at": 0,
                "deleted_at": 0
            }
        })),
    )
        .into_response()
}

pub async fn api_chat_edit(
    State(state): State<AppState>,
    Path((other_user_id, message_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    Json(payload): Json<ChatEditPayload>,
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

    if normalized_pair(user_id, other_user_id).is_none() || message_id <= 0 {
        return json_error(StatusCode::BAD_REQUEST, "invalid_request");
    }

    let message = payload.message.trim();

    if !message_is_valid(message) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_message");
    }

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "chat_edit", 20, 60).await {
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

    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(value) => value,
        None => {
            return json_error(StatusCode::FORBIDDEN, "conversation_not_open");
        }
    };

    let row: Option<(i64, i64)> = connection
        .query_row(
            "SELECT created_at, deleted_at
             FROM messages
             WHERE id = ?1
               AND conversation_id = ?2
               AND sender_user_id = ?3",
            rusqlite::params![message_id, conversation_id, user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let Some((created_at, deleted_at)) = row else {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    };

    let now = crate::web::handlers::common::unix_now();

    if !message_can_be_edited(created_at, deleted_at, now) {
        return json_error(StatusCode::CONFLICT, "message_not_editable");
    }

    let changed = connection
        .execute(
            "UPDATE messages
             SET message = ?1,
                 edited_at = ?2
             WHERE id = ?3
               AND conversation_id = ?4
               AND sender_user_id = ?5
               AND deleted_at = 0",
            rusqlite::params![message, now, message_id, conversation_id, user_id],
        )
        .unwrap_or(0);

    if changed != 1 {
        return json_error(StatusCode::CONFLICT, "message_changed");
    }

    state.publish_chat_event(
        "message.edited",
        conversation_id,
        message_id,
        user_id,
        other_user_id,
    );

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message_id": message_id,
            "message": message,
            "edited_at": now
        })),
    )
        .into_response()
}

pub async fn api_chat_delete(
    State(state): State<AppState>,
    Path((other_user_id, message_id)): Path<(i64, i64)>,
    headers: HeaderMap,
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

    if normalized_pair(user_id, other_user_id).is_none() || message_id <= 0 {
        return json_error(StatusCode::BAD_REQUEST, "invalid_request");
    }

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "chat_delete", 20, 60).await
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

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(value) => value,
        None => {
            return json_error(StatusCode::FORBIDDEN, "conversation_not_open");
        }
    };

    let now = crate::web::handlers::common::unix_now();

    let changed = connection
        .execute(
            "UPDATE messages
             SET message = '',
                 deleted_at = ?1,
                 edited_at = 0
             WHERE id = ?2
               AND conversation_id = ?3
               AND sender_user_id = ?4
               AND deleted_at = 0",
            rusqlite::params![now, message_id, conversation_id, user_id],
        )
        .unwrap_or(0);

    if changed != 1 {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    }

    state.publish_chat_event(
        "message.deleted",
        conversation_id,
        message_id,
        user_id,
        other_user_id,
    );

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message_id": message_id,
            "deleted_at": now
        })),
    )
        .into_response()
}

pub async fn api_chat_peer(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
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

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    touch_profile_last_seen(&connection, user_id);

    let peer_row: Option<(i64, i64)> = connection
        .query_row(
            "SELECT
                COALESCE(last_seen_at, 0),
                COALESCE(open_contact, 0)
             FROM profiles
             WHERE user_id = ?1
             LIMIT 1",
            rusqlite::params![other_user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let (last_seen_at, open_contact) = peer_row.unwrap_or((0, 0));
    let now = crate::web::handlers::common::unix_now();
    let online = last_seen_at > 0 && now.saturating_sub(last_seen_at) < 300;

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "peer_user_id": other_user_id,
            "online": online,
            "last_seen_at": last_seen_at,
            "open_contact": open_contact == 1
        })),
    )
        .into_response()
}

pub async fn api_chat_conversations(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
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

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": "database_unavailable"
                })),
            )
                .into_response();
        }
    };

    super::chat::mark_user_messages_delivered(&db, user_id);
    let conversations = load_user_conversations(&db, user_id);
    let total_unread: i64 = conversations.iter().map(|row| row.unread_count).sum();

    let items: Vec<serde_json::Value> = conversations
        .iter()
        .map(|conversation| {
            let display_name = crate::web::templates::conversation_display_name(
                conversation.other_user_id,
                &conversation.username,
                &conversation.first_name,
                &conversation.last_name,
            );
            let last_message = if conversation.last_message.is_empty() {
                "Новый диалог".to_string()
            } else {
                conversation.last_message.clone()
            };
            let last_time = if conversation.last_message.is_empty() {
                String::new()
            } else {
                crate::web::templates::format_inbox_time(conversation.updated_at)
            };

            json!({
                "other_user_id": conversation.other_user_id,
                "display_name": display_name,
                "username": conversation.username,
                "last_message": last_message,
                "last_time": last_time,
                "unread_count": conversation.unread_count,
                "updated_at": conversation.updated_at,
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "total_unread": total_unread,
            "conversations": items,
        })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ChatReactPayload {
    emoji: String,
}

pub async fn api_chat_react(
    State(state): State<AppState>,
    Path((other_user_id, message_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    Json(payload): Json<ChatReactPayload>,
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

    if normalized_pair(user_id, other_user_id).is_none() || message_id <= 0 {
        return json_error(StatusCode::BAD_REQUEST, "invalid_request");
    }

    let emoji = payload.emoji.trim();

    if !reaction_emoji_is_allowed(emoji) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_reaction");
    }

    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable");
        }
    };

    if users_are_blocked(&connection, user_id, other_user_id) {
        return json_error(StatusCode::FORBIDDEN, "user_blocked");
    }

    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(conversation_id) => conversation_id,
        None => {
            return json_error(StatusCode::FORBIDDEN, "conversation_not_open");
        }
    };

    let message_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM messages
             WHERE id = ?1
               AND conversation_id = ?2
               AND deleted_at = 0",
            rusqlite::params![message_id, conversation_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if message_exists == 0 {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    }

    let existing_emoji: Option<String> = connection
        .query_row(
            "SELECT emoji
             FROM message_reactions
             WHERE message_id = ?1
               AND user_id = ?2
             LIMIT 1",
            rusqlite::params![message_id, user_id],
            |row| row.get(0),
        )
        .ok();

    if existing_emoji.as_deref() == Some(emoji) {
        let _ = connection.execute(
            "DELETE FROM message_reactions
             WHERE message_id = ?1
               AND user_id = ?2",
            rusqlite::params![message_id, user_id],
        );
    } else {
        let now = crate::web::handlers::common::unix_now();
        let changed = connection
            .execute(
                "INSERT INTO message_reactions (message_id, user_id, emoji, created_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(message_id, user_id)
                 DO UPDATE SET emoji = excluded.emoji,
                               created_at = excluded.created_at",
                rusqlite::params![message_id, user_id, emoji, now],
            )
            .unwrap_or(0);

        if changed == 0 {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "reaction_save_failed");
        }
    }

    let Some(message) = load_api_message_by_id(&connection, conversation_id, message_id, user_id)
    else {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_reload_failed");
    };

    state.publish_chat_event(
        "message.reacted",
        conversation_id,
        message_id,
        user_id,
        other_user_id,
    );

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message_id": message_id,
            "reactions": message.reactions,
            "message": message
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
    fn chat_edit_window_is_enforced() {
        assert!(message_can_be_edited(1_000, 0, 1_000 + 86_400));
        assert!(!message_can_be_edited(1_000, 0, 1_000 + 86_401));
        assert!(!message_can_be_edited(1_000, 2_000, 1_001));
    }

    #[test]
    fn chat_reaction_emoji_is_whitelisted() {
        assert!(reaction_emoji_is_allowed("❤️"));
        assert!(reaction_emoji_is_allowed("👍"));
        assert!(!reaction_emoji_is_allowed("💣"));
        assert!(!reaction_emoji_is_allowed("<script>"));
    }

    #[test]
    fn client_message_identity_is_strict() {
        assert!(client_message_id_is_valid(
            "018f7f43-9f52-7d20-a612-4f006c663f10"
        ));
        assert!(client_message_id_is_valid("fallback_1234567890"));
        assert!(!client_message_id_is_valid(""));
        assert!(!client_message_id_is_valid("short"));
        assert!(!client_message_id_is_valid("identity with spaces"));
    }

    #[test]
    fn lifecycle_cursor_rules_are_stable() {
        let payload = ChatSendPayload {
            message: "Ответ".to_string(),
            reply_to_message_id: Some(42),
            client_message_id: None,
        };

        assert_eq!(payload.reply_to_message_id, Some(42));

        let query = ChatMessagesQuery {
            before_id: Some(10),
            after_id: None,
            limit: Some(50),
            mark_read: None,
            read_through_id: None,
        };

        assert_eq!(query.before_id, Some(10));
        assert_eq!(query.after_id, None);
        assert_eq!(normalized_limit(query.limit), 50);
    }
}
