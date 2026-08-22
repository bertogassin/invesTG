use axum::response::Html;
use crate::web::templates::render_continents;

pub async fn app_root() -> Html<String> {
    Html(render_continents())
}
