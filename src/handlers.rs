use crate::db;
use crate::models::*;
use crate::utils;
use log::*;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

#[derive(teloxide::utils::command::BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Start the interactive flow")]
    Start,
    #[command(description = "Cancel current operation")]
    Cancel,
    #[command(description = "View voting statistics (admin only)")]
    Stats,
    #[command(description = "Show this help message")]
    Help,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
) -> ResponseResult<()> {
    let user_id = msg.from.unwrap().id.0 as i64;

    match cmd {
        Command::Start => {
            show_continents(&bot, msg, user_id).await?;
        }
        Command::Cancel => {
            let conn = db::get_db().map_err(|e| {
                error!("DB error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

            let mut session = db::get_user_session(&conn, user_id).map_err(|e| {
                error!("Session error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

            session.current_state = "start".to_string();
            session.current_continent_id = None;
            session.current_country_id = None;
            session.current_city_id = None;
            session.current_category_id = None;

            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

            bot.send_message(msg.chat.id, "❌ Operation cancelled. Use /start to begin again.")
                .await?;
        }
        Command::Stats => {
            if !utils::is_admin(user_id) {
                bot.send_message(msg.chat.id, "❌ You don't have permission to view statistics.")
                    .await?;
                return Ok(());
            }

            let conn = db::get_db().map_err(|e| {
                error!("DB error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

            let vote_count = db::get_vote_count(&conn).map_err(|e| {
                error!("Vote count error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

            let stats = format!("📊 **Voting Statistics**\n\nTotal votes: {}", vote_count);
            bot.send_message(msg.chat.id, stats)
                .parse_mode(ParseMode::Markdown)
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
) -> ResponseResult<()> {
    let user_id = q.from.id.0 as i64;
    let callback_data = q.data.unwrap_or_default();

    info!("Callback from user {}: {}", user_id, callback_data);

    let conn = db::get_db().map_err(|e| {
        error!("DB error: {}", e);
        ResponseError::Other(Box::new(e))
    })?;

    let mut session = db::get_user_session(&conn, user_id).map_err(|e| {
        error!("Session error: {}", e);
        ResponseError::Other(Box::new(e))
    })?;

    if callback_data.starts_with("continent_") {
        let continent_id: i32 = callback_data
            .strip_prefix("continent_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        session.current_continent_id = Some(continent_id);
        session.current_state = "country".to_string();
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

        let countries = db::get_countries_by_continent(&conn, continent_id).map_err(|e| {
            error!("Countries fetch error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

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

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id) {
            let chat_id = q.from.id;
            bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, "🌍 Select a country:")
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
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

        let cities = db::get_cities_by_country(&conn, country_id).map_err(|e| {
            error!("Cities fetch error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

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

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id) {
            let chat_id = q.from.id;
            bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, "🏙️ Select a city:")
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
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

        let categories = db::get_categories(&conn).map_err(|e| {
            error!("Categories fetch error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

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

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id) {
            let chat_id = q.from.id;
            bot.edit_message_text(
                ChatId(chat_id.0 as i64),
                msg_id,
                "📂 Select a category:",
            )
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
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            ResponseError::Other(Box::new(e))
        })?;

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

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id) {
            let chat_id = q.from.id;
            bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, "⭐ Rate (1-5 stars):")
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
            db::save_vote(&conn, user_id, city_id, category_id, rating).map_err(|e| {
                error!("Vote save error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

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
            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                ResponseError::Other(Box::new(e))
            })?;

            if let Some(msg_id) = q.message.as_ref().map(|m| m.id) {
                let chat_id = q.from.id;
                bot.edit_message_text(
                    ChatId(chat_id.0 as i64),
                    msg_id,
                    format!("✅ Vote saved! Rating: {}⭐", rating),
                )
                .await?;
            }
        }
    }

    bot.answer_callback_query(&q.id).await?;

    Ok(())
}

pub async fn handle_message(
    bot: Bot,
    msg: Message,
) -> ResponseResult<()> {
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
) -> ResponseResult<()> {
    let conn = db::get_db().map_err(|e| {
        error!("DB error: {}", e);
        ResponseError::Other(Box::new(e))
    })?;

    let continents = db::get_continents(&conn).map_err(|e| {
        error!("Continents fetch error: {}", e);
        ResponseError::Other(Box::new(e))
    })?;

    let mut session = db::get_user_session(&conn, user_id).map_err(|e| {
        error!("Session error: {}", e);
        ResponseError::Other(Box::new(e))
    })?;

    session.current_state = "continent".to_string();
    db::update_user_session(&conn, &session).map_err(|e| {
        error!("Session update error: {}", e);
        ResponseError::Other(Box::new(e))
    })?;

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
