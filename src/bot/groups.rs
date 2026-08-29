use teloxide::prelude::*;
use teloxide::types::{
    ChatMemberKind, ChatMemberUpdated, InlineKeyboardButton, InlineKeyboardMarkup,
};

const COMMUNITY_CHANNEL_URL: &str = "https://t.me/omnixiuschannel";
const RESURSMAP_URL: &str = "https://resursmap.de/app?entry=telegram&v=public-1";

pub async fn welcome_chat_member(bot: Bot, update: ChatMemberUpdated) -> ResponseResult<()> {
    let was_outside = matches!(
        update.old_chat_member.kind,
        ChatMemberKind::Left | ChatMemberKind::Banned(_)
    );

    let is_inside = matches!(
        update.new_chat_member.kind,
        ChatMemberKind::Member(_)
            | ChatMemberKind::Administrator(_)
            | ChatMemberKind::Owner(_)
            | ChatMemberKind::Restricted(_)
    );

    if !was_outside || !is_inside {
        return Ok(());
    }

    let user = &update.new_chat_member.user;

    if user.is_bot {
        return Ok(());
    }

    let greeting = format!(
        "Добро пожаловать, {}! 👋\n\n\
         🌍 Рады видеть вас в нашем сообществе.\n\n\
         📢 Основные новости и важные обновления публикуются \
         в канале Omnixius.\n\n\
         🗺 ResursMap помогает находить людей, специалистов, \
         ресурсы и возможности.",
        user.first_name
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::url(
            "📢 Подписаться на Omnixius",
            COMMUNITY_CHANNEL_URL.parse().expect("valid channel URL"),
        )],
        vec![InlineKeyboardButton::url(
            "🌍 Открыть ResursMap",
            RESURSMAP_URL.parse().expect("valid ResursMap URL"),
        )],
    ]);

    match bot
        .send_message(update.chat.id, &greeting)
        .reply_markup(keyboard)
        .await
    {
        Ok(_message) => {}

        Err(_err) => {
            // Резервный вариант: если Telegram почему-либо
            // отвергнет клавиатуру, человек всё равно получит приветствие.
            let fallback = format!(
                "{}\n\n📢 {}\n🌍 {}",
                greeting, COMMUNITY_CHANNEL_URL, RESURSMAP_URL
            );

            match bot.send_message(update.chat.id, fallback).await {
                Ok(_message) => {}

                Err(fallback_err) => {
                    return Err(fallback_err);
                }
            }
        }
    }

    Ok(())
}
