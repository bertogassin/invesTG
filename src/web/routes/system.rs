use super::super::handlers::{health, robots_txt, stripe_promotion_webhook};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/robots.txt", get(robots_txt))
        .route("/api/stripe/webhook", post(stripe_promotion_webhook))
}
