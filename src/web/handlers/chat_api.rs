use super::auth::verify_user_session;
use super::common::rate_limit_retry_after;
use crate::state::app_state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct ChatMessagesQuery {
    after_id: Option<i64>,
    before_id: Option<i64>,
    limit: Option<i64>,
    mark_read: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct ChatReactionPayload {
    emoji: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatReaction {
    emoji: String,
    count: i64,
    mine: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatApiMessage {
    id: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
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
    client_message_id: String,
    attachment_kind: String,
    attachment_mime: String,
    attachment_size: i64,
    attachment_url: String,
    reactions: Vec<ChatReaction>,
}

fn serialize_i64_as_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(json!({"ok": false, "error": error}))).into_response()
}

fn request_is_cross_site(headers: &HeaderMap) -> bool {
    headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("cross-site"))
}

fn normalized_pair(a: i64, b: i64) -> Option<(i64, i64)> {
    if a <= 0 || b <= 0 || a == b {
        None
    } else if a < b {
        Some((a, b))
    } else {
        Some((b, a))
    }
}

fn valid_client_id(value: &str) -> bool {
    (16..=80).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn query_truthy(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn conversation_id(
    db: &rusqlite::Connection,
    user_id: i64,
    peer_id: i64,
) -> rusqlite::Result<Option<i64>> {
    let Some((u1, u2)) = normalized_pair(user_id, peer_id) else {
        return Ok(None);
    };
    db.query_row(
        "SELECT id FROM conversations WHERE user1_id=?1 AND user2_id=?2 LIMIT 1",
        params![u1, u2],
        |row| row.get(0),
    )
    .optional()
}

fn ensure_conversation(
    db: &rusqlite::Connection,
    user_id: i64,
    peer_id: i64,
) -> Result<i64, &'static str> {
    if let Ok(Some(id)) = conversation_id(db, user_id, peer_id) {
        return Ok(id);
    }

    let Some((u1, u2)) = normalized_pair(user_id, peer_id) else {
        return Err("invalid_user");
    };

    let peer_exists = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=?1 AND is_active=1)",
            params![peer_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    if peer_exists != 1 {
        return Err("user_not_found");
    }

    let now = crate::web::handlers::common::unix_now();
    db.execute(
        "INSERT OR IGNORE INTO conversations(user1_id,user2_id,created_at,updated_at)
         VALUES(?1,?2,?3,?3)",
        params![u1, u2, now],
    )
    .map_err(|_| "conversation_create_failed")?;

    conversation_id(db, user_id, peer_id)
        .ok()
        .flatten()
        .ok_or("conversation_lookup_failed")
}

fn blocked(db: &rusqlite::Connection, user_id: i64, peer_id: i64) -> bool {
    db.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM user_blocks
            WHERE (blocker_user_id=?1 AND blocked_user_id=?2)
               OR (blocker_user_id=?2 AND blocked_user_id=?1)
        )",
        params![user_id, peer_id],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        == 1
}

fn attachment_url(id: i64, deleted_at: i64, kind: &str, path: &str) -> String {
    if deleted_at > 0 || kind.is_empty() || path.is_empty() {
        String::new()
    } else {
        format!("/api/chat/media/{}", id)
    }
}

fn load_message(
    db: &rusqlite::Connection,
    conversation_id: i64,
    message_id: i64,
    viewer_id: i64,
) -> Option<ChatApiMessage> {
    let mut m = db
        .query_row(
            r#"SELECT
                m.id,m.sender_user_id,m.message,m.delivered_at,m.read_at,m.created_at,
                m.reply_to_message_id,
                (SELECT r.sender_user_id FROM messages r
                 WHERE r.id=m.reply_to_message_id AND r.conversation_id=m.conversation_id),
                COALESCE((SELECT CASE WHEN r.deleted_at>0 THEN 'Сообщение удалено' ELSE r.message END
                 FROM messages r WHERE r.id=m.reply_to_message_id AND r.conversation_id=m.conversation_id),''),
                m.edited_at,m.deleted_at,COALESCE(m.client_message_id,''),
                COALESCE(m.attachment_kind,''),COALESCE(m.attachment_mime,''),
                COALESCE(m.attachment_size,0),COALESCE(m.attachment_path,'')
            FROM messages m
            WHERE m.id=?1 AND m.conversation_id=?2
            LIMIT 1"#,
            params![message_id, conversation_id],
            |row| {
                let id: i64 = row.get(0)?;
                let sender: i64 = row.get(1)?;
                let deleted_at: i64 = row.get(10)?;
                let kind: String = row.get(12)?;
                let path: String = row.get(15)?;
                Ok(ChatApiMessage {
                    id,
                    sender_user_id: sender,
                    message: row.get(2)?,
                    is_mine: sender == viewer_id,
                    delivered_at: row.get(3)?,
                    read_at: row.get(4)?,
                    created_at: row.get(5)?,
                    reply_to_message_id: row.get(6)?,
                    reply_sender_user_id: row.get(7)?,
                    reply_message: row.get(8)?,
                    edited_at: row.get(9)?,
                    deleted_at,
                    client_message_id: row.get(11)?,
                    attachment_kind: kind.clone(),
                    attachment_mime: row.get(13)?,
                    attachment_size: row.get(14)?,
                    attachment_url: attachment_url(id, deleted_at, &kind, &path),
                    reactions: vec![],
                })
            },
        )
        .ok()?;

    attach_reactions(db, std::slice::from_mut(&mut m), viewer_id);
    Some(m)
}

pub(super) fn reactions_for_view(
    db: &rusqlite::Connection,
    message_ids: &[i64],
    viewer_id: i64,
) -> BTreeMap<i64, Vec<ChatReaction>> {
    let mut out = BTreeMap::new();
    for id in message_ids {
        let mut items = vec![];
        if let Ok(mut stmt) = db.prepare(
            "SELECT emoji,COUNT(*),MAX(CASE WHEN user_id=?2 THEN 1 ELSE 0 END)
             FROM message_reactions
             WHERE message_id=?1
             GROUP BY emoji
             ORDER BY COUNT(*) DESC, emoji ASC",
        ) {
            if let Ok(rows) = stmt.query_map(params![id, viewer_id], |row| {
                Ok(ChatReaction {
                    emoji: row.get(0)?,
                    count: row.get(1)?,
                    mine: row.get::<_, i64>(2)? == 1,
                })
            }) {
                items.extend(rows.flatten());
            }
        }
        out.insert(*id, items);
    }
    out
}

fn attach_reactions(db: &rusqlite::Connection, messages: &mut [ChatApiMessage], viewer_id: i64) {
    let ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
    let map = reactions_for_view(db, &ids, viewer_id);
    for m in messages {
        m.reactions = map.get(&m.id).cloned().unwrap_or_default();
    }
}

pub async fn api_chat_messages(
    State(state): State<AppState>,
    Path(peer_id): Path<i64>,
    headers: HeaderMap,
    Query(query): Query<ChatMessagesQuery>,
) -> Response {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    if normalized_pair(user_id, peer_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }
    if query.before_id.is_some() && query.after_id.is_some() {
        return json_error(StatusCode::BAD_REQUEST, "conflicting_cursor");
    }

    let mark_read = query_truthy(query.mark_read.as_deref());

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };

    let Ok(Some(conversation_id)) = conversation_id(&db, user_id, peer_id) else {
        return (
            StatusCode::OK,
            Json(json!({
                "ok": true, "messages": [], "has_more": false, "peer_read_through_id": 0
            })),
        )
            .into_response();
    };

    let limit = query.limit.unwrap_or(100).clamp(1, 150);
    let mut messages: Vec<ChatApiMessage> = vec![];

    let sql_after = r#"SELECT
        m.id,m.sender_user_id,m.message,m.delivered_at,m.read_at,m.created_at,
        m.reply_to_message_id,
        (SELECT r.sender_user_id FROM messages r WHERE r.id=m.reply_to_message_id AND r.conversation_id=m.conversation_id),
        COALESCE((SELECT CASE WHEN r.deleted_at>0 THEN 'Сообщение удалено' ELSE r.message END
          FROM messages r WHERE r.id=m.reply_to_message_id AND r.conversation_id=m.conversation_id),''),
        m.edited_at,m.deleted_at,COALESCE(m.client_message_id,''),
        COALESCE(m.attachment_kind,''),COALESCE(m.attachment_mime,''),
        COALESCE(m.attachment_size,0),COALESCE(m.attachment_path,'')
      FROM messages m
      WHERE m.conversation_id=?1 AND m.id>?2
      ORDER BY m.id ASC LIMIT ?3"#;

    let sql_before = r#"SELECT
        m.id,m.sender_user_id,m.message,m.delivered_at,m.read_at,m.created_at,
        m.reply_to_message_id,
        (SELECT r.sender_user_id FROM messages r WHERE r.id=m.reply_to_message_id AND r.conversation_id=m.conversation_id),
        COALESCE((SELECT CASE WHEN r.deleted_at>0 THEN 'Сообщение удалено' ELSE r.message END
          FROM messages r WHERE r.id=m.reply_to_message_id AND r.conversation_id=m.conversation_id),''),
        m.edited_at,m.deleted_at,COALESCE(m.client_message_id,''),
        COALESCE(m.attachment_kind,''),COALESCE(m.attachment_mime,''),
        COALESCE(m.attachment_size,0),COALESCE(m.attachment_path,'')
      FROM messages m
      WHERE m.conversation_id=?1 AND m.id<?2
      ORDER BY m.id DESC LIMIT ?3"#;

    let initial_load = query.after_id.unwrap_or(0) <= 0 && query.before_id.is_none();
    let cursor = query.after_id.or(query.before_id).unwrap_or(0);
    let sql = if query.before_id.is_some() || initial_load {
        sql_before
    } else {
        sql_after
    };
    let effective_cursor = if query.before_id.is_some() || initial_load {
        if cursor <= 0 {
            i64::MAX
        } else {
            cursor
        }
    } else {
        cursor.max(0)
    };

    let rows_result = db.prepare(sql).and_then(|mut stmt| {
        stmt.query_map(
            params![conversation_id, effective_cursor, limit + 1],
            |row| {
                let id: i64 = row.get(0)?;
                let sender: i64 = row.get(1)?;
                let deleted_at: i64 = row.get(10)?;
                let kind: String = row.get(12)?;
                let path: String = row.get(15)?;
                Ok(ChatApiMessage {
                    id,
                    sender_user_id: sender,
                    message: row.get(2)?,
                    is_mine: sender == user_id,
                    delivered_at: row.get(3)?,
                    read_at: row.get(4)?,
                    created_at: row.get(5)?,
                    reply_to_message_id: row.get(6)?,
                    reply_sender_user_id: row.get(7)?,
                    reply_message: row.get(8)?,
                    edited_at: row.get(9)?,
                    deleted_at,
                    client_message_id: row.get(11)?,
                    attachment_kind: kind.clone(),
                    attachment_mime: row.get(13)?,
                    attachment_size: row.get(14)?,
                    attachment_url: attachment_url(id, deleted_at, &kind, &path),
                    reactions: vec![],
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()
    });

    let mut loaded = match rows_result {
        Ok(v) => v,
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_query_failed"),
    };

    let has_more = loaded.len() as i64 > limit;
    if has_more {
        loaded.truncate(limit as usize);
    }
    if query.before_id.is_some() || initial_load {
        loaded.reverse();
    }

    let latest_visible = loaded.iter().map(|m| m.id).max().unwrap_or(0);
    let now = crate::web::handlers::common::unix_now();

    if latest_visible > 0 {
        let _ = db.execute(
            "UPDATE messages SET delivered_at=?4
             WHERE conversation_id=?1 AND sender_user_id=?2 AND id<=?3 AND delivered_at=0",
            params![conversation_id, peer_id, latest_visible, now],
        );
    }

    if mark_read && latest_visible > 0 {
        let read_through = query
            .read_through_id
            .filter(|v| *v > 0)
            .map(|v| v.min(latest_visible))
            .unwrap_or(latest_visible);

        let _ = db.execute(
            "UPDATE messages
             SET is_read=1,
                 delivered_at=CASE WHEN delivered_at=0 THEN ?4 ELSE delivered_at END,
                 read_at=CASE WHEN read_at=0 THEN ?4 ELSE read_at END
             WHERE conversation_id=?1 AND sender_user_id=?2 AND id<=?3 AND read_at=0",
            params![conversation_id, peer_id, read_through, now],
        );
    }

    for m in &mut loaded {
        if !m.is_mine && m.delivered_at == 0 {
            m.delivered_at = now;
        }
        if !m.is_mine && mark_read && m.read_at == 0 {
            m.read_at = now;
        }
    }

    attach_reactions(&db, &mut loaded, user_id);

    let peer_read_through_id = db
        .query_row(
            "SELECT COALESCE(MAX(id),0) FROM messages
             WHERE conversation_id=?1 AND sender_user_id=?2 AND read_at>0",
            params![conversation_id, user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    messages.append(&mut loaded);

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
    Path(peer_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ChatSendPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    if normalized_pair(user_id, peer_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }
    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "chat_send", 30, 60).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"ok": false, "error": "rate_limited", "retry_after": retry_after})),
        )
            .into_response();
    }

    let text = payload.message.trim().to_string();
    if text.is_empty() || text.chars().count() > 2000 {
        return json_error(StatusCode::BAD_REQUEST, "invalid_message");
    }

    let client_id = payload.client_message_id.unwrap_or_default();
    if !client_id.is_empty() && !valid_client_id(&client_id) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_client_message_id");
    }

    let mut db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    if blocked(&db, user_id, peer_id) {
        return json_error(StatusCode::FORBIDDEN, "user_blocked");
    }

    let conversation_id = match ensure_conversation(&db, user_id, peer_id) {
        Ok(id) => id,
        Err(e) => return json_error(StatusCode::BAD_REQUEST, e),
    };

    if !client_id.is_empty() {
        if let Ok(Some(existing)) = db
            .query_row(
                "SELECT id FROM messages WHERE sender_user_id=?1 AND client_message_id=?2 LIMIT 1",
                params![user_id, client_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
        {
            if let Some(message) = load_message(&db, conversation_id, existing, user_id) {
                return (
                    StatusCode::OK,
                    Json(json!({"ok": true, "duplicate": true, "message": message})),
                )
                    .into_response();
            }
        }
    }

    if let Some(reply_id) = payload.reply_to_message_id {
        let valid = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM messages WHERE id=?1 AND conversation_id=?2)",
                params![reply_id, conversation_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);
        if valid != 1 {
            return json_error(StatusCode::BAD_REQUEST, "invalid_reply");
        }
    }

    let now = crate::web::handlers::common::unix_now();
    let tx = match db.transaction_with_behavior(TransactionBehavior::Immediate) {
        Ok(tx) => tx,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_busy"),
    };

    if tx
        .execute(
            "INSERT INTO messages(
                conversation_id,sender_user_id,message,is_read,
                delivered_at,read_at,created_at,reply_to_message_id,client_message_id
             ) VALUES(?1,?2,?3,0,0,0,?4,?5,?6)",
            params![
                conversation_id,
                user_id,
                text,
                now,
                payload.reply_to_message_id,
                client_id
            ],
        )
        .is_err()
    {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_store_failed");
    }

    let message_id = tx.last_insert_rowid();
    let _ = tx.execute(
        "UPDATE conversations SET updated_at=?2 WHERE id=?1",
        params![conversation_id, now],
    );
    if tx.commit().is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_commit_failed");
    }

    let Some(message) = load_message(&db, conversation_id, message_id, user_id) else {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_load_failed");
    };

    (
        StatusCode::CREATED,
        Json(json!({"ok": true, "message": message})),
    )
        .into_response()
}

pub async fn api_chat_conversations(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };

    let rows = db
        .prepare(
            r#"SELECT
                c.id,
                CASE WHEN c.user1_id=?1 THEN c.user2_id ELSE c.user1_id END,
                COALESCE(p.username,''),COALESCE(p.first_name,''),COALESCE(p.last_name,''),
                COALESCE((SELECT CASE
                    WHEN m.deleted_at>0 THEN 'Сообщение удалено'
                    WHEN m.attachment_kind='image' THEN '📷 Фото'
                    WHEN m.attachment_kind='voice' THEN '🎤 Голосовое сообщение'
                    ELSE m.message END
                  FROM messages m WHERE m.conversation_id=c.id ORDER BY m.id DESC LIMIT 1),''),
                COALESCE((SELECT COUNT(*) FROM messages m
                  WHERE m.conversation_id=c.id AND m.sender_user_id<>?1 AND m.read_at=0),0),
                c.updated_at
             FROM conversations c
             LEFT JOIN profiles p ON p.user_id=CASE WHEN c.user1_id=?1 THEN c.user2_id ELSE c.user1_id END
             WHERE c.user1_id=?1 OR c.user2_id=?1
             ORDER BY c.updated_at DESC,c.id DESC LIMIT 250"#,
        )
        .and_then(|mut stmt| {
            stmt.query_map(params![user_id], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "peer_user_id": row.get::<_, i64>(1)?.to_string(),
                    "username": row.get::<_, String>(2)?,
                    "first_name": row.get::<_, String>(3)?,
                    "last_name": row.get::<_, String>(4)?,
                    "last_message": row.get::<_, String>(5)?,
                    "unread_count": row.get::<_, i64>(6)?,
                    "updated_at": row.get::<_, i64>(7)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(json!({"ok": true, "conversations": rows})),
    )
        .into_response()
}

pub async fn api_chat_peer(
    State(state): State<AppState>,
    Path(peer_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    if normalized_pair(user_id, peer_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };

    let peer = db
        .query_row(
            "SELECT COALESCE(username,''),COALESCE(first_name,''),COALESCE(last_name,''),COALESCE(last_seen_at,0)
             FROM profiles WHERE user_id=?1 LIMIT 1",
            params![peer_id],
            |row| {
                Ok(json!({
                    "user_id": peer_id.to_string(),
                    "username": row.get::<_, String>(0)?,
                    "first_name": row.get::<_, String>(1)?,
                    "last_name": row.get::<_, String>(2)?,
                    "last_seen_at": row.get::<_, i64>(3)?,
                }))
            },
        )
        .unwrap_or_else(|_| json!({"user_id": peer_id.to_string(), "last_seen_at": 0}));

    (StatusCode::OK, Json(json!({"ok": true, "peer": peer}))).into_response()
}

pub async fn api_chat_edit(
    State(state): State<AppState>,
    Path((peer_id, message_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    Json(payload): Json<ChatEditPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    let text = payload.message.trim();
    if text.is_empty() || text.chars().count() > 2000 {
        return json_error(StatusCode::BAD_REQUEST, "invalid_message");
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    let Ok(Some(conversation_id)) = conversation_id(&db, user_id, peer_id) else {
        return json_error(StatusCode::NOT_FOUND, "conversation_not_found");
    };
    let now = crate::web::handlers::common::unix_now();
    let changed = db
        .execute(
            "UPDATE messages SET message=?4,edited_at=?5
             WHERE id=?1 AND conversation_id=?2 AND sender_user_id=?3
               AND deleted_at=0 AND attachment_kind=''",
            params![message_id, conversation_id, user_id, text, now],
        )
        .unwrap_or(0);
    if changed == 0 {
        return json_error(StatusCode::FORBIDDEN, "message_not_editable");
    }
    let Some(message) = load_message(&db, conversation_id, message_id, user_id) else {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    };
    (
        StatusCode::OK,
        Json(json!({"ok": true, "message": message})),
    )
        .into_response()
}

pub async fn api_chat_delete(
    State(state): State<AppState>,
    Path((peer_id, message_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    let Ok(Some(conversation_id)) = conversation_id(&db, user_id, peer_id) else {
        return json_error(StatusCode::NOT_FOUND, "conversation_not_found");
    };
    let attachment_path: String = db
        .query_row(
            "SELECT COALESCE(attachment_path,'') FROM messages
             WHERE id=?1 AND conversation_id=?2 AND sender_user_id=?3 AND deleted_at=0",
            params![message_id, conversation_id, user_id],
            |row| row.get(0),
        )
        .unwrap_or_default();
    let now = crate::web::handlers::common::unix_now();
    let changed = db
        .execute(
            "UPDATE messages
             SET message='',deleted_at=?4,attachment_kind='',attachment_mime='',attachment_size=0,attachment_path=''
             WHERE id=?1 AND conversation_id=?2 AND sender_user_id=?3 AND deleted_at=0",
            params![message_id, conversation_id, user_id, now],
        )
        .unwrap_or(0);
    if changed == 0 {
        return json_error(StatusCode::FORBIDDEN, "message_not_deletable");
    }
    if !attachment_path.is_empty() {
        let media_root = std::path::Path::new("data/chat-media-next");
        if let (Ok(root), Ok(file)) = (
            std::fs::canonicalize(media_root),
            std::fs::canonicalize(std::path::Path::new(&attachment_path)),
        ) {
            if file.starts_with(&root) {
                let _ = std::fs::remove_file(file);
            }
        }
    }
    let _ = db.execute(
        "DELETE FROM message_reactions WHERE message_id=?1",
        params![message_id],
    );
    let Some(message) = load_message(&db, conversation_id, message_id, user_id) else {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    };
    (
        StatusCode::OK,
        Json(json!({"ok": true, "message": message})),
    )
        .into_response()
}

pub async fn api_chat_react(
    State(state): State<AppState>,
    Path((peer_id, message_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    Json(payload): Json<ChatReactionPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    let emoji = payload.emoji.trim();
    const ALLOWED: &[&str] = &["👍", "❤️", "😂", "🔥", "👏", "🙏", "✅", "🤝"];
    if !ALLOWED.contains(&emoji) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_reaction");
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    let Ok(Some(conversation_id)) = conversation_id(&db, user_id, peer_id) else {
        return json_error(StatusCode::NOT_FOUND, "conversation_not_found");
    };
    let exists = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM messages WHERE id=?1 AND conversation_id=?2 AND deleted_at=0)",
            params![message_id, conversation_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    if exists != 1 {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    }

    let current: Option<String> = db
        .query_row(
            "SELECT emoji FROM message_reactions WHERE message_id=?1 AND user_id=?2",
            params![message_id, user_id],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten();

    if current.as_deref() == Some(emoji) {
        let _ = db.execute(
            "DELETE FROM message_reactions WHERE message_id=?1 AND user_id=?2",
            params![message_id, user_id],
        );
    } else {
        let now = crate::web::handlers::common::unix_now();
        let _ = db.execute(
            "INSERT INTO message_reactions(message_id,user_id,emoji,created_at)
             VALUES(?1,?2,?3,?4)
             ON CONFLICT(message_id,user_id) DO UPDATE SET emoji=excluded.emoji,created_at=excluded.created_at",
            params![message_id, user_id, emoji, now],
        );
    }

    let Some(message) = load_message(&db, conversation_id, message_id, user_id) else {
        return json_error(StatusCode::NOT_FOUND, "message_not_found");
    };
    (
        StatusCode::OK,
        Json(json!({"ok": true, "message": message})),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_is_stable() {
        assert_eq!(normalized_pair(9, 7), Some((7, 9)));
        assert_eq!(normalized_pair(7, 9), Some((7, 9)));
        assert_eq!(normalized_pair(7, 7), None);
    }

    #[test]
    fn client_ids_are_restricted() {
        assert!(valid_client_id("1234567890abcdef"));
        assert!(!valid_client_id("short"));
        assert!(!valid_client_id("1234567890abcde!"));
    }

    #[test]
    fn mark_read_query_accepts_browser_values() {
        assert!(query_truthy(Some("1")));
        assert!(query_truthy(Some("true")));
        assert!(query_truthy(Some("TRUE")));
        assert!(!query_truthy(Some("0")));
        assert!(!query_truthy(Some("false")));
        assert!(!query_truthy(None));
    }
}
