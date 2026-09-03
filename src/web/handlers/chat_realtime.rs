use super::auth::verify_user_session;
use crate::state::app_state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use tokio::time::{interval, Duration};

fn cross_site(headers: &HeaderMap) -> bool {
    headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("cross-site"))
}

pub async fn api_chat_realtime(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if cross_site(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Some(user_id) = verify_user_session(&state, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    ws.on_upgrade(move |socket| live_socket(socket, user_id))
}

async fn live_socket(mut socket: WebSocket, user_id: i64) {
    let ready = json!({
        "type": "ready",
        "protocol": "resursmap.messenger.clean.v1",
        "user_id": user_id.to_string()
    })
    .to_string();

    if socket.send(Message::Text(ready.into())).await.is_err() {
        return;
    }

    let mut ticks = interval(Duration::from_millis(900));
    let mut n: u64 = 0;

    loop {
        tokio::select! {
            _ = ticks.tick() => {
                n = n.wrapping_add(1);
                let frame = json!({"type":"sync","seq":n}).to_string();
                if socket.send(Message::Text(frame.into())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if text.as_str() == "ping" {
                            if socket.send(Message::Text("pong".into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(bytes))) => {
                        if socket.send(Message::Pong(bytes)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}
