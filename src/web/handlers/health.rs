use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn health(State(state): State<AppState>) -> Response {
    let connection = match state.db_pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "db_unavailable").into_response();
        }
    };

    match connection.query_row("SELECT 1", [], |_| Ok(())) {
        Ok(()) => "ok".into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "db_error").into_response(),
    }
}

pub async fn robots_txt() -> Response {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        "User-agent: *\n\
Disallow: /app/user/\n\
Disallow: /app/me\n\
Disallow: /app/chat\n\
Disallow: /app/messages\n\
Disallow: /login\n\
Disallow: /register\n",
    )
        .into_response()
}
