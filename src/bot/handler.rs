use crate::db::pool::DbPool;
use crate::geography::world;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, WebAppInfo};

#[allow(dead_code)]
pub async fn send_notification(bot: &Bot, telegram_id: i64, text: &str) -> ResponseResult<()> {
    bot.send_message(ChatId(telegram_id), text).await?;
    Ok(())
}

fn parse_city_registration(value: &str) -> Option<(usize, usize, usize)> {
    let parts = value.split_whitespace().collect::<Vec<_>>();

    if parts.len() != 3 {
        return None;
    }

    let continent_index = parts[0].parse().ok()?;
    let country_index = parts[1].parse().ok()?;
    let city_index = parts[2].parse().ok()?;

    Some((continent_index, country_index, city_index))
}

fn city_from_indices(
    continent_index: usize,
    country_index: usize,
    city_index: usize,
) -> Option<(&'static str, &'static str, &'static str)> {
    let geography = world();

    let (continent, countries) = geography.iter().nth(continent_index)?;

    let (country, cities) = countries.iter().nth(country_index)?;

    let city = cities.get(city_index)?;

    Some((*continent, *country, *city))
}

pub async fn register_city_handler(
    bot: Bot,
    msg: Message,
    arguments: String,
    db_pool: DbPool,
) -> ResponseResult<()> {
    let configured_admin_id = env::var("ADMIN_TELEGRAM_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());

    let sender_id = msg.from.as_ref().map(|user| user.id.0);

    if configured_admin_id.is_none() || sender_id != configured_admin_id {
        bot.send_message(
            msg.chat.id,
            "Команда доступна только главному администратору ResursMap.",
        )
        .await?;

        return Ok(());
    }

    if msg.chat.id.0 >= 0 {
        bot.send_message(
            msg.chat.id,
            "Эту команду необходимо отправить внутри городской группы.",
        )
        .await?;

        return Ok(());
    }

    let Some((continent_index, country_index, city_index)) = parse_city_registration(&arguments)
    else {
        bot.send_message(
            msg.chat.id,
            "Формат команды:\n/registercity КОНТИНЕНТ СТРАНА ГОРОД\n\nДля Ниццы: /registercity 0 2 4",
        )
        .await?;

        return Ok(());
    };

    let Some((continent, country, city)) =
        city_from_indices(continent_index, country_index, city_index)
    else {
        bot.send_message(
            msg.chat.id,
            "Город с такими координатами не найден в ResursMap.",
        )
        .await?;

        return Ok(());
    };

    let city_links = load_cities();

    let telegram_url = city_links.get(city).cloned().unwrap_or_default();

    let connection = match db_pool.get() {
        Ok(connection) => connection,

        Err(_) => {
            bot.send_message(
                msg.chat.id,
                "Не удалось подключить группу: база данных недоступна.",
            )
            .await?;

            return Ok(());
        }
    };

    let changed = connection
        .execute(
            "INSERT INTO city_publication_targets (
                continent_index,
                country_index,
                city_index,
                city_name,
                target_name,
                telegram_chat_id,
                telegram_url,
                target_kind,
                is_active,
                created_at,
                updated_at
             )
             VALUES (
                ?1, ?2, ?3, ?4,
                'Основная городская группа',
                ?5, ?6, 'group', 1,
                strftime('%s','now'),
                strftime('%s','now')
             )
             ON CONFLICT (
                continent_index,
                country_index,
                city_index,
                target_name
             )
             DO UPDATE SET
                city_name = excluded.city_name,
                telegram_chat_id =
                    excluded.telegram_chat_id,
                telegram_url =
                    excluded.telegram_url,
                target_kind = 'group',
                is_active = 1,
                updated_at = strftime('%s','now')",
            rusqlite::params![
                continent_index,
                country_index,
                city_index,
                city,
                msg.chat.id.0,
                telegram_url,
            ],
        )
        .unwrap_or(0);

    drop(connection);

    if changed != 1 {
        bot.send_message(msg.chat.id, "Не удалось сохранить городскую группу.")
            .await?;

        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Группа подключена к ResursMap\n\n📍 {}, {}\n🌍 {}\n\nТеперь сюда можно будет публиковать одобренные объявления ResursMap.",
            city,
            country,
            continent,
        ),
    )
    .await?;

    Ok(())
}

pub async fn start_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let map_web_app = WebAppInfo {
        url: "https://resursmap.de/app?entry=telegram&v=public-1"
            .to_string()
            .parse()
            .unwrap(),
    };

    let mut keyboard_rows = vec![vec![InlineKeyboardButton::web_app(
        "🌍 Открыть карту",
        map_web_app,
    )]];

    // Кнопку модерации получает только ADMIN_TELEGRAM_ID.
    let admin_telegram_id = env::var("ADMIN_TELEGRAM_ID")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());

    let current_user_id = msg.from.as_ref().map(|user| user.id.0);

    if admin_telegram_id.is_some() && current_user_id == admin_telegram_id {
        let admin_web_app = WebAppInfo {
            url: "https://resursmap.de/login?next=%2Fapp%2Fcenter"
                .to_string()
                .parse()
                .unwrap(),
        };

        keyboard_rows.push(vec![InlineKeyboardButton::web_app(
            "🛡 Модерация",
            admin_web_app,
        )]);
    }

    let keyboard = InlineKeyboardMarkup::new(keyboard_rows);

    let cities = load_cities();
    let mut city_list = String::new();

    for (city, _) in cities.iter().take(10) {
        city_list.push_str(&format!("• {}\n", city));
    }

    bot.send_message(
        msg.chat.id,
        format!(
            "Привет! Я бот ResursMap.\n\n\
             Работа, люди и бизнес рядом.\n\n\
             Доступные команды:\n\
             /start — приветствие\n\
             /help — помощь\n\
             /cities — список городов\n\n\
             🌍 Города (первые 10):\n{}",
            city_list
        ),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub async fn help_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "Доступные команды:\n/start — приветствие\n/help — помощь\n/cities — список городов с ссылками",
    ).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn stats_handler(
    bot: Bot,
    msg: Message,
    db_pool: crate::db::pool::DbPool,
) -> ResponseResult<()> {
    let users_count: i64 = db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM users WHERE is_active = 1",
                [],
                |row| row.get(0),
            )
            .ok()
        })
        .unwrap_or(0);

    let resources_count: i64 = db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM resources WHERE is_active = 1",
                [],
                |row| row.get(0),
            )
            .ok()
        })
        .unwrap_or(0);

    let online_count: i64 = db_pool
        .get()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM profiles WHERE last_seen_at > strftime('%s','now') - 300",
                [],
                |row| row.get(0),
            )
            .ok()
        })
        .unwrap_or(0);

    let text = format!(
        "📊 Статистика ResursMap\n\n👥 Людей: {}\n🟢 Онлайн: {}\n📦 Ресурсов: {}\n\n🌍 https://resursmap.de/app",
        users_count, online_count, resources_count
    );

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

pub async fn cities_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let cities = load_cities();
    let mut response = String::from("🌍 Города и чаты:\n\n");
    for (city, link) in cities {
        response.push_str(&format!("📍 {}: {}\n", city, link));
    }
    bot.send_message(msg.chat.id, response).await?;
    Ok(())
}

fn load_cities() -> HashMap<String, String> {
    let path = "data/city_chats.json";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            if let Some(obj) = json.as_object() {
                let mut map = HashMap::new();
                for (city, link) in obj {
                    if let Some(link_str) = link.as_str() {
                        map.insert(city.clone(), link_str.to_string());
                    }
                }
                return map;
            }
        }
    }
    HashMap::new()
}

#[cfg(test)]
mod city_registration_tests {
    use super::*;

    #[test]
    fn registration_arguments_are_strict() {
        assert_eq!(parse_city_registration("0 2 4"), Some((0, 2, 4)),);

        assert_eq!(parse_city_registration(" 0 2 4 "), Some((0, 2, 4)),);

        assert_eq!(parse_city_registration("0 2"), None,);

        assert_eq!(parse_city_registration("0 2 4 extra"), None,);

        assert_eq!(parse_city_registration("0 france 4"), None,);
    }

    #[test]
    fn nice_coordinates_are_stable() {
        assert_eq!(
            city_from_indices(0, 2, 4),
            Some(("Европа", "Франция", "Ницца")),
        );
    }

    #[test]
    fn invalid_city_coordinates_are_rejected() {
        assert_eq!(city_from_indices(0, 2, 999), None,);
    }
}
