use super::super::handlers::{
    accept_contact_request, api_chat_block, api_chat_block_status, api_chat_conversations,
    api_chat_delete, api_chat_edit, api_chat_media, api_chat_messages, api_chat_peer,
    api_chat_react, api_chat_realtime, api_chat_send, api_chat_send_image, api_chat_send_voice,
    api_chat_unblock, api_start_direct_chat, chat_page, contact_requests_page, messages_page,
    reject_contact_request,
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
        .route("/api/chat/conversations", get(api_chat_conversations))
        .route("/api/chat/{other_user_id}/messages", get(api_chat_messages))
        .route("/api/chat/{other_user_id}/peer", get(api_chat_peer))
        .route("/api/chat/realtime", get(api_chat_realtime))
        .route("/api/chat/{other_user_id}/send", post(api_chat_send))
        .route(
            "/api/chat/{other_user_id}/send-image",
            post(api_chat_send_image),
        )
        .route(
            "/api/chat/{other_user_id}/send-voice",
            post(api_chat_send_voice),
        )
        .route("/api/chat/media/{message_id}", get(api_chat_media))
        .route(
            "/api/chat/{other_user_id}/block",
            get(api_chat_block_status).post(api_chat_block),
        )
        .route("/api/chat/{other_user_id}/unblock", post(api_chat_unblock))
        .route(
            "/api/chat/{other_user_id}/messages/{message_id}/edit",
            post(api_chat_edit),
        )
        .route(
            "/api/chat/{other_user_id}/messages/{message_id}/delete",
            post(api_chat_delete),
        )
        .route(
            "/api/chat/{other_user_id}/messages/{message_id}/react",
            post(api_chat_react),
        )
        .route("/api/contact/request", post(api_start_direct_chat))
}
