pub fn notify_telegram_user(bot_token: Option<&str>, telegram_id: i64, text: &str) {
    if telegram_id <= 0 {
        return;
    }

    #[cfg(feature = "telegram-bot")]
    {
        let Some(token) = bot_token.filter(|value| !value.is_empty()) else {
            return;
        };

        let token = token.to_string();
        let text = text.to_string();

        tokio::spawn(async move {
            if let Err(error) = deliver(token, telegram_id, text).await {
                eprintln!("telegram notification failed: {error}");
            }
        });
    }

    #[cfg(not(feature = "telegram-bot"))]
    {
        let _ = (bot_token, text);
    }
}

#[cfg(feature = "telegram-bot")]
async fn deliver(
    token: String,
    telegram_id: i64,
    text: String,
) -> Result<(), teloxide::RequestError> {
    use teloxide::prelude::*;

    Bot::new(token)
        .send_message(ChatId(telegram_id), text)
        .await?;

    Ok(())
}

pub async fn publish_to_telegram_group(
    bot_token: Option<&str>,
    chat_id: i64,
    text: &str,
) -> Result<i64, String> {
    if chat_id >= 0 {
        return Err("invalid_group_chat".into());
    }

    #[cfg(feature = "telegram-bot")]
    {
        let Some(token) = bot_token.filter(|value| !value.is_empty()) else {
            return Err("bot_token_missing".into());
        };

        deliver_group(token.to_string(), chat_id, text.to_string())
            .await
            .map(|message_id| message_id as i64)
            .map_err(|error| error.to_string())
    }

    #[cfg(not(feature = "telegram-bot"))]
    {
        let _ = (bot_token, chat_id, text);
        Err("telegram_bot_feature_disabled".into())
    }
}

#[cfg(feature = "telegram-bot")]
async fn deliver_group(
    token: String,
    chat_id: i64,
    text: String,
) -> Result<i32, teloxide::RequestError> {
    use teloxide::prelude::*;

    let message = Bot::new(token)
        .send_message(ChatId(chat_id), text)
        .await?;

    Ok(message.id.0)
}
