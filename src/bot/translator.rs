use std::env;

use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

const DEFAULT_LIBRETRANSLATE_URL: &str = "https://libretranslate.com/translate";

const MAX_TRANSLATION_CHARS: usize = 2000;

#[derive(Debug, Serialize)]
struct LibreTranslateRequest<'a> {
    q: &'a str,
    source: &'a str,
    target: &'a str,
    format: &'a str,
}

#[derive(Debug, Deserialize)]
struct LibreTranslateResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

fn normalize_target_language(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ru" | "рус" | "russian" => Some("ru"),
        "en" | "eng" | "english" => Some("en"),
        "fr" | "fra" | "french" => Some("fr"),
        "de" | "ger" | "german" => Some("de"),
        "es" | "spa" | "spanish" => Some("es"),
        "it" | "ita" | "italian" => Some("it"),
        "pt" | "por" | "portuguese" => Some("pt"),
        "pl" | "pol" | "polish" => Some("pl"),
        "nl" | "dut" | "dutch" => Some("nl"),
        "uk" | "ua" | "ukrainian" => Some("uk"),
        "tr" | "turkish" => Some("tr"),
        "ar" | "arabic" => Some("ar"),
        "zh" | "chinese" => Some("zh"),
        "ja" | "japanese" => Some("ja"),
        "ko" | "korean" => Some("ko"),
        _ => None,
    }
}

fn parse_translation_command(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();

    let rest = if let Some(rest) = trimmed.strip_prefix("/tr") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("/translate") {
        rest
    } else {
        return None;
    };

    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }

    let rest = rest.trim();

    if rest.is_empty() {
        return Some(("", ""));
    }

    let mut split = rest.splitn(2, char::is_whitespace);

    let language = split.next().unwrap_or("");
    let body = split.next().unwrap_or("").trim();

    Some((language, body))
}

async fn translate_with_libretranslate(text: &str, target_lang: &str) -> Result<String, String> {
    let api_url =
        env::var("LIBRETRANSLATE_URL").unwrap_or_else(|_| DEFAULT_LIBRETRANSLATE_URL.to_string());

    let client = reqwest::Client::new();

    let payload = LibreTranslateRequest {
        q: text,
        source: "auto",
        target: target_lang,
        format: "text",
    };

    let response = client
        .post(api_url)
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("HTTP_ERROR: {err}"))?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();

        return Err(format!(
            "LIBRETRANSLATE_HTTP_STATUS={} BODY={}",
            status.as_u16(),
            body
        ));
    }

    let payload = response
        .json::<LibreTranslateResponse>()
        .await
        .map_err(|err| format!("JSON_ERROR: {err}"))?;

    if payload.translated_text.trim().is_empty() {
        return Err("EMPTY_TRANSLATION".to_string());
    }

    Ok(payload.translated_text)
}

pub fn is_translation_command(msg: &Message) -> bool {
    let Some(text) = msg.text() else {
        return false;
    };

    parse_translation_command(text).is_some()
}

pub async fn translation_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let Some(command_text) = msg.text() else {
        return Ok(());
    };

    let Some((language_raw, inline_text)) = parse_translation_command(command_text) else {
        return Ok(());
    };

    if language_raw.is_empty() {
        bot.send_message(
            msg.chat.id,
            "🌐 Переводчик\n\n\
             Использование:\n\
             /tr fr Привет\n\
             /tr en Bonjour\n\
             /tr ru Hello\n\n\
             Или ответьте командой /tr fr \
             на сообщение, которое хотите перевести.",
        )
        .await?;

        return Ok(());
    }

    let Some(target_lang) = normalize_target_language(language_raw) else {
        bot.send_message(
            msg.chat.id,
            "Неизвестный язык.\n\n\
             Поддерживаются: ru, en, fr, de, es, \
             it, pt, pl, nl, uk, tr, ar, zh, ja, ko.",
        )
        .await?;

        return Ok(());
    };

    let source_text = if !inline_text.is_empty() {
        inline_text.to_string()
    } else {
        msg.reply_to_message()
            .and_then(|reply| reply.text())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    if source_text.is_empty() {
        bot.send_message(
            msg.chat.id,
            "Ответьте командой на текстовое сообщение \
             или напишите текст после языка.\n\n\
             Пример: /tr fr Привет",
        )
        .await?;

        return Ok(());
    }

    if source_text.chars().count() > MAX_TRANSLATION_CHARS {
        bot.send_message(
            msg.chat.id,
            "Текст слишком длинный. \
             Максимум 2000 символов за один перевод.",
        )
        .await?;

        return Ok(());
    }

    match translate_with_libretranslate(&source_text, target_lang).await {
        Ok(translated) => {
            bot.send_message(
                msg.chat.id,
                format!("🌐 {}:\n\n{}", target_lang.to_uppercase(), translated),
            )
            .await?;
        }

        Err(error) => {
            eprintln!("TRANSLATION_ERROR chat_id={} error={}", msg.chat.id, error);

            bot.send_message(
                msg.chat.id,
                "Не удалось выполнить перевод. \
                 Попробуйте немного позже.",
            )
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_translation() {
        assert_eq!(
            parse_translation_command("/tr fr Привет мир"),
            Some(("fr", "Привет мир"))
        );
    }

    #[test]
    fn parses_reply_translation() {
        assert_eq!(parse_translation_command("/tr en"), Some(("en", "")));
    }

    #[test]
    fn rejects_similar_command() {
        assert_eq!(parse_translation_command("/trash test"), None);
    }

    #[test]
    fn languages_normalize() {
        assert_eq!(normalize_target_language("fr"), Some("fr"));

        assert_eq!(normalize_target_language("RU"), Some("ru"));

        assert_eq!(normalize_target_language("xyz"), None);
    }
}
