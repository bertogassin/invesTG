use axum::{
    extract::Path,
    response::Html,
};
use crate::web::templates::render_continent;

pub async fn app_continent(Path(ci): Path<usize>) -> Html<String> {
    Html(render_continent(ci))
}
