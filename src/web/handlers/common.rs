use crate::state::app_state::AppState;
use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(super) fn telegram_owner_user_id(client_id: &str) -> Option<i64> {
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

pub(super) fn request_is_cross_site(headers: &HeaderMap) -> bool {
    // Современные браузеры прямо сообщают происхождение запроса.
    // Если они говорят cross-site — такой state-changing запрос
    // с cookie-сессией нам не нужен.
    if let Some(value) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        if value.eq_ignore_ascii_case("cross-site") {
            return true;
        }
    }

    // Дополнительная проверка Origin.
    //
    // Заголовок может отсутствовать у curl, некоторых WebView и
    // старых клиентов — отсутствие Origin само по себе НЕ блокируем.
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let origin = origin.trim_end_matches('/');

        if origin != "https://resursmap.de"
            && origin != "https://www.resursmap.de"
            && origin != "http://127.0.0.1:3000"
        {
            return true;
        }
    }

    false
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
