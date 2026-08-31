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
        let Some(token) = bot_token.filter(|value| !value.is_empty()) else {
            return;
        };

        let token = token.to_string();
        let text = text.to_string();

        tokio::spawn(async move {
            if let Err(error) = send_telegram_message_http(&token, telegram_id, &text).await {
                eprintln!("telegram notification failed: {error}");
            }
        });
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

    let token = bot_token
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "bot_token_missing".to_string())?;

    send_telegram_message_http(token, chat_id, text).await
}

async fn send_telegram_message_http(token: &str, chat_id: i64, text: &str) -> Result<i64, String> {
    let response = reqwest::Client::new()
        .post(format!("https://api.telegram.org/bot{token}/sendMessage"))
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "disable_web_page_preview": false,
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let payload: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;

    if payload.get("ok").and_then(|value| value.as_bool()) != Some(true) {
        let description = payload
            .get("description")
            .and_then(|value| value.as_str())
            .unwrap_or("telegram_send_failed");
        return Err(description.to_string());
    }

    payload
        .get("result")
        .and_then(|value| value.get("message_id"))
        .and_then(|value| value.as_i64())
        .ok_or_else(|| "telegram_missing_message_id".to_string())
}
