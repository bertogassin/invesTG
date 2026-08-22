use std::env;

mod bot;
mod db;
mod state;
mod utils;
mod web;

use bot::handler::{cities_handler, help_handler, start_handler};
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
    let db = db::queries::init_db()
        .expect("Не удалось открыть базу данных");

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN не задан");

    let state = AppState::new(db, bot_token.clone());

    let app = web::routes::routes(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Не удалось открыть порт 3000");

    println!("ResursMap запущен на http://0.0.0.0:3000");

    let bot = Bot::new(bot_token);

    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(
            |bot: Bot, msg: Message, command: Command| async move {
                match command {
                    Command::Start => start_handler(bot, msg).await,
                    Command::Help => help_handler(bot, msg).await,
                    Command::Cities => cities_handler(bot, msg).await,
                }
            },
        );

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
