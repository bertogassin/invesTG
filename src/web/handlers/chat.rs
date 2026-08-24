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

    let conversations: Vec<(i64, i64, String, String, String, String, i64, i64)> = db
        .prepare(
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
                    SELECT m.message
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
               ON p.client_id = (
                    'tg:' ||
                    CASE
                        WHEN c.user1_id = ?1
                        THEN c.user2_id
                        ELSE c.user1_id
                    END
               )

             WHERE c.user1_id = ?1
                OR c.user2_id = ?1

             ORDER BY
                c.updated_at DESC,
                c.id DESC

             LIMIT 200",
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
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let _ = db.execute(
        "UPDATE user_notifications
         SET is_read = 1
         WHERE user_id = ?1
           AND kind = 'chat_message'
           AND is_read = 0",
        rusqlite::params![user_id],
    );

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
            return Html(templates::render_chat(false, 0, "", "", "", vec![]));
        }
    };

    if other_user_id <= 0 || other_user_id == user_id {
        return Html(templates::render_chat(
            true,
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
             WHERE client_id = ?1
             LIMIT 1",
            rusqlite::params![format!("tg:{}", other_user_id)],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let (other_username, other_first_name, other_last_name) =
        other_profile.unwrap_or_else(|| (String::new(), String::new(), String::new()));

    let messages: Vec<(i64, i64, String, i64, i64)> = db
        .prepare(
            "SELECT
                id,
                sender_user_id,
                message,
                is_read,
                created_at
             FROM messages
             WHERE conversation_id = ?1
             ORDER BY
                created_at ASC,
                id ASC
             LIMIT 500",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![conversation_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    let _ = db.execute(
        "UPDATE messages
         SET is_read = 1
         WHERE conversation_id = ?1
           AND sender_user_id = ?2
           AND is_read = 0",
        rusqlite::params![conversation_id, other_user_id,],
    );

    drop(db);

    Html(templates::render_chat(
        true,
        other_user_id,
        &other_username,
        &other_first_name,
        &other_last_name,
        messages,
    ))
}

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
            return (StatusCode::UNAUTHORIZED, "Требуется вход через Telegram").into_response();
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
                    NULL,
                    'chat_message',
                    'Новое сообщение',
                    'У вас новое сообщение в ResursMap.',
                    0,
                    strftime('%s','now')
                 )",
                rusqlite::params![other_user_id],
            );
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
