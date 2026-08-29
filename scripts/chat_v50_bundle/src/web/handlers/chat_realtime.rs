use super::auth::verify_user_session;
use super::common::request_is_cross_site;
use crate::state::app_state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;

#[derive(Debug, Deserialize)]
struct ClientFrame {
    #[serde(rename = "type")]
    frame_type: String,
    other_user_id: Option<String>,
}

fn parse_other_user_id(value: Option<&str>) -> Option<i64> {
    let raw = value?.trim();
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    raw.parse::<i64>().ok().filter(|id| *id > 0)
}

fn handle_client_frame(state: &AppState, user_id: i64, text: &str) -> bool {
    let frame = match serde_json::from_str::<ClientFrame>(text) {
        Ok(frame) => frame,
        Err(_) => return text.contains("\"ping\""),
    };

    match frame.frame_type.as_str() {
        "ping" => true,
        "typing.start" | "typing.stop" => {
            let Some(other_user_id) = parse_other_user_id(frame.other_user_id.as_deref()) else {
                return false;
            };

            if user_id == other_user_id {
                return false;
            }

            let _ = state.publish_typing_event(&frame.frame_type, user_id, other_user_id);
            false
        }
        _ => false,
    }
}

pub async fn api_chat_realtime(
    State(state): State<AppState>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Response {
    if request_is_cross_site(&headers) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "ok": false,
                "error": "cross_site_request_rejected"
            })),
        )
            .into_response();
    }

    let user_id = match verify_user_session(&state, &headers) {
        Some(user_id) => user_id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "login_required"
                })),
            )
                .into_response();
        }
    };

    websocket
        .on_upgrade(move |socket| chat_socket(socket, state, user_id))
        .into_response()
}

async fn send_json(socket: &mut WebSocket, value: serde_json::Value) -> bool {
    socket
        .send(Message::Text(value.to_string().into()))
        .await
        .is_ok()
}

async fn chat_socket(mut socket: WebSocket, state: AppState, user_id: i64) {
    let mut events = state.chat_events.subscribe();
    let mut typing_events = state.chat_typing_events.subscribe();

    if !send_json(
        &mut socket,
        json!({
            "type": "ready",
            "protocol": "resursmap.chat.v5",
            "user_id": user_id.to_string()
        }),
    )
    .await
    {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if handle_client_frame(&state, user_id, text.as_str())
                            && !send_json(
                                &mut socket,
                                json!({
                                    "type": "pong"
                                }),
                            )
                            .await
                        {
                            break;
                        }
                    }

                    Some(Ok(Message::Ping(payload))) => {
                        if socket
                            .send(Message::Pong(payload))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }

                    Some(Ok(Message::Close(_))) |
                    Some(Err(_)) |
                    None => {
                        break;
                    }

                    _ => {}
                }
            }

            event = events.recv() => {
                match event {
                    Ok(event) => {
                        if !event.includes_user(user_id) {
                            continue;
                        }

                        if !send_json(
                            &mut socket,
                            json!({
                                "type": "chat_event",
                                "event": event
                            }),
                        )
                        .await
                        {
                            break;
                        }
                    }

                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        if !send_json(
                            &mut socket,
                            json!({
                                "type": "sync_required"
                            }),
                        )
                        .await
                        {
                            break;
                        }
                    }

                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            typing = typing_events.recv() => {
                match typing {
                    Ok(event) => {
                        if !event.is_visible_to(user_id) {
                            continue;
                        }

                        if !send_json(
                            &mut socket,
                            json!({
                                "type": "typing_event",
                                "event": event
                            }),
                        )
                        .await
                        {
                            break;
                        }
                    }

                    Err(broadcast::error::RecvError::Lagged(_)) => {}

                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_other_user_id, ClientFrame};

    #[test]
    fn other_user_id_parser_is_strict() {
        assert_eq!(parse_other_user_id(Some("42")), Some(42));
        assert_eq!(parse_other_user_id(Some(" 7 ")), Some(7));
        assert_eq!(parse_other_user_id(Some("0")), None);
        assert_eq!(parse_other_user_id(Some("-1")), None);
        assert_eq!(parse_other_user_id(Some("abc")), None);
    }

    #[test]
    fn client_frame_parses_typing() {
        let frame: ClientFrame = serde_json::from_str(
            r#"{"type":"typing.start","other_user_id":"18"}"#,
        )
        .expect("typing frame");

        assert_eq!(frame.frame_type, "typing.start");
        assert_eq!(frame.other_user_id.as_deref(), Some("18"));
    }

    #[test]
    fn realtime_protocol_name_is_stable() {
        assert_eq!("resursmap.chat.v5", "resursmap.chat.v5");
    }
}
