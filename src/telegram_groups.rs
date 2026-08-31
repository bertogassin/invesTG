#[derive(Debug, Clone)]
pub struct TelegramGroupInfo {
    pub title: String,
    pub _chat_type: String,
}

pub async fn verify_telegram_group(
    bot_token: Option<&str>,
    chat_id: i64,
) -> Result<TelegramGroupInfo, String> {
    if chat_id >= 0 {
        return Err("invalid_group_chat_id".into());
    }

    let token = bot_token
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "bot_token_missing".to_string())?;

    let response = reqwest::Client::new()
        .get(format!("https://api.telegram.org/bot{token}/getChat"))
        .query(&[("chat_id", chat_id.to_string())])
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let payload: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;

    if payload.get("ok").and_then(|value| value.as_bool()) != Some(true) {
        let description = payload
            .get("description")
            .and_then(|value| value.as_str())
            .unwrap_or("telegram_get_chat_failed");
        return Err(description.to_string());
    }

    let result = payload
        .get("result")
        .ok_or_else(|| "telegram_missing_result".to_string())?;

    let title = result
        .get("title")
        .or_else(|| result.get("username"))
        .and_then(|value| value.as_str())
        .unwrap_or("Telegram group")
        .to_string();

    let chat_type = result
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(TelegramGroupInfo {
        title,
        _chat_type: chat_type,
    })
}
