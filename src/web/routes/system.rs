use super::super::handlers::health;
use super::super::handlers::stripe_promotion_webhook;
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/api/stripe/webhook", post(stripe_promotion_webhook))
}
