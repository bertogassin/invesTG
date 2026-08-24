use super::super::handlers::health;
use crate::state::app_state::AppState;
use axum::{routing::get, Router};

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
