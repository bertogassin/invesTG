use super::auth::verify_user_session;
use super::common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
};
use super::types::ChatMessageForm;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use teloxide::prelude::*;

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
                id: row.get(0)?,
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
            return Html(templates::render_chat(false, 0, 0, "", "", "", vec![]));
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

    let conversation_id = match conversation_id {
        Some(id) => id,

        None => {
            drop(db);

            return Html(templates::render_chat(
                true,
                user_id,
                other_user_id,
                "",
                "",
                "",
                vec![],
            ));
        }
    };

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

    let messages: Vec<crate::web::view_models::ChatMessageRow> = db
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
                messages.deleted_at
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
                Ok(crate::web::view_models::ChatMessageRow {
                    id: row.get(0)?,
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
                    deleted_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

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

    Html(templates::render_chat(
        true,
        user_id,
        other_user_id,
        &other_username,
        &other_first_name,
        &other_last_name,
        messages,
    ))
}

#[allow(dead_code)]
pub async fn send_chat_message(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<ChatMessageForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return (StatusCode::UNAUTHORIZED, "Требуется вход в аккаунт").into_response();
        }
    };

    if other_user_id <= 0 || other_user_id == user_id {
        return (StatusCode::BAD_REQUEST, "Некорректный пользователь").into_response();
    }

    let message = form.message.trim();

    if message.is_empty() {
        return (
            StatusCode::SEE_OTHER,
            [(
                header::LOCATION,
                format!("/app/chat/{}#chat-end", other_user_id),
            )],
        )
            .into_response();
    }

    if !input_text_is_valid(message, 1, 2000) {
        return (
            StatusCode::BAD_REQUEST,
            "Сообщение слишком длинное или содержит недопустимые символы",
        )
            .into_response();
    }

    if let Some(retry_after) = rate_limit_retry_after(&state, user_id, "chat_send", 30, 60).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много сообщений. Попробуйте немного позже.",
        )
            .into_response();
    }

    let (user1_id, user2_id) = if user_id < other_user_id {
        (user_id, other_user_id)
    } else {
        (other_user_id, user_id)
    };

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

    let conversation_id = match conversation_id {
        Some(id) => id,

        None => {
            drop(db);

            return (StatusCode::FORBIDDEN, "Чат между пользователями не открыт").into_response();
        }
    };

    let transaction_result = (|| -> rusqlite::Result<usize> {
        let tx = db.unchecked_transaction()?;

        let inserted = tx.execute(
            "INSERT INTO messages (
                conversation_id,
                sender_user_id,
                message,
                is_read,
                created_at
             )
             VALUES (
                ?1,
                ?2,
                ?3,
                0,
                strftime('%s','now')
             )",
            rusqlite::params![conversation_id, user_id, message],
        )?;

        if inserted == 1 {
            tx.execute(
                "UPDATE conversations
                 SET updated_at = strftime('%s','now')
                 WHERE id = ?1",
                rusqlite::params![conversation_id],
            )?;
        }

        tx.commit()?;
        Ok(inserted)
    })();

    let inserted = match transaction_result {
        Ok(inserted) => inserted,
        Err(err) => {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка сохранения сообщения: {}", err),
            )
                .into_response();
        }
    };

    if inserted == 1 {
        let existing_chat_notification = db
            .execute(
                "UPDATE user_notifications
                 SET created_at = strftime('%s','now')
                 WHERE user_id = ?1
                   AND kind = 'chat_message'
                   AND is_read = 0",
                rusqlite::params![other_user_id],
            )
            .unwrap_or(0);

        if existing_chat_notification == 0 {
            let _ = db.execute(
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
                    ?1,
                    ?2,
                    'chat_message',
                    'Новое сообщение',
                    'У вас новое сообщение в ResursMap.',
                    0,
                    strftime('%s','now')
                 )",
                rusqlite::params![other_user_id, user_id],
            );

            // Отправим уведомление в Telegram, если привязан
            let telegram_id: Option<i64> = db
                .query_row(
                    "SELECT telegram_id FROM users WHERE id = ?1",
                    rusqlite::params![other_user_id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(tg_id) = telegram_id {
                if tg_id > 0 {
                    let bot = Bot::new(state.bot_token.clone());
                    let _ = crate::bot::handler::send_notification(
                        &bot,
                        tg_id,
                        "📩 У вас новое сообщение в ResursMap!\n\nОткройте чат: https://resursmap.de/app/messages",
                    )
                    .await;
                }
            }
        }
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(
            header::LOCATION,
            format!("/app/chat/{}#chat-end", other_user_id),
        )],
    )
        .into_response()
}
