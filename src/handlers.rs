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
            bot.send_message(
                msg.chat.id,
                format!("📊 <b>Статистика</b>\n\nВсего голосов: {}", vote_count),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Command::Help => {
            bot.send_message(
                msg.chat.id,
                Command::descriptions().to_string(),
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn handle_callback(bot: Bot, q: CallbackQuery) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await.ok();

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
        edit_msg(&bot, &q, "🌍 Выберите страну:", keyboard).await?;
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
        edit_msg(&bot, &q, "🏙️ Выберите город:", keyboard).await?;
    } else if callback_data.starts_with("city_") {
        let city_id: i32 = callback_data
            .strip_prefix("city_")
            .unwrap()
            .parse()
            .unwrap_or(0);
        session.current_city_id = Some(city_id);
        session.current_state = "city_menu".to_string();
        session.current_menu_state = "city_menu".to_string();
        db::update_user_session(&conn, &session)?;
        let city_name = db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
        show_city_menu_edit(&bot, &q, &city_name).await?;
    } else if callback_data == "back_to_city_menu" {
        if let Some(city_id) = session.current_city_id {
            session.current_menu_state = "city_menu".to_string();
            session.current_category_id = None;
            db::update_user_session(&conn, &session)?;
            let city_name = db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
            show_city_menu_edit(&bot, &q, &city_name).await?;
        }
    } else if callback_data == "category_list" {
        if session.current_city_id.is_some() {
            session.current_menu_state = "categories".to_string();
            db::update_user_session(&conn, &session)?;
            let categories = db::get_categories(&conn)?;
            let keyboard = build_categories_keyboard(&categories);
            edit_msg(&bot, &q, "📂 Выберите категорию:", keyboard).await?;
        }
    } else if callback_data.starts_with("cat_") {
        let category_id: i32 = callback_data
            .strip_prefix("cat_")
            .unwrap()
            .parse()
            .unwrap_or(0);
        if let Some(city_id) = session.current_city_id {
            session.current_category_id = Some(category_id);
            session.current_menu_state = "items".to_string();
            db::update_user_session(&conn, &session)?;
            show_items_edit(&bot, &q, &conn, user_id, city_id, category_id).await?;
        }
    } else if callback_data.starts_with("item_") {
        let item_id: i32 = callback_data
            .strip_prefix("item_")
            .unwrap()
            .parse()
            .unwrap_or(0);
        if let (Some(city_id), Some(category_id)) =
            (session.current_city_id, session.current_category_id)
        {
            let _ = db::toggle_vote(&conn, user_id, city_id, item_id)?;
            show_items_edit(&bot, &q, &conn, user_id, city_id, category_id).await?;
        }
    } else if callback_data == "city_stats" {
        if let Some(city_id) = session.current_city_id {
            session.current_menu_state = "stats".to_string();
            db::update_user_session(&conn, &session)?;
            let stats = db::get_city_stats(&conn, city_id)?;
            let city_name = db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
            let mut text = format!("📊 <b>Статистика города {}</b>\n\n", city_name);
            if stats.is_empty() {
                text.push_str("Пока нет отметок.");
            } else {
                for (name, count) in &stats {
                    text.push_str(&format!("• {} — {}\n", name, count));
                }
            }
            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "◀️ Назад",
                "back_to_city_menu",
            )]]);
            edit_msg_html(&bot, &q, &text, keyboard).await?;
        }
    } else if callback_data == "my_marks" {
        if let Some(city_id) = session.current_city_id {
            session.current_menu_state = "my_marks".to_string();
            db::update_user_session(&conn, &session)?;
            let marks = db::get_user_marks(&conn, user_id, city_id)?;
            let city_name = db::get_city_name(&conn, city_id).unwrap_or_else(|_| "Город".to_string());
            let mut text = format!("📌 <b>Мои отметки — {}</b>\n\n", city_name);
            if marks.is_empty() {
                text.push_str("Вы ещё ничего не отметили.");
            } else {
                for (cat, item) in &marks {
                    text.push_str(&format!("• <b>{}</b>: {}\n", cat, item));
                }
            }
            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "◀️ Назад",
                "back_to_city_menu",
            )]]);
            edit_msg_html(&bot, &q, &text, keyboard).await?;
        }
    } else if callback_data == "city_chat" {
        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            "◀️ Назад",
            "back_to_city_menu",
        )]]);
        edit_msg(
            &bot,
            &q,
            "💬 Чат города — скоро. Пока используйте главный чат сообщества.",
            keyboard,
        )
        .await?;
    }

    Ok(())
}

async fn show_continents(bot: &Bot, msg: Message, user_id: i64) -> Result<()> {
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

    bot.send_message(msg.chat.id, "🌍 Добро пожаловать! Выберите континент:")
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

async fn show_city_menu_edit(bot: &Bot, q: &CallbackQuery, city_name: &str) -> Result<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("📂 Категории", "category_list")],
        vec![InlineKeyboardButton::callback("📊 Статистика города", "city_stats")],
        vec![InlineKeyboardButton::callback("📌 Мои отметки", "my_marks")],
        vec![InlineKeyboardButton::callback("💬 Чат города", "city_chat")],
    ]);
    edit_msg_html(
        bot,
        q,
        &format!("🏙️ Город: <b>{}</b>\n\nВыберите раздел:", city_name),
        keyboard,
    )
    .await
}

async fn show_items_edit(
    bot: &Bot,
    q: &CallbackQuery,
    conn: &rusqlite::Connection,
    user_id: i64,
    city_id: i32,
    category_id: i32,
) -> Result<()> {
    let items = db::get_items_by_category(conn, category_id)?;
    let voted = db::get_user_item_votes_for_category(conn, user_id, city_id, category_id)?;
    let cat_name = db::get_category_name(conn, category_id).unwrap_or_else(|_| "Категория".into());
    let keyboard = build_items_keyboard(&items, &voted);
    let text = format!("📂 <b>{}</b>\nНажмите, чтобы отметить / снять:", cat_name);
    edit_msg_html(bot, q, &text, keyboard).await
}

fn build_categories_keyboard(categories: &[crate::models::Category]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = categories
        .iter()
        .map(|c| {
            vec![InlineKeyboardButton::callback(
                c.name.clone(),
                format!("cat_{}", c.id),
            )]
        })
        .collect();
    rows.push(vec![InlineKeyboardButton::callback(
        "◀️ Назад",
        "back_to_city_menu",
    )]);
    InlineKeyboardMarkup::new(rows)
}

fn build_items_keyboard(
    items: &[crate::models::Item],
    voted_ids: &[i32],
) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = items
        .iter()
        .map(|i| {
            let mark = if voted_ids.contains(&i.id) {
                "✅"
            } else {
                "▫️"
            };
            vec![InlineKeyboardButton::callback(
                format!("{} {}", mark, i.name),
                format!("item_{}", i.id),
            )]
        })
        .collect();
    rows.push(vec![InlineKeyboardButton::callback(
        "◀️ К категориям",
        "category_list",
    )]);
    InlineKeyboardMarkup::new(rows)
}

async fn edit_msg(
    bot: &Bot,
    q: &CallbackQuery,
    text: &str,
    keyboard: InlineKeyboardMarkup,
) -> Result<()> {
    if let Some(msg) = q.message.as_ref() {
        let _ = bot
            .edit_message_text(msg.chat().id, msg.id(), text)
            .reply_markup(keyboard)
            .await;
    }
    Ok(())
}

async fn edit_msg_html(
    bot: &Bot,
    q: &CallbackQuery,
    text: &str,
    keyboard: InlineKeyboardMarkup,
) -> Result<()> {
    if let Some(msg) = q.message.as_ref() {
        let _ = bot
            .edit_message_text(msg.chat().id, msg.id(), text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await;
    }
    Ok(())
}
