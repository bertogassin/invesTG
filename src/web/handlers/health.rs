use crate::state::app_state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
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
