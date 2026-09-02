use super::super::handlers::{
    api_map_country_cities, app_city, app_continent, app_country, app_geo_city, app_geo_continent,
    app_geo_country, app_geo_professions, app_menu, app_root, app_search, home,
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
        .route("/app/map/continent/{continent_id}", get(app_geo_continent))
        .route("/app/map/country/{country_id}", get(app_geo_country))
        .route("/app/map/city/{city_id}", get(app_geo_city))
        .route(
            "/app/map/city/{city_id}/sector/{sector_key}",
            get(app_geo_professions),
        )
        .route(
            "/api/map/countries/{country_id}/cities",
            get(api_map_country_cities),
        )
        .route("/app/{ci}", get(app_continent))
        .route("/app/{ci}/{si}", get(app_country))
        .route("/app/{ci}/{si}/{zi}", get(app_city))
}
