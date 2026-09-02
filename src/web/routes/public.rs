use super::super::handlers::{
    app_add, app_city, app_continent, app_country, app_menu, app_root, app_search, home,
};
use crate::state::app_state::AppState;
use axum::{routing::get, Router};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/app/menu", get(app_menu))
        .route("/app", get(app_root))
        .route("/app/", get(app_root))
        .route("/app/search", get(app_search))
        .route("/app/add", get(app_add))
        .route("/app/{ci}", get(app_continent))
        .route("/app/{ci}/{si}", get(app_country))
        .route("/app/{ci}/{si}/{zi}", get(app_city))
}
