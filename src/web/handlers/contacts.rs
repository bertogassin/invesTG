use super::auth::verify_user_session;
use super::common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
};
use super::types::ContactRequestPayload;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

pub async fn contact_requests_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_contact_requests(vec![], false));
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let requests: Vec<crate::web::view_models::ContactRequestRow> = db
        .prepare(
            "SELECT
                cr.id,
                cr.sender_user_id,
                cr.message,
                cr.status,
                COALESCE(p.public_id, ''),
                COALESCE(p.username, ''),
                COALESCE(p.first_name, ''),
                cr.created_at,
                CASE
                    WHEN cr.status = 'pending' THEN 0
                    WHEN cr.status = 'accepted' THEN 1
                    ELSE 2
                END
             FROM contact_requests cr
             LEFT JOIN profiles p
               ON p.user_id = cr.sender_user_id
             WHERE cr.receiver_user_id = ?1
             ORDER BY
                CASE cr.status
                    WHEN 'pending' THEN 0
                    WHEN 'accepted' THEN 1
                    ELSE 2
                END,
                cr.updated_at DESC,
                cr.id DESC
             LIMIT 100",
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
                    row.get(8)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(db);

    Html(templates::render_contact_requests(requests, true))
}

pub async fn accept_contact_request(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
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

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "contact_decision", 30, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много действий с запросами. Попробуйте позже.",
        )
            .into_response();
    }

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

    let request: Option<i64> = db
        .query_row(
            "SELECT sender_user_id
             FROM contact_requests
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id],
            |row| row.get(0),
        )
        .ok();

    let sender_user_id = match request {
        Some(id) => id,

        None => {
            drop(db);

            return (StatusCode::NOT_FOUND, "Запрос не найден").into_response();
        }
    };

    let (user1_id, user2_id) = if sender_user_id < user_id {
        (sender_user_id, user_id)
    } else {
        (user_id, sender_user_id)
    };

    let transaction_result = (|| -> rusqlite::Result<usize> {
        let tx = db.unchecked_transaction()?;

        let changed = tx.execute(
            "UPDATE contact_requests
             SET status = 'accepted',
                 updated_at = strftime('%s','now')
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id],
        )?;

        if changed == 1 {
            tx.execute(
                "INSERT INTO conversations (
                    user1_id,
                    user2_id,
                    created_at,
                    updated_at
                 )
                 VALUES (
                    ?1,
                    ?2,
                    strftime('%s','now'),
                    strftime('%s','now')
                 )
                 ON CONFLICT(user1_id, user2_id)
                 DO UPDATE SET
                    updated_at = strftime('%s','now')",
                rusqlite::params![user1_id, user2_id],
            )?;
        }

        tx.commit()?;
        Ok(changed)
    })();

    let changed = match transaction_result {
        Ok(changed) => changed,
        Err(err) => {
            drop(db);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка принятия запроса: {}", err),
            )
                .into_response();
        }
    };

    if changed == 1 {
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
                'contact_accepted',
                'Запрос принят',
                'Ваш запрос на связь принят. Теперь можно начать общение в ResursMap.',
                0,
                strftime('%s','now')
             )",
            rusqlite::params![sender_user_id],
        );
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/contact-requests")],
    )
        .into_response()
}

pub async fn reject_contact_request(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
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

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "contact_decision", 30, 600).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много действий с запросами. Попробуйте позже.",
        )
            .into_response();
    }

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

    let request: Option<i64> = db
        .query_row(
            "SELECT sender_user_id
             FROM contact_requests
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id],
            |row| row.get(0),
        )
        .ok();

    let sender_user_id = match request {
        Some(id) => id,

        None => {
            drop(db);

            return (StatusCode::NOT_FOUND, "Запрос не найден").into_response();
        }
    };

    let changed = db
        .execute(
            "UPDATE contact_requests
             SET status = 'rejected',
                 updated_at = strftime('%s','now')
             WHERE id = ?1
               AND receiver_user_id = ?2
               AND status = 'pending'",
            rusqlite::params![id, user_id,],
        )
        .unwrap_or(0);

    if changed == 1 {
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
                'contact_rejected',
                'Запрос отклонён',
                'Пользователь отклонил запрос на связь.',
                0,
                strftime('%s','now')
             )",
            rusqlite::params![sender_user_id,],
        );
    }

    drop(db);

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/app/contact-requests")],
    )
        .into_response()
}

pub async fn api_contact_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ContactRequestPayload>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let sender_user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

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

    let public_id = payload.public_id.trim();
    let message = payload.message.trim();

    if public_id.is_empty()
        || public_id.len() > 64
        || !public_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_public_id"
            })),
        )
            .into_response();
    }

    if !input_text_is_valid(message, 2, 500) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_message"
            })),
        )
            .into_response();
    }

    let receiver_user_id: Option<i64> = {
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

        db.query_row(
            "SELECT p.user_id
             FROM profiles AS p
             JOIN users AS u
               ON u.id = p.user_id
              AND u.is_active = 1
             WHERE p.public_id = ?1
             LIMIT 1",
            rusqlite::params![public_id],
            |row| row.get(0),
        )
        .ok()
    };

    let receiver_user_id = match receiver_user_id {
        Some(id) if id > 0 => id,

        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "ok": false,
                    "error": "user_not_found"
                })),
            )
                .into_response();
        }
    };

    if sender_user_id == receiver_user_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "cannot_contact_self"
            })),
        )
            .into_response();
    }

    if let Some(retry_after) =
        rate_limit_retry_after(&state, sender_user_id, "contact_request", 6, 600).await
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

    let existing_status: Option<String> = db
        .query_row(
            "SELECT status
             FROM contact_requests
             WHERE sender_user_id = ?1
               AND receiver_user_id = ?2
             LIMIT 1",
            rusqlite::params![sender_user_id, receiver_user_id,],
            |row| row.get(0),
        )
        .ok();

    if let Some(status) = existing_status.as_deref() {
        if status == "pending" {
            drop(db);

            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "ok": false,
                    "error": "request_already_pending"
                })),
            )
                .into_response();
        }

        if status == "accepted" {
            drop(db);

            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "ok": false,
                    "error": "already_connected"
                })),
            )
                .into_response();
        }
    }

    let result = db.execute(
        "INSERT INTO contact_requests (
            sender_user_id,
            receiver_user_id,
            message,
            status,
            created_at,
            updated_at
         )
         VALUES (
            ?1,
            ?2,
            ?3,
            'pending',
            strftime('%s','now'),
            strftime('%s','now')
         )

         ON CONFLICT(sender_user_id, receiver_user_id)
         DO UPDATE SET
            message = excluded.message,
            status = 'pending',
            updated_at = strftime('%s','now')",
        rusqlite::params![sender_user_id, receiver_user_id, message,],
    );

    drop(db);

    match result {
        Ok(_) => Json(json!({
            "ok": true,
            "status": "pending"
        }))
        .into_response(),

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": "database_error",
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}
