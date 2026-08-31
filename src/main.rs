use std::env;

mod db;
mod geography;
mod resource_publisher;
mod resource_screening;
mod state;
mod telegram_notify;
mod utils;
mod web;

use state::app_state::AppState;

#[cfg(feature = "telegram-bot")]
mod bot;

#[tokio::main]
async fn main() {
    drop(db::queries::init_db().expect("Не удалось инициализировать базу данных"));

    db::admin_v2::initialize().expect("Не удалось применить миграцию административной системы V2");

    db::geography_v2::initialize().expect("Не удалось применить миграцию Geography V2");

    db::admin_geography::initialize()
        .expect("Не удалось синхронизировать административную географию");

    let db_pool = db::pool::create_pool().expect("Не удалось создать SQLite connection pool");

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let admin_key = env::var("ADMIN_KEY").expect("ADMIN_KEY не задан");

    let state = AppState::new(db_pool.clone(), bot_token.clone(), admin_key);

    #[cfg(feature = "telegram-bot")]
    match bot_token {
        Some(token) => bot::runtime::spawn_dispatcher(token, db_pool),
        None => println!(
            "Telegram bot dispatcher skipped: TELEGRAM_BOT_TOKEN is not set (web-only mode)"
        ),
    }

    #[cfg(not(feature = "telegram-bot"))]
    if bot_token.is_some() {
        println!(
            "TELEGRAM_BOT_TOKEN is set but this binary was built without the telegram-bot feature"
        );
    }

    let app = web::routes::routes(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Не удалось открыть порт 3000");

    println!("ResursMap запущен на http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Ошибка HTTP сервера");
}
