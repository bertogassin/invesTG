use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, WebAppInfo};

pub async fn start_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let map_web_app = WebAppInfo {
        url: "https://resursmap.de/app/auth".to_string().parse().unwrap(),
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
            url: "https://resursmap.de/app/admin/login"
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
            "Привет! Я бот Карта ресурсов.\n\n\
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
