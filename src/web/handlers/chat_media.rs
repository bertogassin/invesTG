use super::auth::verify_user_session;
use super::common::rate_limit_retry_after;
use crate::state::app_state::AppState;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use rusqlite::{params, OptionalExtension};
use serde_json::json;
use std::path::{Path as FsPath, PathBuf};

const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_VOICE_BYTES: usize = 12 * 1024 * 1024;

fn json_error(status: StatusCode, error: &str) -> axum::response::Response {
    (status, Json(json!({"ok": false, "error": error}))).into_response()
}

fn pair(a: i64, b: i64) -> Option<(i64, i64)> {
    if a <= 0 || b <= 0 || a == b {
        None
    } else if a < b {
        Some((a, b))
    } else {
        Some((b, a))
    }
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

fn extension_for_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "audio/webm" => Some("webm"),
        "audio/ogg" => Some("ogg"),
        "audio/mp4" => Some("m4a"),
        "audio/mpeg" => Some("mp3"),
        _ => None,
    }
}

fn ensure_conversation(db: &rusqlite::Connection, user_id: i64, peer_id: i64) -> Option<i64> {
    let (u1, u2) = pair(user_id, peer_id)?;
    let existing = db
        .query_row(
            "SELECT id FROM conversations WHERE user1_id=?1 AND user2_id=?2 LIMIT 1",
            params![u1, u2],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .ok()
        .flatten();
    if existing.is_some() {
        return existing;
    }
    let peer_exists = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=?1 AND is_active=1)",
            params![peer_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    if peer_exists != 1 {
        return None;
    }
    let now = crate::web::handlers::common::unix_now();
    let _ = db.execute(
        "INSERT OR IGNORE INTO conversations(user1_id,user2_id,created_at,updated_at)
         VALUES(?1,?2,?3,?3)",
        params![u1, u2, now],
    );
    db.query_row(
        "SELECT id FROM conversations WHERE user1_id=?1 AND user2_id=?2 LIMIT 1",
        params![u1, u2],
        |row| row.get::<_, i64>(0),
    )
    .ok()
}

fn clean_client_id(value: &str) -> Option<String> {
    let value = value.trim();
    if (16..=80).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        Some(value.to_string())
    } else {
        None
    }
}

fn media_root() -> PathBuf {
    PathBuf::from("data/chat-media-next")
}

async fn parse_upload(
    mut multipart: Multipart,
    wanted_field: &str,
    max_bytes: usize,
) -> Result<(Vec<u8>, String, String, Option<i64>, String), &'static str> {
    let mut bytes = Vec::new();
    let mut mime = String::new();
    let mut caption = String::new();
    let mut reply_to = None;
    let mut client_id = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| "invalid_multipart")?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == wanted_field {
            mime = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            bytes = field
                .bytes()
                .await
                .map_err(|_| "upload_read_failed")?
                .to_vec();
            if bytes.len() > max_bytes {
                return Err("file_too_large");
            }
        } else if name == "caption" {
            caption = field.text().await.unwrap_or_default();
        } else if name == "reply_to_message_id" {
            reply_to = field
                .text()
                .await
                .ok()
                .and_then(|v| v.trim().parse::<i64>().ok());
        } else if name == "client_message_id" {
            client_id = field.text().await.unwrap_or_default();
        }
    }

    if bytes.is_empty() {
        return Err("file_required");
    }
    if caption.chars().count() > 2000 {
        return Err("caption_too_long");
    }
    let Some(client_id) = clean_client_id(&client_id) else {
        return Err("invalid_client_message_id");
    };
    Ok((bytes, mime, caption.trim().to_string(), reply_to, client_id))
}

async fn save_attachment(
    state: AppState,
    headers: HeaderMap,
    peer_id: i64,
    multipart: Multipart,
    field_name: &str,
    kind: &str,
    allowed_mimes: &[&str],
    max_bytes: usize,
) -> axum::response::Response {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return json_error(StatusCode::UNAUTHORIZED, "login_required");
    };
    if pair(user_id, peer_id).is_none() {
        return json_error(StatusCode::BAD_REQUEST, "invalid_user");
    }
    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "chat_media_send", 12, 60).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"ok": false, "error": "rate_limited", "retry_after": retry_after})),
        )
            .into_response();
    }

    let (bytes, mime, caption, reply_to, client_id) =
        match parse_upload(multipart, field_name, max_bytes).await {
            Ok(v) => v,
            Err(e) => return json_error(StatusCode::BAD_REQUEST, e),
        };

    if !allowed_mimes.contains(&mime.as_str()) {
        return json_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return json_error(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable"),
    };
    if blocked(&db, user_id, peer_id) {
        return json_error(StatusCode::FORBIDDEN, "user_blocked");
    }

    let Some(conversation_id) = ensure_conversation(&db, user_id, peer_id) else {
        return json_error(StatusCode::BAD_REQUEST, "conversation_unavailable");
    };

    if let Ok(Some(existing)) = db
        .query_row(
            "SELECT id FROM messages WHERE sender_user_id=?1 AND client_message_id=?2 LIMIT 1",
            params![user_id, client_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
    {
        return (
            StatusCode::OK,
            Json(json!({"ok": true, "duplicate": true, "message_id": existing})),
        )
            .into_response();
    }

    if let Some(reply_id) = reply_to {
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

    if db
        .execute(
            "INSERT INTO messages(
               conversation_id,sender_user_id,message,is_read,delivered_at,read_at,created_at,
               reply_to_message_id,client_message_id,attachment_kind,attachment_mime,attachment_size,attachment_path
             ) VALUES(?1,?2,?3,0,0,0,?4,?5,?6,?7,?8,?9,'')",
            params![
                conversation_id,user_id,caption,now,reply_to,client_id,kind,mime,bytes.len() as i64
            ],
        )
        .is_err()
    {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "message_store_failed");
    }

    let message_id = db.last_insert_rowid();
    let root = media_root();
    if std::fs::create_dir_all(&root).is_err() {
        let _ = db.execute("DELETE FROM messages WHERE id=?1", params![message_id]);
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "media_directory_failed");
    }

    let Some(extension) = extension_for_mime(&mime) else {
        let _ = db.execute("DELETE FROM messages WHERE id=?1", params![message_id]);
        return json_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
    };
    let file_name = format!("{}.{}", message_id, extension);
    let file_path = root.join(&file_name);
    if std::fs::write(&file_path, &bytes).is_err() {
        let _ = db.execute("DELETE FROM messages WHERE id=?1", params![message_id]);
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "media_write_failed");
    }

    let relative_path = file_path.to_string_lossy().to_string();
    let _ = db.execute(
        "UPDATE messages SET attachment_path=?2 WHERE id=?1",
        params![message_id, relative_path],
    );
    let _ = db.execute(
        "UPDATE conversations SET updated_at=?2 WHERE id=?1",
        params![conversation_id, now],
    );

    (
        StatusCode::CREATED,
        Json(json!({"ok": true, "message_id": message_id})),
    )
        .into_response()
}

pub async fn api_chat_send_image(
    State(state): State<AppState>,
    Path(peer_id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> axum::response::Response {
    save_attachment(
        state,
        headers,
        peer_id,
        multipart,
        "image",
        "image",
        &["image/jpeg", "image/png", "image/webp"],
        MAX_IMAGE_BYTES,
    )
    .await
}

pub async fn api_chat_send_voice(
    State(state): State<AppState>,
    Path(peer_id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> axum::response::Response {
    save_attachment(
        state,
        headers,
        peer_id,
        multipart,
        "voice",
        "voice",
        &["audio/webm", "audio/ogg", "audio/mp4", "audio/mpeg"],
        MAX_VOICE_BYTES,
    )
    .await
}

pub async fn api_chat_media(
    State(state): State<AppState>,
    Path(message_id): Path<i64>,
    headers: HeaderMap,
) -> axum::response::Response {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let row = db
        .query_row(
            r#"SELECT
                m.attachment_path,m.attachment_mime,m.deleted_at,
                c.user1_id,c.user2_id
              FROM messages m
              JOIN conversations c ON c.id=m.conversation_id
              WHERE m.id=?1
              LIMIT 1"#,
            params![message_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .ok();

    let Some((path, mime, deleted_at, u1, u2)) = row else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if deleted_at > 0 || (user_id != u1 && user_id != u2) || path.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let canonical_root = match std::fs::canonicalize(media_root()) {
        Ok(v) => v,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let canonical_file = match std::fs::canonicalize(FsPath::new(&path)) {
        Ok(v) => v,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    if !canonical_file.starts_with(&canonical_root) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let bytes = match std::fs::read(canonical_file) {
        Ok(v) => v,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::empty()))
        .into_response()
}
