use super::handlers::{
    accept_contact_request, add_resource, add_resource_page, admin_approve_resource,
    admin_bulk_resources, admin_close_report, admin_hide_reported_resource, admin_login,
    admin_login_page, admin_reject_reported_resource, admin_reject_resource, admin_reports,
    admin_resources, admin_toggle_active, admin_toggle_premium, admin_toggle_verified,
    api_contact_request, api_favorite_status, api_favorite_toggle, api_open_count, api_profile_get,
    api_profile_set, api_report_resource, api_resource_vote, app_auth, app_auth_page, app_cat,
    app_city, app_continent, app_country, app_me, app_root, app_search, chat_page,
    contact_requests_page, edit_resource, edit_resource_page, favorites_page, health, home,
    messages_page, my_resources, notifications_page, public_user_profile, reject_contact_request,
    resource_profile, send_chat_message,
};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/app", get(app_root))
        .route("/app/auth", get(app_auth_page).post(app_auth))
        .route("/app/", get(app_root))
        .route("/app/search", get(app_search))
        .route("/app/me", get(app_me))
        .route("/app/my-resources", get(my_resources))
        .route("/app/favorites", get(favorites_page))
        .route("/app/notifications", get(notifications_page))
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
        .route("/app/user/{public_id}", get(public_user_profile))
        .route("/app/resource/{id}", get(resource_profile))
        .route(
            "/app/resource/{id}/edit",
            get(edit_resource_page).post(edit_resource),
        )
        .route("/app/{ci}", get(app_continent))
        .route("/app/{ci}/{si}", get(app_country))
        .route("/app/{ci}/{si}/{zi}", get(app_city))
        .route("/app/{ci}/{si}/{zi}/cat/{k}", get(app_cat))
        .route(
            "/app/{ci}/{si}/{zi}/cat/{k}/add",
            get(add_resource_page).post(add_resource),
        )
        .route("/app/admin/login", get(admin_login_page).post(admin_login))
        .route("/app/admin/resources", get(admin_resources))
        .route("/app/admin/reports", get(admin_reports))
        .route("/app/admin/report/{id}/close", post(admin_close_report))
        .route(
            "/app/admin/report/{id}/hide-resource",
            post(admin_hide_reported_resource),
        )
        .route(
            "/app/admin/report/{id}/reject-resource",
            post(admin_reject_reported_resource),
        )
        .route("/app/admin/resources/bulk", post(admin_bulk_resources))
        .route(
            "/app/admin/resource/{id}/toggle-verified",
            post(admin_toggle_verified),
        )
        .route(
            "/app/admin/resource/{id}/approve",
            post(admin_approve_resource),
        )
        .route(
            "/app/admin/resource/{id}/reject",
            post(admin_reject_resource),
        )
        .route(
            "/app/admin/resource/{id}/toggle-premium",
            post(admin_toggle_premium),
        )
        .route(
            "/app/admin/resource/{id}/toggle-active",
            post(admin_toggle_active),
        )
        .route("/health", get(health))
        .route("/api/resource/{id}/vote", post(api_resource_vote))
        .route(
            "/api/resource/{id}/favorite",
            post(api_favorite_toggle).get(api_favorite_status),
        )
        .route("/api/resource/{id}/report", post(api_report_resource))
        .route("/api/contact/request", post(api_contact_request))
        .route("/api/profile", get(api_profile_get).post(api_profile_set))
        .route("/api/open_count", get(api_open_count))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .with_state(state)
}
