use super::auth::verify_user_session;
use super::common::{input_text_is_valid, rate_limit_retry_after, request_is_cross_site, unix_now};
use super::user_blocks::users_are_blocked;
use crate::state::app_state::AppState;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::{Path as FsPath, PathBuf};

const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Serialize)]
struct MediaChatMessage {
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
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(json!({"ok": false, "error": error}))).into_response()
}

fn normalized_pair(a: i64, b: i64) -> Option<(i64, i64)> {
    if a <= 0 || b <= 0 || a == b {
        return None;
    }
    if a < b {
        Some((a, b))
    } else {
        Some((b, a))
    }
}

fn conversation_id(conn: &rusqlite::Connection, a: i64, b: i64) -> Option<i64> {
    let (u1, u2) = normalized_pair(a, b)?;
    conn.query_row(
        "SELECT id FROM conversations WHERE user1_id = ?1 AND user2_id = ?2 LIMIT 1",
        rusqlite::params![u1, u2],
        |row| row.get(0),
    )
    .ok()
}

fn client_message_id_is_valid(value: &str) -> bool {
    (16..=80).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn media_root() -> PathBuf {
    PathBuf::from("data/chat-media")
}

fn detect_image(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.len() >= 3 && bytes[0] == 0xff && bytes[1] == 0xd8 && bytes[2] == 0xff {
        return Some(("image", "image/jpeg"));
    }
    if bytes.len() >= 8
        && bytes[0] == 0x89
        && bytes[1] == 0x50
        && bytes[2] == 0x4e
        && bytes[3] == 0x47
    {
        return Some(("image", "image/png"));
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(("image", "image/webp"));
    }
    None
}

fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => "bin",
    }
}

fn random_file_stem() -> String {
    let mut bytes = [0u8; 16];
    let _ = getrandom::getrandom(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn load_message(
    conn: &rusqlite::Connection,
    conversation_id: i64,
    message_id: i64,
    user_id: i64,
) -> Option<MediaChatMessage> {
    conn.query_row(
        "SELECT messages.id, messages.sender_user_id, messages.message,
                messages.delivered_at, messages.read_at, messages.created_at,
                messages.reply_to_message_id,
                (SELECT reply.sender_user_id FROM messages AS reply
                  WHERE reply.id = messages.reply_to_message_id AND reply.conversation_id = messages.conversation_id),
                COALESCE((SELECT CASE WHEN reply.deleted_at > 0 THEN 'Сообщение удалено' ELSE reply.message END
                          FROM messages AS reply
                          WHERE reply.id = messages.reply_to_message_id AND reply.conversation_id = messages.conversation_id), ''),
                messages.edited_at, messages.deleted_at,
                COALESCE(messages.client_message_id, ''),
                COALESCE(messages.attachment_kind, ''),
                COALESCE(messages.attachment_mime, ''),
                COALESCE(messages.attachment_size, 0),
                COALESCE(messages.attachment_path, '')
         FROM messages WHERE messages.id = ?1 AND messages.conversation_id = ?2 LIMIT 1",
        rusqlite::params![message_id, conversation_id],
        |row| {
            let id: i64 = row.get(0)?;
            let sender: i64 = row.get(1)?;
            let deleted_at: i64 = row.get(10)?;
            let kind: String = row.get(12)?;
            let path: String = row.get(15)?;
            let attachment_url = if deleted_at == 0 && kind == "image" && !path.is_empty() {
                format!("/api/chat/media/{id}")
            } else { String::new() };
            Ok(MediaChatMessage {
                id, sender_user_id: sender, message: row.get(2)?, is_mine: sender == user_id,
                delivered_at: row.get(3)?, read_at: row.get(4)?, created_at: row.get(5)?,
                reply_to_message_id: row.get(6)?, reply_sender_user_id: row.get(7)?,
                reply_message: row.get(8)?, edited_at: row.get(9)?, deleted_at,
                client_message_id: row.get(11)?, attachment_kind: kind,
                attachment_mime: row.get(13)?, attachment_size: row.get(14)?, attachment_url,
            })
        },
    ).ok()
}

pub async fn api_chat_send_image(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if request_is_cross_site(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross_site_request_rejected");
    }
    let user_id = match verify_user_session(&state, &headers) {
        Some(v) => v,
        None => return json_error(StatusCode::UNAUTHORIZED, "login_required"),
    };
    if normalized_pair(user_id, other_user_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }
    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "chat_api_send_image", 20, 60).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(json!({"ok": false, "error": "rate_limited", "retry_after": retry_after})),
        )
            .into_response();
    }

    let mut caption = String::new();
    let mut client_message_id = String::new();
    let mut reply_to_message_id: Option<i64> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "caption" | "message" => {
                if let Ok(t) = field.text().await {
                    caption = t.trim().to_string();
                }
            }
            "client_message_id" => {
                if let Ok(t) = field.text().await {
                    client_message_id = t.trim().to_string();
                }
            }
            "reply_to_message_id" => {
                if let Ok(t) = field.text().await {
                    if let Ok(id) = t.trim().parse::<i64>() {
                        if id > 0 {
                            reply_to_message_id = Some(id);
                        }
                    }
                }
            }
            "image" | "file" => {
                if let Ok(b) = field.bytes().await {
                    file_bytes = Some(b.to_vec());
                }
            }
            _ => {}
        }
    }

    let file_bytes = match file_bytes {
        Some(b) if !b.is_empty() => b,
        _ => return json_error(StatusCode::BAD_REQUEST, "image_required"),
    };
    if file_bytes.len() > MAX_IMAGE_BYTES {
        return json_error(StatusCode::BAD_REQUEST, "image_too_large");
    }
    let (kind, mime) = match detect_image(&file_bytes) {
        Some(p) => p,
        None => return json_error(StatusCode::BAD_REQUEST, "unsupported_image"),
    };
    if !client_message_id.is_empty() && !client_message_id_is_valid(&client_message_id) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_client_message_id");
    }
    if !caption.is_empty() && !input_text_is_valid(&caption, 1, 2000) {
        return json_error(StatusCode::BAD_REQUEST, "invalid_message");
    }

    let mut connection = match state.db_pool.get() {
        Ok(c) => c,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    if users_are_blocked(&connection, user_id, other_user_id) {
        return json_error(StatusCode::FORBIDDEN, "user_blocked");
    }
    let conversation_id = match conversation_id(&connection, user_id, other_user_id) {
        Some(id) => id,
        None => return json_error(StatusCode::FORBIDDEN, "conversation_not_open"),
    };

    if !client_message_id.is_empty() {
        if let Ok(existing_id) = connection.query_row(
            "SELECT id FROM messages WHERE sender_user_id = ?1 AND client_message_id = ?2 LIMIT 1",
            rusqlite::params![user_id, client_message_id],
            |row| row.get::<_, i64>(0),
        ) {
            if let Some(message) = load_message(&connection, conversation_id, existing_id, user_id)
            {
                return (
                    StatusCode::OK,
                    Json(json!({"ok": true, "duplicate": true, "message": message})),
                )
                    .into_response();
            }
        }
    }

    if let Some(reply_id) = reply_to_message_id {
        let exists: i64 = connection.query_row(
            "SELECT COUNT(*) FROM messages WHERE id = ?1 AND conversation_id = ?2 AND deleted_at = 0",
            rusqlite::params![reply_id, conversation_id],
            |row| row.get(0),
        ).unwrap_or(0);
        if exists != 1 {
            return json_error(StatusCode::BAD_REQUEST, "invalid_reply");
        }
    }

    let relative = format!(
        "{}/{}.{}",
        conversation_id,
        random_file_stem(),
        extension_for_mime(mime)
    );
    let absolute = media_root().join(&relative);
    if let Some(parent) = absolute.parent() {
        if fs::create_dir_all(parent).is_err() {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "media_store_failed");
        }
    }
    match fs::File::create(&absolute) {
        Ok(mut file) => {
            use std::io::Write;
            if file.write_all(&file_bytes).is_err() {
                let _ = fs::remove_file(&absolute);
                return json_error(StatusCode::INTERNAL_SERVER_ERROR, "media_store_failed");
            }
        }
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "media_store_failed"),
    }

    let now = unix_now();
    let transaction =
        match connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
            Ok(tx) => tx,
            Err(_) => {
                let _ = fs::remove_file(&absolute);
                return json_error(StatusCode::CONFLICT, "chat_busy");
            }
        };

    if transaction.execute(
        "INSERT INTO messages (
            conversation_id, sender_user_id, message, is_read, delivered_at, read_at, created_at,
            reply_to_message_id, client_message_id, attachment_kind, attachment_mime, attachment_size, attachment_path
         ) VALUES (?1,?2,?3,0,0,0,?4,?5,?6,?7,?8,?9,?10)",
        rusqlite::params![
            conversation_id, user_id, caption, now, reply_to_message_id, client_message_id,
            kind, mime, file_bytes.len() as i64, relative
        ],
    ).unwrap_or(0) != 1 {
        let _ = fs::remove_file(&absolute);
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_store_failed");
    }
    let message_id = transaction.last_insert_rowid();
    let _ = transaction.execute(
        "UPDATE conversations SET updated_at = ?2 WHERE id = ?1",
        rusqlite::params![conversation_id, now],
    );
    if transaction.commit().is_err() {
        let _ = fs::remove_file(&absolute);
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_store_failed");
    }

    let message = match load_message(&connection, conversation_id, message_id, user_id) {
        Some(m) => m,
        None => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_load_failed"),
    };
    state.publish_chat_event(
        "message.created",
        conversation_id,
        message_id,
        user_id,
        other_user_id,
    );
    (
        StatusCode::OK,
        Json(json!({"ok": true, "message": message})),
    )
        .into_response()
}

pub async fn api_chat_media(
    State(state): State<AppState>,
    Path(message_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(v) => v,
        None => return json_error(StatusCode::UNAUTHORIZED, "login_required"),
    };
    let connection = match state.db_pool.get() {
        Ok(c) => c,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    let row: Result<(i64, String, String), _> = connection.query_row(
        "SELECT m.deleted_at, m.attachment_path, m.attachment_mime
         FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE m.id = ?1 AND (c.user1_id = ?2 OR c.user2_id = ?2) LIMIT 1",
        rusqlite::params![message_id, user_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    );
    let (deleted_at, relative, mime) = match row {
        Ok(r) => r,
        Err(_) => return json_error(StatusCode::NOT_FOUND, "media_not_found"),
    };
    if deleted_at > 0
        || relative.trim().is_empty()
        || relative.contains("..")
        || FsPath::new(&relative).is_absolute()
    {
        return json_error(StatusCode::NOT_FOUND, "media_not_found");
    }
    let bytes = match fs::read(media_root().join(&relative)) {
        Ok(b) => b,
        Err(_) => return json_error(StatusCode::NOT_FOUND, "media_not_found"),
    };
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    let ct = if mime.is_empty() {
        "application/octet-stream"
    } else {
        mime.as_str()
    };
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(ct)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}
