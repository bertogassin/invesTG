use axum::{
    extract::Path,
    response::Html,
};
use crate::web::templates::render_city;

pub async fn app_city(Path((ci, si, zi)): Path<(usize, usize, usize)>) -> Html<String> {
    Html(render_city(ci, si, zi))
}
