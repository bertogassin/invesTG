mod favorites;
pub use favorites::*;

mod notifications;
pub use notifications::*;

mod contacts;
pub use contacts::*;

mod chat;
pub use chat::*;

mod profiles;
pub use profiles::*;

mod resources;
pub use resources::*;

mod admin;
pub use admin::*;

mod navigation;
pub use navigation::*;

mod auth;
pub use auth::{app_auth, app_auth_page};
use auth::{
    create_admin_session, is_admin_session, verify_telegram_init_data, verify_user_session,
};

use super::templates;
use crate::state::app_state::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use sha2::Sha256;

fn telegram_owner_user_id(client_id: &str) -> Option<i64> {
    client_id
        .strip_prefix("tg:")
        .and_then(|value| value.parse::<i64>().ok())
}

fn input_text_is_valid(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let length = value.chars().count();

    if length < min_chars || length > max_chars {
        return false;
    }

    // Запрещаем управляющие символы, которые не нужны
    // обычному пользовательскому тексту.
    //
    // Перевод строки / CR / TAB разрешаем:
    // они нужны описаниям, сообщениям и статусам.
    !value
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
}

fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

async fn rate_limit_retry_after(
    state: &AppState,
    user_id: i64,
    action: &str,
    max_requests: usize,
    window_seconds: i64,
) -> Option<u64> {
    let now = unix_now();
    let cutoff = now.saturating_sub(window_seconds);
    let key = format!("{}:{}", action, user_id);

    let mut limits = state.rate_limits.lock().await;
    let events = limits.entry(key).or_default();

    // Sliding window:
    // забываем только запросы, вышедшие за текущее окно.
    while let Some(timestamp) = events.front().copied() {
        if timestamp <= cutoff {
            events.pop_front();
        } else {
            break;
        }
    }

    if events.len() >= max_requests {
        let oldest = events.front().copied().unwrap_or(now);

        let retry_after = oldest
            .saturating_add(window_seconds)
            .saturating_sub(now)
            .max(1);

        return Some(retry_after as u64);
    }

    events.push_back(now);

    None
}

fn request_is_cross_site(headers: &HeaderMap) -> bool {
    // Современные браузеры прямо сообщают происхождение запроса.
    // Если они говорят cross-site — такой state-changing запрос
    // с cookie-сессией нам не нужен.
    if let Some(value) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        if value.eq_ignore_ascii_case("cross-site") {
            return true;
        }
    }

    // Дополнительная проверка Origin.
    //
    // Заголовок может отсутствовать у curl, некоторых WebView и
    // старых клиентов — отсутствие Origin само по себе НЕ блокируем.
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let origin = origin.trim_end_matches('/');

        if origin != "https://resursmap.de"
            && origin != "https://www.resursmap.de"
            && origin != "http://127.0.0.1:3000"
        {
            return true;
        }
    }

    false
}

fn csrf_rejected_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "ok": false,
            "error": "cross_site_request_rejected"
        })),
    )
        .into_response()
}

// ------------------------------------------------------------
// HTML-страницы
// ------------------------------------------------------------
// ============================================================
// ДОБАВЛЕНИЕ РЕСУРСА
// ============================================================

#[derive(Debug, Deserialize)]
pub struct AddResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
    #[serde(default)]
    pub init_data: String,
}

// ============================================================
// PUBLIC USER PROFILE
// ============================================================

// ============================================================
// ПРОФИЛЬ РЕСУРСА
// ============================================================
// ============================================================
// МОИ РЕСУРСЫ
// ============================================================

// ============================================================
// CONTACT REQUESTS
// ============================================================

// ============================================================
// TASK 7.22G-C — MESSAGES LIST
// ============================================================

// ============================================================
// TASK 7.22F-E — INTERNAL CHAT
// ============================================================

#[derive(Debug, Deserialize)]
pub struct ChatMessageForm {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct EditResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
}

pub async fn health() -> &'static str {
    "ok"
}

// ------------------------------------------------------------
// API-обработчики (реальная логика)
// ------------------------------------------------------------
#[derive(Debug, serde::Deserialize)]
pub struct ReportResourcePayload {
    pub reason: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ContactRequestPayload {
    pub public_id: String,
    pub message: String,
}

// ============================================================
// TASK 7.3B — MODERATION APPROVE / REJECT
// ============================================================

#[derive(serde::Deserialize)]
pub struct RejectResourceForm {
    pub reason: String,
}
