use std::env;

mod bot;
mod db;
mod geography;
mod state;
mod utils;
mod web;

use bot::groups::welcome_chat_member;
use bot::handler::{cities_handler, help_handler, start_handler};
use bot::security::{is_security_candidate, security_observer};
use bot::translator::{is_translation_command, translation_handler};
use state::app_state::AppState;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды:")]
enum Command {
    #[command(description = "открыть карту")]
    Start,
    #[command(description = "помощь")]
    Help,
    #[command(description = "список городов")]
    Cities,
}

#[tokio::main]
async fn main() {
    // Инициализируем/проверяем схему SQLite и сразу закрываем
    // одноразовое startup-соединение.
    drop(db::queries::init_db().expect("Не удалось инициализировать базу данных"));

    db::admin_v2::initialize().expect("Не удалось применить миграцию административной системы V2");

    // Единственный постоянный production-доступ к SQLite — connection pool.
    let db_pool = db::pool::create_pool().expect("Не удалось создать SQLite connection pool");

    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN не задан");

    let admin_key = env::var("ADMIN_KEY").expect("ADMIN_KEY не задан");

    let admin_telegram_id: i64 = env::var("ADMIN_TELEGRAM_ID")
        .expect("ADMIN_TELEGRAM_ID не задан")
        .parse()
        .expect("ADMIN_TELEGRAM_ID должен быть числом");

    let security_db_pool = db_pool.clone();

    let state = AppState::new(db_pool, bot_token.clone(), admin_key, admin_telegram_id);

    let app = web::routes::routes(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Не удалось открыть порт 3000");

    println!("ResursMap запущен на http://0.0.0.0:3000");

    let bot = Bot::new(bot_token);

    let translation_handler_branch = Update::filter_message()
        .filter(|msg: Message| is_translation_command(&msg))
        .endpoint(translation_handler);

    let security_handler = Update::filter_message()
        .filter(|msg: Message| is_security_candidate(&msg))
        .endpoint(move |bot: Bot, msg: Message| {
            let db_pool = security_db_pool.clone();

            async move { security_observer(bot, msg, db_pool).await }
        });

    let command_handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(|bot: Bot, msg: Message, command: Command| async move {
            match command {
                Command::Start => start_handler(bot, msg).await,
                Command::Help => help_handler(bot, msg).await,
                Command::Cities => cities_handler(bot, msg).await,
            }
        });

    let group_welcome_handler = Update::filter_chat_member().endpoint(welcome_chat_member);

    let handler = dptree::entry()
        .branch(command_handler)
        .branch(translation_handler_branch)
        .branch(group_welcome_handler)
        .branch(security_handler);

    tokio::spawn(async move {
        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });

    axum::serve(listener, app)
        .await
        .expect("Ошибка HTTP сервера");
}
