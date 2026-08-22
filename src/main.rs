use axum::Router;
use rusqlite::Connection;
use std::env;
use tower_http::services::ServeDir;

mod bot;
mod db;
mod state;
mod utils;
mod web;

use state::app_state::AppState;

#[tokio::main]
async fn main() {
    // ------------------------------------------------------------
    // База данных
    // ------------------------------------------------------------

    let db = db::queries::init_db()
        .expect("Не удалось открыть базу данных");

    // ------------------------------------------------------------
    // Telegram Bot Token
    // ------------------------------------------------------------

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .unwrap_or_default();

    // ------------------------------------------------------------
    // Общее состояние приложения
    // ------------------------------------------------------------

    let state = AppState::new(db, bot_token);

    // ------------------------------------------------------------
    // Routes
    // ------------------------------------------------------------

    let app = web::routes::routes(state);;

    // ------------------------------------------------------------
    // HTTP server
    // ------------------------------------------------------------

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Не удалось открыть порт 3000");

    println!("ResursMap запущен на http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Ошибка HTTP сервера");
}
