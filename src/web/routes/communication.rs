use super::super::handlers::{
    accept_contact_request, api_chat_delete, api_chat_edit, api_chat_messages, api_chat_send,
    api_start_direct_chat, chat_page, contact_requests_page, messages_page, reject_contact_request,
    send_chat_message,
};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/app/contact-requests", get(contact_requests_page))
        .route("/app/messages", get(messages_page))
        .route(
            "/app/contact-request/{id}/accept",
            post(accept_contact_request),
        )
        .route(
            "/app/contact-request/{id}/reject",
            post(reject_contact_request),
        )
        .route("/app/chat/{other_user_id}", get(chat_page))
        .route("/app/chat/{other_user_id}/send", post(send_chat_message))
        .route("/api/chat/{other_user_id}/messages", get(api_chat_messages))
        .route("/api/chat/{other_user_id}/send", post(api_chat_send))
        .route(
            "/api/chat/{other_user_id}/messages/{message_id}/edit",
            post(api_chat_edit),
        )
        .route(
            "/api/chat/{other_user_id}/messages/{message_id}/delete",
            post(api_chat_delete),
        )
        .route("/api/contact/request", post(api_start_direct_chat))
}
