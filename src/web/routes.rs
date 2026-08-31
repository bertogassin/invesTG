mod account;
mod admin;
mod communication;
mod public;
mod resources;
mod system;

use crate::state::app_state::AppState;
use axum::{extract::DefaultBodyLimit, Router};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .merge(public::routes())
        .merge(account::routes())
        .merge(communication::routes())
        .merge(resources::routes())
        .merge(admin::routes())
        .merge(system::routes())
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .with_state(state)
}
