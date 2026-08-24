use super::super::handlers::{
    add_resource, add_resource_page, api_favorite_status, api_favorite_toggle, api_report_resource,
    api_resource_vote, app_cat, edit_resource, edit_resource_page, my_resources, resource_profile,
};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/app/my-resources", get(my_resources))
        .route("/app/resource/{id}", get(resource_profile))
        .route(
            "/app/resource/{id}/edit",
            get(edit_resource_page).post(edit_resource),
        )
        .route("/app/{ci}/{si}/{zi}/cat/{k}", get(app_cat))
        .route(
            "/app/{ci}/{si}/{zi}/cat/{k}/add",
            get(add_resource_page).post(add_resource),
        )
        .route("/api/resource/{id}/vote", post(api_resource_vote))
        .route(
            "/api/resource/{id}/favorite",
            post(api_favorite_toggle).get(api_favorite_status),
        )
        .route("/api/resource/{id}/report", post(api_report_resource))
}
