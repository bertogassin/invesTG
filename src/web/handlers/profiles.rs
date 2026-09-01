use super::auth::{current_session_public_id, verify_authenticated_user, verify_user_session};
use super::common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
    unix_now,
};
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

type PublicProfileRow = (i64, String, String, String, String, i64, String, i64);
type MeProfileRow = (
    String,
    String,
    String,
    i64,
    String,
    i64,
    String,
    i64,
    i64,
    i64,
);

fn noindex_html(html: String) -> Response {
    (
        [(
            header::HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("noindex, nofollow"),
        )],
        Html(html),
    )
        .into_response()
}

fn parse_home_city_value(raw: &str) -> (Option<i64>, Option<i64>, Option<i64>) {
    if raw.trim().is_empty() {
        return (None, None, None);
    }

    let mut parts = raw.split(':');
    let Some(ci) = parts.next().and_then(|value| value.parse::<i64>().ok()) else {
        return (None, None, None);
    };
    let Some(si) = parts.next().and_then(|value| value.parse::<i64>().ok()) else {
        return (None, None, None);
    };
    let Some(zi) = parts.next().and_then(|value| value.parse::<i64>().ok()) else {
        return (None, None, None);
    };

    if parts.next().is_some() || ci < 0 || si < 0 || zi < 0 {
        return (None, None, None);
    }

    let world_data = crate::geography::world();
    let Some((_, countries)) = world_data.iter().nth(ci as usize) else {
        return (None, None, None);
    };
    let Some((_, cities)) = countries.iter().nth(si as usize) else {
        return (None, None, None);
    };
    if (zi as usize) >= cities.len() {
        return (None, None, None);
    }

    (Some(ci), Some(si), Some(zi))
}

pub async fn app_me(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,

        None => {
            return Html(templates::render_me(templates::RenderMeParams {
                authenticated: false,
                user_id: 0,
                username: "",
                first_name: "",
                last_name: "",
                resources_count: 0,
                approved_count: 0,
                pending_count: 0,
                rejected_count: 0,
                favorites_count: 0,
                unread_notifications_count: 0,
                pending_contact_requests_count: 0,
                unread_messages_count: 0,
                moderator_level: 0,
                intent_text: "",
                intent_until: 0,
                category: "",
                home_continent_index: -1,
                home_country_index: -1,
                home_city_index: -1,
                user_sessions: vec![],
            }));
        }
    };

    let user_id = user.user_id;
    let client_id = user.client_id;

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let profile: Option<MeProfileRow> = db
        .query_row(
            "SELECT
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until,
                category,
                COALESCE(home_continent_index, -1),
                COALESCE(home_country_index, -1),
                COALESCE(home_city_index, -1)
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
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                ))
            },
        )
        .ok();

    let (
        username,
        first_name,
        last_name,
        _open_contact,
        intent_text,
        intent_until,
        category,
        home_continent_index,
        home_country_index,
        home_city_index,
    ) = profile.unwrap_or_else(|| {
        (
            String::new(),
            String::new(),
            String::new(),
            0,
            String::new(),
            0,
            String::new(),
            -1,
            -1,
            -1,
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

    let moderator_level: i64 = db
        .query_row(
            "SELECT COALESCE(MAX(role_level), 0)
             FROM admin_assignments
             WHERE user_id = ?1
               AND status = 'active'
               AND valid_from <= strftime('%s','now')
               AND (
                    valid_until IS NULL
                    OR valid_until > strftime('%s','now')
               )",
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

    let current_session_public_id = current_session_public_id(&headers).unwrap_or_default();
    let user_sessions = load_user_sessions(&db, user_id, &current_session_public_id, unix_now());

    drop(db);

    Html(templates::render_me(templates::RenderMeParams {
        authenticated: true,
        user_id,
        username: &username,
        first_name: &first_name,
        last_name: &last_name,
        resources_count,
        approved_count,
        pending_count,
        rejected_count,
        favorites_count,
        unread_notifications_count,
        pending_contact_requests_count,
        unread_messages_count,
        moderator_level,
        intent_text: &intent_text,
        intent_until,
        category: &category,
        home_continent_index,
        home_country_index,
        home_city_index,
        user_sessions,
    }))
}

pub async fn api_attention_count(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return Json(json!({
                "count": 0,
                "messages": 0,
                "notifications": 0,
                "contacts": 0,
            }))
            .into_response();
        }
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "database_unavailable" })),
            )
                .into_response();
        }
    };

    let counts = query_attention_counts(&db, user_id);

    Json(json!({
        "count": counts.0,
        "messages": counts.1,
        "notifications": counts.2,
        "contacts": counts.3,
    }))
    .into_response()
}

fn query_attention_counts(db: &rusqlite::Connection, user_id: i64) -> (i64, i64, i64, i64) {
    let notifications: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM user_notifications
             WHERE user_id = ?1
               AND is_read = 0",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let contacts: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM contact_requests
             WHERE receiver_user_id = ?1
               AND status = 'pending'",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let messages: i64 = db
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

    (
        messages + notifications + contacts,
        messages,
        notifications,
        contacts,
    )
}

fn load_user_sessions(
    db: &rusqlite::Connection,
    user_id: i64,
    current_public_id: &str,
    now: i64,
) -> Vec<crate::web::view_models::UserSessionRow> {
    let mut stmt = match db.prepare(
        "SELECT session_public_id, ip_address, user_agent, created_at, last_seen_at
         FROM user_sessions
         WHERE user_id = ?1
           AND revoked_at IS NULL
           AND expires_at > ?2
         ORDER BY last_seen_at DESC
         LIMIT 8",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Vec::new(),
    };

    stmt.query_map(rusqlite::params![user_id, now], |row| {
        let session_public_id: String = row.get(0)?;

        Ok(crate::web::view_models::UserSessionRow {
            session_public_id: session_public_id.clone(),
            ip_address: row.get(1)?,
            user_agent: row.get(2)?,
            _created_at: row.get(3)?,
            _last_seen_at: row.get(4)?,
            is_current: !current_public_id.is_empty() && session_public_id == current_public_id,
        })
    })
    .ok()
    .map(|rows| rows.filter_map(Result::ok).collect())
    .unwrap_or_default()
}

pub async fn public_user_profile(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let public_id = public_id.trim();

    if public_id.is_empty()
        || public_id.len() > 64
        || !public_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return noindex_html(templates::render_public_user_not_found());
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return noindex_html("<h1>503</h1><p>База данных временно недоступна.</p>".to_string());
        }
    };

    let profile: Option<PublicProfileRow> = db
        .query_row(
            "SELECT
                p.user_id,
                p.client_id,
                p.username,
                p.first_name,
                p.last_name,
                p.open_contact,
                p.intent_text,
                p.intent_until
             FROM profiles AS p
             JOIN users AS u
               ON u.id = p.user_id
              AND u.is_active = 1
             WHERE p.public_id = ?1
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
                    row.get(7)?,
                ))
            },
        )
        .ok();

    let (
        profile_user_id,
        client_id,
        username,
        first_name,
        last_name,
        _open_contact,
        intent_text,
        intent_until,
    ) = match profile {
        Some(profile) => profile,

        None => {
            drop(db);

            return noindex_html(templates::render_public_user_not_found());
        }
    };

    let resources: Vec<crate::web::view_models::PublicProfileResourceRow> = db
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

    noindex_html(templates::render_public_user_profile(
        templates::RenderPublicUserProfileParams {
            public_id,
            username: &username,
            first_name: &first_name,
            last_name: &last_name,
            intent_text: &visible_intent,
            chat_user_id,
            resources,
        },
    ))
}

pub async fn api_profile_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,

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

    let client_id = user.client_id;

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

    let profile: Option<(String, String, String, i64, String, i64, String)> = db
        .query_row(
            "SELECT
                username,
                first_name,
                last_name,
                open_contact,
                intent_text,
                intent_until,
                category
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
                    row.get(6)?,
                ))
            },
        )
        .ok();

    drop(db);

    let (username, first_name, last_name, _open_contact, intent_text, intent_until, _category) =
        profile.unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                0,
                String::new(),
                0,
                String::new(),
            )
        });

    Json(json!({
        "ok": true,
        "username": username,
        "first_name": first_name,
        "last_name": last_name,
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

    let user = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,

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

    let user_id = user.user_id;
    let client_id = user.client_id;

    let open_contact = payload
        .get("open_contact")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let intent_text = payload
        .get("intent_text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    let raw_category = payload
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    let home_city = payload
        .get("home_city")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    let (home_continent_index, home_country_index, home_city_index) =
        parse_home_city_value(home_city);

    if !input_text_is_valid(raw_category, 0, 80) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "invalid_category"
            })),
        )
            .into_response();
    }

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

    let intent_until = if intent_text.is_empty() || duration_days == 0 {
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

    let home_city_id: Option<i64> =
        match (home_continent_index, home_country_index, home_city_index) {
            (Some(ci), Some(si), Some(zi)) => db
                .query_row(
                    "SELECT city.id
                 FROM geo_cities AS city
                 WHERE city.legacy_continent_index = ?1
                   AND city.legacy_country_index = ?2
                   AND city.legacy_city_index = ?3
                   AND city.is_active = 1
                 LIMIT 1",
                    rusqlite::params![ci, si, zi],
                    |row| row.get(0),
                )
                .ok(),
            _ => None,
        };

    let category = match crate::db::professions::synchronize_profile(&db, user_id, raw_category) {
        Ok(category) => category,
        Err(_) => raw_category.to_string(),
    };

    let result = db.execute(
        "INSERT INTO profiles (
            client_id,
            user_id,
            open_contact,
            intent_text,
            intent_until,
            category,
            home_continent_index,
            home_country_index,
            home_city_index,
            home_city_id,
            updated_at
         )
         VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            ?6,
            ?7,
            ?8,
            ?9,
            ?10,
            strftime('%s','now')
         )

         ON CONFLICT(client_id)
         DO UPDATE SET
            user_id = excluded.user_id,
            open_contact = excluded.open_contact,
            intent_text = excluded.intent_text,
            intent_until = excluded.intent_until,
            category = excluded.category,
            home_continent_index = excluded.home_continent_index,
            home_country_index = excluded.home_country_index,
            home_city_index = excluded.home_city_index,
            home_city_id = excluded.home_city_id,
            updated_at = strftime('%s','now')",
        rusqlite::params![
            &client_id,
            user_id,
            if open_contact { 1 } else { 0 },
            intent_text,
            intent_until,
            category,
            home_continent_index,
            home_country_index,
            home_city_index,
            home_city_id,
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
