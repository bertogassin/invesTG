/// Optional Telegram DM notifications for web events (chat, etc.).
///
/// When the `telegram-bot` feature is disabled or `TELEGRAM_BOT_TOKEN` is unset,
/// this becomes a no-op so the Axum server can run without Teloxide.

pub fn notify_telegram_user(bot_token: Option<&str>, telegram_id: i64, text: &str) {
    if telegram_id <= 0 {
        return;
    }

    let Some(token) = bot_token.filter(|value| !value.is_empty()) else {
        return;
    };

    #[cfg(feature = "telegram-bot")]
    {
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
        let _ = text;
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
