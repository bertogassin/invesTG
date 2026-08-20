use crate::db;
use crate::models::*;
use crate::utils;
use log::*;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Start the interactive flow")]
    Start,
    #[command(description = "Cancel current operation")]
    Cancel,
    #[command(description = "View voting statistics (admin only)")]
    Stats,
    #[command(description = "Show help message")]
    Help,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = msg.from.map(|u| u.id.0 as i64).unwrap_or(0);

    match cmd {
        Command::Start => {
            show_continents(&bot, msg, user_id).await?;
        }
        Command::Cancel => {
            let conn = db::get_db()?;
            let mut session = db::get_user_session(&conn, user_id)?;

            session.current_state = "start".to_string();
            session.current_continent_id = None;
            session.current_country_id = None;
            session.current_city_id = None;
            session.current_category_id = None;

            db::update_user_session(&conn, &session)?;

            bot.send_message(msg.chat.id, "❌ Operation cancelled. Use /start to begin again.")
                .await?;
        }
        Command::Stats => {
            if !utils::is_admin(user_id) {
                bot.send_message(msg.chat.id, "❌ You don't have permission to view statistics.")
                    .await?;
                return Ok(());
            }

            let conn = db::get_db()?;
            let vote_count = db::get_vote_count(&conn)?;

            let stats = format!("📊 <b>Voting Statistics</b>\n\nTotal votes: {}", vote_count);
            bot.send_message(msg.chat.id, stats)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Command::Help => {
            let help_text = Command::descriptions().to_string();
            bot.send_message(msg.chat.id, help_text).await?;
        }
    }

    Ok(())
}

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = q.from.id.0 as i64;
    let callback_data = q.data.clone().unwrap_or_default();

    info!("Callback from user {}: {}", user_id, callback_data);

    let conn = db::get_db()?;
    let mut session = db::get_user_session(&conn, user_id)?;

    if callback_data.starts_with("continent_") {
        let continent_id: i32 = callback_data
            .strip_prefix("continent_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        session.current_continent_id = Some(continent_id);
        session.current_state = "country".to_string();
        db::update_user_session(&conn, &session)?;

        let countries = db::get_countries_by_continent(&conn, continent_id)?;

        let keyboard = InlineKeyboardMarkup::new(
            countries
                .iter()
                .map(|c| {
                    vec![InlineKeyboardButton::callback(
                        c.name.clone(),
                        format!("country_{}", c.id),
                    )]
                })
                .collect::<Vec<_>>(),
        );

        if let Some(Message { id: message_id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, message_id, "🌍 Select a country:")
                .reply_markup(keyboard)
                .await?;
        }
    } else if callback_data.starts_with("country_") {
        let country_id: i32 = callback_data
            .strip_prefix("country_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        session.current_country_id = Some(country_id);
        session.current_state = "city".to_string();
        db::update_user_session(&conn, &session)?;

        let cities = db::get_cities_by_country(&conn, country_id)?;

        let keyboard = InlineKeyboardMarkup::new(
            cities
                .iter()
                .map(|c| {
                    vec![InlineKeyboardButton::callback(
                        c.name.clone(),
                        format!("city_{}", c.id),
                    )]
                })
                .collect::<Vec<_>>(),
        );

        if let Some(Message { id: message_id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, message_id, "🏙️ Select a city:")
                .reply_markup(keyboard)
                .await?;
        }
    } else if callback_data.starts_with("city_") {
        let city_id: i32 = callback_data
            .strip_prefix("city_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        session.current_city_id = Some(city_id);
        session.current_state = "category".to_string();
        db::update_user_session(&conn, &session)?;

        let categories = db::get_categories(&conn)?;

        let keyboard = InlineKeyboardMarkup::new(
            categories
                .iter()
                .map(|c| {
                    vec![InlineKeyboardButton::callback(
                        c.name.clone(),
                        format!("category_{}", c.id),
                    )]
                })
                .collect::<Vec<_>>(),
        );

        if let Some(Message { id: message_id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, message_id, "📂 Select a category:")
                .reply_markup(keyboard)
                .await?;
        }
    } else if callback_data.starts_with("category_") {
        let category_id: i32 = callback_data
            .strip_prefix("category_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        session.current_category_id = Some(category_id);
        session.current_state = "vote".to_string();
        db::update_user_session(&conn, &session)?;

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback("⭐", "vote_1"),
                InlineKeyboardButton::callback("⭐⭐", "vote_2"),
                InlineKeyboardButton::callback("⭐⭐⭐", "vote_3"),
            ],
            vec![
                InlineKeyboardButton::callback("⭐⭐⭐⭐", "vote_4"),
                InlineKeyboardButton::callback("⭐⭐⭐⭐⭐", "vote_5"),
            ],
        ]);

        if let Some(Message { id: message_id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, message_id, "⭐ Rate (1-5 stars):")
                .reply_markup(keyboard)
                .await?;
        }
    } else if callback_data.starts_with("vote_") {
        let rating: i32 = callback_data
            .strip_prefix("vote_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        if let (Some(city_id), Some(category_id)) = (session.current_city_id, session.current_category_id) {
            db::save_vote(&conn, user_id, city_id, category_id, rating)?;

            info!(
                "Vote saved: user={}, city={}, category={}, rating={}",
                user_id, city_id, category_id, rating
            );

            // Reset session
            session.current_state = "start".to_string();
            session.current_continent_id = None;
            session.current_country_id = None;
            session.current_city_id = None;
            session.current_category_id = None;
            db::update_user_session(&conn, &session)?;

            if let Some(Message { id: message_id, chat, .. }) = q.message {
                bot.edit_message_text(
                    chat.id,
                    message_id,
                    format!("✅ Vote saved! Rating: {}⭐", rating),
                )
                .await?;
            }
        }
    }

    bot.answer_callback_query(q.id).await?;

    Ok(())
}

pub async fn handle_message(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        if !text.starts_with('/') {
            bot.send_message(msg.chat.id, "👋 Hello! Use /start to begin or /help for commands.")
                .await?;
        }
    }

    Ok(())
}

async fn show_continents(
    bot: &Bot,
    msg: Message,
    user_id: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = db::get_db()?;
    let continents = db::get_continents(&conn)?;

    let mut session = db::get_user_session(&conn, user_id)?;
    session.current_state = "continent".to_string();
    db::update_user_session(&conn, &session)?;

    let keyboard = InlineKeyboardMarkup::new(
        continents
            .iter()
            .map(|c| {
                vec![InlineKeyboardButton::callback(
                    c.name.clone(),
                    format!("continent_{}", c.id),
                )]
            })
            .collect::<Vec<_>>(),
    );

    bot.send_message(msg.chat.id, "🌍 Welcome! Select a continent:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
