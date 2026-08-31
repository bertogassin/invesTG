use crate::state::app_state::AppState;
use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(super) fn telegram_owner_user_id(client_id: &str) -> Option<i64> {
    resource_owner_user_id(client_id)
}

pub(super) fn resource_owner_user_id(client_id: &str) -> Option<i64> {
    if let Some(value) = client_id.strip_prefix("user:") {
        return value.parse::<i64>().ok();
    }

    client_id
        .strip_prefix("tg:")
        .and_then(|value| value.parse::<i64>().ok())
}

pub(super) fn input_text_is_valid(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let length = value.chars().count();

    if length < min_chars || length > max_chars {
        return false;
    }

    // Запрещаем управляющие символы, которые не нужны
    // обычному пользовательскому тексту.
    //
    // Перевод строки / CR / TAB разрешаем:
    // они нужны описаниям, сообщениям и статусам.
    !value
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
}

pub(super) fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub(super) async fn rate_limit_retry_after(
    state: &AppState,
    user_id: i64,
    action: &str,
    max_requests: usize,
    window_seconds: i64,
) -> Option<u64> {
    let now = unix_now();
    let cutoff = now.saturating_sub(window_seconds);
    let key = format!("{}:{}", action, user_id);

    let mut limits = state.rate_limits.lock().await;
    let events = limits.entry(key).or_default();

    // Sliding window:
    // забываем только запросы, вышедшие за текущее окно.
    while let Some(timestamp) = events.front().copied() {
        if timestamp <= cutoff {
            events.pop_front();
        } else {
            break;
        }
    }

    if events.len() >= max_requests {
        let oldest = events.front().copied().unwrap_or(now);

        let retry_after = oldest
            .saturating_add(window_seconds)
            .saturating_sub(now)
            .max(1);

        return Some(retry_after as u64);
    }

    events.push_back(now);

    None
}

fn trusted_request_origin(origin: &str) -> bool {
    matches!(
        origin.trim_end_matches('/'),
        "https://resursmap.de"
            | "https://www.resursmap.de"
            | "http://127.0.0.1:3000"
            | "https://t.me"
            | "https://telegram.me"
            | "https://telegram.org"
            | "https://web.telegram.org"
    )
}

pub(super) fn request_is_cross_site(headers: &HeaderMap) -> bool {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::trim);

    let fetch_site = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .map(str::trim);

    let fetch_is_cross_site =
        fetch_site.is_some_and(|value| value.eq_ignore_ascii_case("cross-site"));

    // Обычный браузер и доверенные Telegram-контейнеры.
    if origin.is_some_and(trusted_request_origin) {
        return false;
    }

    // Некоторые Android WebView отправляют Origin: null
    // для формы, открытой внутри доверенного приложения.
    //
    // Такой запрос разрешается только если браузер не
    // обозначил его как настоящий cross-site запрос.
    if origin.is_some_and(|value| value.eq_ignore_ascii_case("null")) {
        return fetch_is_cross_site;
    }

    // Любой другой явно указанный Origin не доверен.
    if origin.is_some() {
        return true;
    }

    // Если Origin отсутствует, Sec-Fetch-Site остаётся
    // дополнительной защитой от внешней формы.
    fetch_is_cross_site
}

pub(super) fn csrf_rejected_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "ok": false,
            "error": "cross_site_request_rejected"
        })),
    )
        .into_response()
}

#[cfg(test)]
mod request_origin_tests {
    use super::*;

    #[test]
    fn opaque_android_origin_is_allowed_only_when_not_cross_site() {
        let mut same_origin = HeaderMap::new();

        same_origin.insert(header::ORIGIN, "null".parse().expect("origin"));

        same_origin.insert("sec-fetch-site", "same-origin".parse().expect("fetch site"));

        assert!(!request_is_cross_site(&same_origin));

        let mut cross_site = HeaderMap::new();

        cross_site.insert(header::ORIGIN, "null".parse().expect("origin"));

        cross_site.insert("sec-fetch-site", "cross-site".parse().expect("fetch site"));

        assert!(request_is_cross_site(&cross_site));
    }

    #[test]
    fn missing_origin_does_not_override_cross_site_signal() {
        let mut headers = HeaderMap::new();

        headers.insert("sec-fetch-site", "cross-site".parse().expect("fetch site"));

        assert!(request_is_cross_site(&headers));
    }

    #[test]
    fn trusted_origins_are_strict() {
        assert!(trusted_request_origin("https://resursmap.de"));
        assert!(trusted_request_origin("https://t.me"));
        assert!(trusted_request_origin("https://web.telegram.org"));
        assert!(!trusted_request_origin("https://evil.example"));
        assert!(!trusted_request_origin("https://resursmap.de.evil.example"));
    }
}
