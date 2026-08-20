use crate::db;
use crate::utils;
use anyhow::Result;
use log::*;
use teloxide::utils::command::BotCommands;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

#[derive(teloxide::utils::command::BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum Command {
    #[command(description = "Начать работу")]
    Start,
    #[command(description = "Отменить текущую операцию")]
    Cancel,
    #[command(description = "Статистика голосований (только для администраторов)")]
    Stats,
    #[command(description = "Показать это сообщение")]
    Help,
}

pub async fn handle_command(bot: Bot, msg: Message, cmd: Command) -> Result<()> {
    let user_id = msg.from.as_ref().unwrap().id.0 as i64;

    match cmd {
        Command::Start => {
            show_continents(&bot, msg, user_id).await?;
        }
        Command::Cancel => {
            let conn = db::get_db().map_err(|e| {
                error!("DB error: {}", e);
                e
            })?;

            let mut session = db::get_user_session(&conn, user_id).map_err(|e| {
                error!("Session error: {}", e);
                e
            })?;

            session.current_state = "start".to_string();
            session.current_continent_id = None;
            session.current_country_id = None;
            session.current_city_id = None;
            session.current_category_id = None;
            session.current_menu_state = "city_menu".to_string();

            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                e
            })?;

            bot.send_message(
                msg.chat.id,
                "❌ Операция отменена. Используйте /start для начала.",
            )
            .await?;
        }
        Command::Stats => {
            if !utils::is_admin(user_id) {
                bot.send_message(msg.chat.id, "❌ У вас нет прав для просмотра статистики.")
                    .await?;
                return Ok(());
            }

            let conn = db::get_db().map_err(|e| {
                error!("DB error: {}", e);
                e
            })?;

            let vote_count = db::get_vote_count(&conn).map_err(|e| {
                error!("Vote count error: {}", e);
                e
            })?;

            let stats = format!(
                "📊 <b>Статистика голосований</b>\n\nВсего голосов: {}",
                vote_count
            );
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

pub async fn handle_callback(bot: Bot, q: CallbackQuery) -> Result<()> {
    let user_id = q.from.id.0 as i64;
    let callback_data = q.data.clone().unwrap_or_default();

    info!("Callback from user {}: {}", user_id, callback_data);

    let conn = db::get_db().map_err(|e| {
        error!("DB error: {}", e);
        e
    })?;

    let mut session = db::get_user_session(&conn, user_id).map_err(|e| {
        error!("Session error: {}", e);
        e
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
            e
        })?;

        let countries = db::get_countries_by_continent(&conn, continent_id).map_err(|e| {
            error!("Countries fetch error: {}", e);
            e
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

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
            let chat_id = q.from.id;
            bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, "🌍 Выберите страну:")
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
            e
        })?;

        let cities = db::get_cities_by_country(&conn, country_id).map_err(|e| {
            error!("Cities fetch error: {}", e);
            e
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

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
            let chat_id = q.from.id;
            bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, "🏙️ Выберите город:")
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
        session.current_state = "city_menu".to_string();
        session.current_menu_state = "city_menu".to_string();
        session.current_category_id = None;
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            e
        })?;

        let city_name = db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
        show_city_menu_edit(&bot, &q, &city_name).await?;
    } else if callback_data == "category_list" {
        if session.current_city_id.is_some() {
            session.current_menu_state = "categories".to_string();
            session.current_category_id = None;
            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                e
            })?;

            let categories = db::get_categories(&conn).map_err(|e| {
                error!("Categories fetch error: {}", e);
                e
            })?;

            let keyboard = build_categories_keyboard(&categories);

            if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
                let chat_id = q.from.id;
                bot.edit_message_text(
                    ChatId(chat_id.0 as i64),
                    msg_id,
                    "📂 Категории — нажмите для выбора:",
                )
                .reply_markup(keyboard)
                .await?;
            }
        }
    } else if callback_data.starts_with("category_") {
        let category_id: i32 = callback_data
            .strip_prefix("category_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        if let Some(city_id) = session.current_city_id {
            session.current_category_id = Some(category_id);
            session.current_menu_state = "items".to_string();
            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                e
            })?;

            let category_name = db::get_category_name(&conn, category_id)
                .unwrap_or_else(|_| "Категория".to_string());
            let items = db::get_items_by_category(&conn, category_id).map_err(|e| {
                error!("Items fetch error: {}", e);
                e
            })?;
            let user_votes = db::get_user_votes_for_city(&conn, user_id, city_id).map_err(|e| {
                error!("Votes fetch error: {}", e);
                e
            })?;
            let keyboard = build_items_keyboard(&items, &user_votes);

            if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
                let chat_id = q.from.id;
                bot.edit_message_text(
                    ChatId(chat_id.0 as i64),
                    msg_id,
                    format!("📂 <b>{}</b>\n\nВыберите пункты:", category_name),
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .await?;
            }
        }
    } else if callback_data == "city_stats" {
        if let Some(city_id) = session.current_city_id {
            session.current_menu_state = "stats".to_string();
            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                e
            })?;

            let stats = db::get_city_stats(&conn, city_id).map_err(|e| {
                error!("Stats fetch error: {}", e);
                e
            })?;

            let city_name =
                db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
            let mut text = format!("📊 <b>Статистика города {}</b>\n\n", city_name);
            for (cat, count) in &stats {
                text.push_str(&format!("• {} — {} голосов\n", cat, count));
            }

            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "◀️ Назад",
                "back_to_city_menu",
            )]]);

            if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
                let chat_id = q.from.id;
                bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;
            }
        }
    } else if callback_data == "my_marks" {
        if let Some(city_id) = session.current_city_id {
            session.current_menu_state = "marks".to_string();
            db::update_user_session(&conn, &session).map_err(|e| {
                error!("Session update error: {}", e);
                e
            })?;

            let marked_items = db::get_user_marked_items_for_city(&conn, user_id, city_id)
                .map_err(|e| {
                    error!("Votes fetch error: {}", e);
                    e
                })?;

            let city_name =
                db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
            let mut text = format!("📌 <b>Мои отметки — {}</b>\n\n", city_name);
            if marked_items.is_empty() {
                text.push_str("Пока нет отмеченных пунктов.");
            } else {
                for (category, item) in &marked_items {
                    text.push_str(&format!("✅ {} — {}\n", category, item));
                }
            }

            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "◀️ Назад",
                "back_to_city_menu",
            )]]);

            if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
                let chat_id = q.from.id;
                bot.edit_message_text(ChatId(chat_id.0 as i64), msg_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;
            }
        }
    } else if callback_data == "city_chat" {
        session.current_menu_state = "chat".to_string();
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            e
        })?;

        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            "◀️ Назад",
            "back_to_city_menu",
        )]]);

        if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
            let chat_id = q.from.id;
            bot.edit_message_text(
                ChatId(chat_id.0 as i64),
                msg_id,
                "💬 Чат города откроется позже",
            )
            .reply_markup(keyboard)
            .await?;
        }
    } else if callback_data == "back_to_city_menu" {
        session.current_menu_state = "city_menu".to_string();
        session.current_category_id = None;
        db::update_user_session(&conn, &session).map_err(|e| {
            error!("Session update error: {}", e);
            e
        })?;

        let city_name = session
            .current_city_id
            .and_then(|id| db::get_city_name(&conn, id).ok())
            .unwrap_or_else(|| "Город".to_string());

        show_city_menu_edit(&bot, &q, &city_name).await?;
    } else if callback_data.starts_with("item_") {
        let item_id: i32 = callback_data
            .strip_prefix("item_")
            .unwrap()
            .parse()
            .unwrap_or(0);

        if let (Some(city_id), Some(category_id)) =
            (session.current_city_id, session.current_category_id)
        {
            db::toggle_vote(&conn, user_id, city_id, item_id).map_err(|e| {
                error!("Toggle vote error: {}", e);
                e
            })?;

            let category_name = db::get_category_name(&conn, category_id)
                .unwrap_or_else(|_| "Категория".to_string());
            let items = db::get_items_by_category(&conn, category_id).map_err(|e| {
                error!("Items fetch error: {}", e);
                e
            })?;

            let user_votes = db::get_user_votes_for_city(&conn, user_id, city_id).map_err(|e| {
                error!("Votes fetch error: {}", e);
                e
            })?;

            let keyboard = build_items_keyboard(&items, &user_votes);

            if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
                let chat_id = q.from.id;
                bot.edit_message_text(
                    ChatId(chat_id.0 as i64),
                    msg_id,
                    format!("📂 <b>{}</b>\n\nВыберите пункты:", category_name),
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .await?;
            }
        }
    }

    bot.answer_callback_query(q.id).await?;

    Ok(())
}

pub async fn handle_message(bot: Bot, msg: Message) -> Result<()> {
    if let Some(text) = msg.text() {
        if !text.starts_with('/') {
            bot.send_message(
                msg.chat.id,
                "👋 Привет! Используйте /start для начала или /help для справки.",
            )
            .await?;
        }
    }

    Ok(())
}

async fn show_continents(bot: &Bot, msg: Message, user_id: i64) -> Result<()> {
    let conn = db::get_db().map_err(|e| {
        error!("DB error: {}", e);
        e
    })?;

    let continents = db::get_continents(&conn).map_err(|e| {
        error!("Continents fetch error: {}", e);
        e
    })?;

    let mut session = db::get_user_session(&conn, user_id).map_err(|e| {
        error!("Session error: {}", e);
        e
    })?;

    session.current_state = "continent".to_string();
    db::update_user_session(&conn, &session).map_err(|e| {
        error!("Session update error: {}", e);
        e
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

    bot.send_message(msg.chat.id, "🌍 Добро пожаловать! Выберите континент:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn show_city_menu_edit(bot: &Bot, q: &CallbackQuery, city_name: &str) -> Result<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "📂 Категории",
            "category_list",
        )],
        vec![InlineKeyboardButton::callback(
            "📊 Статистика города",
            "city_stats",
        )],
        vec![InlineKeyboardButton::callback("📌 Мои отметки", "my_marks")],
        vec![InlineKeyboardButton::callback("💬 Чат города", "city_chat")],
    ]);

    if let Some(msg_id) = q.message.as_ref().map(|m| m.id()) {
        let chat_id = q.from.id;
        bot.edit_message_text(
            ChatId(chat_id.0 as i64),
            msg_id,
            format!("🏙️ Город: <b>{}</b>\n\nВыберите раздел:", city_name),
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    }
    Ok(())
}

fn build_categories_keyboard(categories: &[crate::models::Category]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = categories
        .iter()
        .map(|c| {
            vec![InlineKeyboardButton::callback(
                c.name.clone(),
                format!("category_{}", c.id),
            )]
        })
        .collect();

    rows.push(vec![InlineKeyboardButton::callback(
        "◀️ Назад",
        "back_to_city_menu",
    )]);

    InlineKeyboardMarkup::new(rows)
}

fn build_items_keyboard(items: &[crate::models::Item], user_votes: &[i32]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = items
        .iter()
        .map(|item| {
            let mark = if user_votes.contains(&item.id) {
                "✅"
            } else {
                "▫️"
            };
            vec![InlineKeyboardButton::callback(
                format!("{} {}", mark, item.name),
                format!("item_{}", item.id),
            )]
        })
        .collect();

    rows.push(vec![InlineKeyboardButton::callback(
        "◀️ К категориям",
        "category_list",
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🏙️ В меню города",
        "back_to_city_menu",
    )]);

    InlineKeyboardMarkup::new(rows)
}
