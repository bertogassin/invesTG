use super::auth::verify_user_session;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
};

pub(super) fn mark_user_messages_delivered(db: &rusqlite::Connection, user_id: i64) -> usize {
    if user_id <= 0 {
        return 0;
    }

    db.execute(
        "UPDATE messages
         SET delivered_at = strftime('%s','now')
         WHERE delivered_at = 0
           AND sender_user_id <> ?1
           AND conversation_id IN (
               SELECT id
               FROM conversations
               WHERE user1_id = ?1 OR user2_id = ?1
           )",
        rusqlite::params![user_id],
    )
    .unwrap_or(0)
}

pub(super) fn load_user_conversations(
    db: &rusqlite::Connection,
    user_id: i64,
) -> Vec<crate::web::view_models::ConversationRow> {
    db.prepare(
        "SELECT
            c.id,

            CASE
                WHEN c.user1_id = ?1
                THEN c.user2_id
                ELSE c.user1_id
            END AS other_user_id,

            COALESCE(p.username, ''),
            COALESCE(p.first_name, ''),
            COALESCE(p.last_name, ''),

            COALESCE((
                SELECT CASE
                    WHEN m.deleted_at > 0
                    THEN 'Сообщение удалено'
                    ELSE m.message
                END
                FROM messages m
                WHERE m.conversation_id = c.id
                ORDER BY
                    m.created_at DESC,
                    m.id DESC
                LIMIT 1
            ), ''),

            (
                SELECT COUNT(*)
                FROM messages m
                WHERE m.conversation_id = c.id
                  AND m.sender_user_id <> ?1
                  AND m.is_read = 0
            ) AS unread_count,

            c.updated_at

         FROM conversations c

         LEFT JOIN profiles p
           ON p.user_id = CASE
                WHEN c.user1_id = ?1
                THEN c.user2_id
                ELSE c.user1_id
           END

         WHERE c.user1_id = ?1
            OR c.user2_id = ?1

         ORDER BY
            c.updated_at DESC,
            c.id DESC

         LIMIT 200",
    )
    .and_then(|mut stmt| {
        stmt.query_map(rusqlite::params![user_id], |row| {
            Ok(crate::web::view_models::ConversationRow {
                _id: row.get(0)?,
                other_user_id: row.get(1)?,
                username: row.get(2)?,
                first_name: row.get(3)?,
                last_name: row.get(4)?,
                last_message: row.get(5)?,
                unread_count: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
    })
    .unwrap_or_else(|_| vec![])
}

pub async fn messages_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_messages(false, vec![]));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    mark_user_messages_delivered(&db, user_id);
    let conversations = load_user_conversations(&db, user_id);

    drop(db);

    Html(templates::render_messages(true, conversations))
}

pub async fn chat_page(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,
        None => {
            return Html(templates::render_chat(
                false,
                0,
                0,
                "",
                "",
                "",
                vec![],
                None,
            ));
        }
    };

    if other_user_id <= 0 || other_user_id == user_id {
        return Html(templates::render_chat(
            true,
            user_id,
            other_user_id,
            "",
            "",
            "",
            vec![],
            None,
        ));
    }

    let (user1_id, user2_id) = if user_id < other_user_id {
        (user_id, other_user_id)
    } else {
        (other_user_id, user_id)
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let _ = db.execute(
        "UPDATE profiles
         SET last_seen_at = strftime('%s','now')
         WHERE user_id = ?1",
        rusqlite::params![user_id],
    );

    let conversation_id: Option<i64> = db
        .query_row(
            "SELECT id
             FROM conversations
             WHERE user1_id = ?1
               AND user2_id = ?2
             LIMIT 1",
            rusqlite::params![user1_id, user2_id,],
            |row| row.get(0),
        )
        .ok();

    let conversation_id = conversation_id.unwrap_or(0);

    let other_profile: Option<(String, String, String)> = db
        .query_row(
            "SELECT
                COALESCE(username, ''),
                COALESCE(first_name, ''),
                COALESCE(last_name, '')
             FROM profiles
             WHERE user_id = ?1
             LIMIT 1",
            rusqlite::params![other_user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let (other_username, other_first_name, other_last_name) =
        other_profile.unwrap_or_else(|| (String::new(), String::new(), String::new()));

    let contact_request = db
        .query_row(
            "SELECT id, sender_user_id, status
             FROM contact_requests
             WHERE (
                 sender_user_id = ?1 AND receiver_user_id = ?2
             ) OR (
                 sender_user_id = ?2 AND receiver_user_id = ?1
             )
             ORDER BY id DESC
             LIMIT 1",
            rusqlite::params![user_id, other_user_id],
            |row| {
                Ok(templates::ChatContactRequestState {
                    id: row.get(0)?,
                    sender_user_id: row.get(1)?,
                    status: row.get(2)?,
                })
            },
        )
        .ok();

    let mut messages: Vec<crate::web::view_models::ChatMessageRow> = if conversation_id <= 0 {
        Vec::new()
    } else {
        db
        .prepare(
            "SELECT
                messages.id,
                messages.sender_user_id,
                messages.message,
                messages.is_read,
                messages.created_at,
                messages.delivered_at,
                messages.read_at,
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
                COALESCE(messages.attachment_kind, ''),
                COALESCE(messages.attachment_path, '')
             FROM (
                SELECT id
                FROM messages
                WHERE conversation_id = ?1
                ORDER BY id DESC
                LIMIT 100
             ) AS recent
             INNER JOIN messages
               ON messages.id = recent.id
             ORDER BY messages.id ASC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![conversation_id], |row| {
                let deleted_at: i64 = row.get(11)?;
                let attachment_kind: String = row.get(12)?;
                let attachment_path: String = row.get(13)?;
                let message_id: i64 = row.get(0)?;

                Ok(crate::web::view_models::ChatMessageRow {
                    id: message_id,
                    sender_user_id: row.get(1)?,
                    message: row.get(2)?,
                    is_read: row.get(3)?,
                    created_at: row.get(4)?,
                    delivered_at: row.get(5)?,
                    read_at: row.get(6)?,
                    reply_to_message_id: row.get(7)?,
                    reply_sender_user_id: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                    reply_message: row.get(9)?,
                    edited_at: row.get(10)?,
                    deleted_at,
                    attachment_kind: attachment_kind.clone(),
                    attachment_url: if deleted_at == 0
                        && (attachment_kind == "image" || attachment_kind == "voice")
                        && !attachment_path.is_empty()
                    {
                        format!("/api/chat/media/{message_id}")
                    } else {
                        String::new()
                    },
                    reactions: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default()
    };

    let message_ids: Vec<i64> = messages.iter().map(|message| message.id).collect();
    let reactions_by_message = super::chat_api::reactions_for_view(&db, &message_ids, user_id);

    for message in &mut messages {
        if let Some(reactions) = reactions_by_message.get(&message.id) {
            message.reactions = reactions.clone();
        }
    }

    let read_through_id = messages
        .iter()
        .filter(|message| message.sender_user_id == other_user_id)
        .map(|message| message.id)
        .max()
        .unwrap_or(0);

    let read_changed = if read_through_id > 0 {
        let now = crate::web::handlers::common::unix_now();

        db.execute(
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
        .unwrap_or(0)
    } else {
        0
    };

    let _ = db.execute(
        "UPDATE user_notifications
         SET is_read = 1
         WHERE user_id = ?1
           AND kind = 'chat_message'
           AND is_read = 0
           AND resource_id = ?2",
        rusqlite::params![user_id, other_user_id],
    );

    drop(db);

    if read_changed > 0 {
        state.publish_chat_event(
            "message.read",
            conversation_id,
            read_through_id,
            user_id,
            other_user_id,
        );
    }

    Html(templates::render_chat(
        true,
        user_id,
        other_user_id,
        &other_username,
        &other_first_name,
        &other_last_name,
        messages,
        contact_request.as_ref(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_marks_only_incoming_messages_delivered() {
        let db = rusqlite::Connection::open_in_memory().expect("database");
        db.execute_batch(
            "CREATE TABLE conversations (
                id INTEGER PRIMARY KEY,
                user1_id INTEGER NOT NULL,
                user2_id INTEGER NOT NULL
             );
             CREATE TABLE messages (
                id INTEGER PRIMARY KEY,
                conversation_id INTEGER NOT NULL,
                sender_user_id INTEGER NOT NULL,
                delivered_at INTEGER NOT NULL DEFAULT 0
             );
             INSERT INTO conversations (id, user1_id, user2_id)
             VALUES (1, 10, 20), (2, 20, 30);
             INSERT INTO messages (id, conversation_id, sender_user_id)
             VALUES (1, 1, 10), (2, 1, 20), (3, 2, 30);",
        )
        .expect("schema");

        assert_eq!(mark_user_messages_delivered(&db, 20), 2);

        let delivered: Vec<(i64, i64)> = db
            .prepare("SELECT id, delivered_at FROM messages ORDER BY id")
            .expect("statement")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("rows")
            .collect::<Result<_, _>>()
            .expect("values");

        assert!(delivered[0].1 > 0);
        assert_eq!(delivered[1].1, 0);
        assert!(delivered[2].1 > 0);
    }
}
