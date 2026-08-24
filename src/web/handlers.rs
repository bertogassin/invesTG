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

mod common;
use common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
    telegram_owner_user_id, unix_now,
};

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
