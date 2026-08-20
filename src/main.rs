mod db;
mod handlers;
mod models;
mod utils;

use anyhow::Result;
use dotenv::dotenv;
use log::*;
use std::env;
use teloxide::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment and logging
    dotenv().ok();
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting invesTG Telegram Bot");

    // Validate required environment variables
    let bot_token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found in .env");
    let _admin_ids = utils::parse_admin_ids();

    // Initialize database
    db::init_db().expect("Failed to initialize database");
    info!("Database initialized successfully");

    // Create bot
    let bot = Bot::new(bot_token);
    info!("Bot created successfully");

    // Create dispatcher with handlers
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<handlers::Command>()
                .endpoint(handlers::handle_command),
        )
        .branch(
            Update::filter_callback_query().endpoint(handlers::handle_callback),
        )
        .branch(Update::filter_message().endpoint(handlers::handle_message));

    // Start dispatcher
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("Bot stopped");
    Ok(())
}
