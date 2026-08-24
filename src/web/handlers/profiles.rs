use super::*;

pub async fn app_me(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user_id = match verify_user_session(&state, &headers) {
        Some(id) => id,

        None => {
            return Html(templates::render_me(
                false, 0, "", "", "", 0, 0, 0, 0, 0, 0, 0, 0, false, "", 0,
            ));
        }
    };

    let client_id = format!("tg:{}", user_id);

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let profile: Option<(String, String, String, i64, String, i64)> = db
        .query_row(
            "SELECT
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until
             FROM profiles
             WHERE client_id = ?1",
            rusqlite::params![&client_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .ok();

    let (username, first_name, last_name, open_contact, intent_text, intent_until) = profile
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                0,
                String::new(),
                0,
            )
        });

    let resources_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let approved_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1
               AND moderation_status = 'approved'",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pending_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1
               AND moderation_status = 'pending'",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let rejected_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM resources
             WHERE client_id = ?1
               AND moderation_status = 'rejected'",
            rusqlite::params![&client_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let favorites_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM favorites
             WHERE user_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let unread_notifications_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM user_notifications
             WHERE user_id = ?1
               AND is_read = 0",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pending_contact_requests_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM contact_requests
             WHERE receiver_user_id = ?1
               AND status = 'pending'",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let unread_messages_count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM messages m
             JOIN conversations c
               ON c.id = m.conversation_id
             WHERE (c.user1_id = ?1 OR c.user2_id = ?1)
               AND m.sender_user_id <> ?1
               AND m.is_read = 0",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    drop(db);

    Html(templates::render_me(
        true,
        user_id,
        &username,
        &first_name,
        &last_name,
        resources_count,
        approved_count,
        pending_count,
        rejected_count,
        favorites_count,
        unread_notifications_count,
        pending_contact_requests_count,
        unread_messages_count,
        open_contact == 1,
        &intent_text,
        intent_until,
    ))
}

pub async fn public_user_profile(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let public_id = public_id.trim();

    if public_id.is_empty()
        || public_id.len() > 64
        || !public_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Html(templates::render_public_user_not_found());
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let profile: Option<(String, String, String, String, i64, String, i64)> = db
        .query_row(
            "SELECT
                client_id,
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until
             FROM profiles
             WHERE public_id = ?1
             LIMIT 1",
            rusqlite::params![public_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .ok();

    let (client_id, username, first_name, last_name, open_contact, intent_text, intent_until) =
        match profile {
            Some(profile) => profile,

            None => {
                drop(db);

                return Html(templates::render_public_user_not_found());
            }
        };

    // Старые web:-профили публичными не делаем.
    if !client_id.starts_with("tg:") {
        drop(db);

        return Html(templates::render_public_user_not_found());
    }

    let resources: Vec<(i64, String, String, String, f64, i64, i64, i64)> = db
        .prepare(
            "SELECT
                id,
                title,
                category,
                description,
                rating,
                votes,
                is_verified,
                is_premium
             FROM resources
             WHERE client_id = ?1
               AND is_active = 1
               AND moderation_status = 'approved'
             ORDER BY
                is_premium DESC,
                is_verified DESC,
                rating DESC,
                votes DESC,
                id DESC",
        )
        .and_then(|mut stmt| {
            stmt.query_map(rusqlite::params![&client_id], |row| {
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

    // Определяем, кто сейчас смотрит публичный профиль.
    let viewer_user_id = verify_user_session(&state, &headers);

    // Если между текущим пользователем и владельцем профиля
    // уже существует conversation, передаём ID владельца
    // в шаблон, чтобы вместо повторного запроса показать чат.
    let chat_user_id: Option<i64> = viewer_user_id.and_then(|viewer_id| {
        let profile_user_id = client_id
            .strip_prefix("tg:")
            .and_then(|value| value.parse::<i64>().ok())?;

        if viewer_id <= 0 || profile_user_id <= 0 || viewer_id == profile_user_id {
            return None;
        }

        let (user1_id, user2_id) = if viewer_id < profile_user_id {
            (viewer_id, profile_user_id)
        } else {
            (profile_user_id, viewer_id)
        };

        let exists: Option<i64> = db
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

        exists.map(|_| profile_user_id)
    });

    drop(db);

    // Просроченный статус публично не показываем.
    let visible_intent = if intent_until > 0 && intent_until < unix_now() {
        String::new()
    } else {
        intent_text
    };

    Html(templates::render_public_user_profile(
        public_id,
        &username,
        &first_name,
        &last_name,
        open_contact == 1,
        &visible_intent,
        chat_user_id,
        resources,
    ))
}

pub async fn api_profile_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
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

    let client_id = format!("tg:{}", user_id);

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

    let profile: Option<(String, String, String, i64, String, i64)> = db
        .query_row(
            "SELECT
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until
             FROM profiles
             WHERE client_id = ?1",
            rusqlite::params![&client_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .ok();

    drop(db);

    let (username, first_name, last_name, open_contact, intent_text, intent_until) = profile
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                0,
                String::new(),
                0,
            )
        });

    Json(json!({
        "ok": true,
        "username": username,
        "first_name": first_name,
        "last_name": last_name,
        "open_contact": open_contact == 1,
        "intent_text": intent_text,
        "intent_until": intent_until
    }))
    .into_response()
}

pub async fn api_profile_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if request_is_cross_site(&headers) {
        return csrf_rejected_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
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

    let client_id = format!("tg:{}", user_id);

    let open_contact = payload
        .get("open_contact")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let intent_text = payload
        .get("intent_text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if !input_text_is_valid(intent_text, 0, 300) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_intent"
            })),
        )
            .into_response();
    }

    let duration_days = payload
        .get("duration_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let allowed_days = [0_i64, 1, 3, 7, 30];

    if !allowed_days.contains(&duration_days) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_duration"
            })),
        )
            .into_response();
    }

    let intent_until = if intent_text.is_empty() {
        0
    } else if duration_days == 0 {
        0
    } else {
        unix_now() + duration_days * 86_400
    };

    if let Some(retry_after) =
        rate_limit_retry_after(&state, user_id, "profile_update", 30, 600).await
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

    let result = db.execute(
        "INSERT INTO profiles (
            client_id,
            open_contact,
            intent_text,
            intent_until,
            updated_at
         )
         VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            strftime('%s','now')
         )

         ON CONFLICT(client_id)
         DO UPDATE SET
            open_contact = excluded.open_contact,
            intent_text = excluded.intent_text,
            intent_until = excluded.intent_until,
            updated_at = strftime('%s','now')",
        rusqlite::params![
            &client_id,
            if open_contact { 1 } else { 0 },
            intent_text,
            intent_until,
        ],
    );

    drop(db);

    match result {
        Ok(_) => Json(json!({
            "ok": true,
            "open_contact": open_contact,
            "intent_text": intent_text,
            "intent_until": intent_until
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

pub async fn api_open_count(State(state): State<AppState>) -> Json<serde_json::Value> {
    let count: i64 = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db
            .query_row(
                "SELECT COUNT(*)
                 FROM profiles
                 WHERE open_contact = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0),
        Err(_) => 0,
    };

    Json(json!({
        "count": count
    }))
}
