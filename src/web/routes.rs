use axum::{
    routing::{get, post},
    Router,
};
use super::handlers::*;
use crate::state::app_state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/app", get(app_root))
        .route("/app/", get(app_root))
        .route("/app/search", get(app_search))
        .route("/app/me", get(app_me))
.route("/app/resource/{id}", get(resource_profile))
        .route("/app/{ci}", get(app_continent))
        .route("/app/{ci}/{si}", get(app_country))
        .route("/app/{ci}/{si}/{zi}", get(app_city))
        .route("/app/{ci}/{si}/{zi}/cat/{k}", get(app_cat))
        .route("/app/{ci}/{si}/{zi}/cat/{k}/add", get(add_resource_page).post(add_resource))
        .route("/health", get(health))
        .route("/api/vote", post(api_vote))
        .route("/api/points", get(api_points))
        .route("/api/my", get(api_my))
        .route("/api/stats", get(api_stats))
        .route("/api/profile", get(api_profile_get).post(api_profile_set))
        .route("/api/open_count", get(api_open_count))
        .route("/webhook", post(webhook_handler))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .with_state(state)
}
