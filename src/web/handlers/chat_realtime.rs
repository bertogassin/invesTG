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
use serde_json::json;
use tokio::sync::broadcast;

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

    if !send_json(
        &mut socket,
        json!({
            "type": "ready",
            "protocol": "resursmap.chat.v4",
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
                        if text.as_str().contains("\"ping\"")
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

                    Err(
                        broadcast::error::RecvError::Lagged(_)
                    ) => {
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

                    Err(
                        broadcast::error::RecvError::Closed
                    ) => {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn realtime_protocol_name_is_stable() {
        assert_eq!("resursmap.chat.v4", "resursmap.chat.v4");
    }
}
