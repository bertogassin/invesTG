use super::super::handlers::{
    api_open_count, api_profile_get, api_profile_set, app_auth, app_auth_page, app_me,
    email_auth_request, email_auth_verify, favorites_page, notifications_page, public_user_profile,
};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/app/auth", get(app_auth_page).post(app_auth))
        .route("/app/auth/email/request", post(email_auth_request))
        .route("/app/auth/email/verify", post(email_auth_verify))
        .route("/app/me", get(app_me))
        .route("/app/favorites", get(favorites_page))
        .route("/app/notifications", get(notifications_page))
        .route("/app/user/{public_id}", get(public_user_profile))
        .route("/api/profile", get(api_profile_get).post(api_profile_set))
        .route("/api/open_count", get(api_open_count))
}
