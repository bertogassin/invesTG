use axum::{
    extract::Path,
    response::Html,
};
use crate::web::templates::render_country;

pub async fn app_country(Path((ci, si)): Path<(usize, usize)>) -> Html<String> {
    Html(render_country(ci, si))
}
