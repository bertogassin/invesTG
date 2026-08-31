use crate::bot::groups::welcome_chat_member;
use crate::bot::handler::{cities_handler, help_handler, register_city_handler, start_handler};
use crate::bot::security::{is_security_candidate, security_observer};
use crate::bot::translator::{is_translation_command, translation_handler};
use crate::db::pool::DbPool;
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

    #[command(description = "подключить городскую группу", hide)]
    Registercity(String),
}

pub fn spawn_dispatcher(bot_token: String, db_pool: DbPool) {
    tokio::spawn(async move {
        let bot = Bot::new(bot_token);
        let security_db_pool = db_pool.clone();
        let registration_db_pool = db_pool;

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
            .endpoint(move |bot: Bot, msg: Message, command: Command| {
                let db_pool = registration_db_pool.clone();

                async move {
                    match command {
                        Command::Start => start_handler(bot, msg).await,
                        Command::Help => help_handler(bot, msg).await,
                        Command::Cities => cities_handler(bot, msg).await,
                        Command::Registercity(arguments) => {
                            register_city_handler(bot, msg, arguments, db_pool).await
                        }
                    }
                }
            });

        let group_welcome_handler = Update::filter_chat_member().endpoint(welcome_chat_member);

        let handler = dptree::entry()
            .branch(command_handler)
            .branch(translation_handler_branch)
            .branch(group_welcome_handler)
            .branch(security_handler);

        println!("Telegram bot dispatcher started");

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });
}
