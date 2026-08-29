use crate::db::pool::DbPool;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{broadcast, Mutex};

#[derive(Clone, Debug, Serialize)]
pub struct ChatRealtimeEvent {
    pub event_id: u64,
    pub kind: String,
    pub conversation_id: i64,
    pub message_id: i64,
    pub user1_id: i64,
    pub user2_id: i64,
}

impl ChatRealtimeEvent {
    pub fn includes_user(&self, user_id: i64) -> bool {
        self.user1_id == user_id || self.user2_id == user_id
    }
}

#[derive(Clone)]
pub struct AppState {
    // Новый SQLite connection pool.
    pub db_pool: DbPool,

    pub bot_token: String,
    pub admin_key: String,
    pub admin_telegram_id: i64,

    // Быстрый process-local rate limiter.
    //
    // Ключ:
    //     "<action>:<telegram_user_id>"
    //
    // Значение:
    //     timestamps последних запросов в sliding window.
    //
    // Это не бизнес-данные, поэтому хранить limiter в SQLite
    // специально не нужно.
    pub rate_limits: Arc<Mutex<HashMap<String, VecDeque<i64>>>>,

    // Chat V4 realtime bus.
    //
    // Событие публикуется только после успешной записи в SQLite.
    // Каждый WebSocket фильтрует события по внутреннему user_id.
    pub chat_events: broadcast::Sender<ChatRealtimeEvent>,
    pub chat_event_sequence: Arc<AtomicU64>,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        bot_token: String,
        admin_key: String,
        admin_telegram_id: i64,
    ) -> Self {
        let (chat_events, _) = broadcast::channel(2_048);

        Self {
            db_pool,
            bot_token,
            admin_key,
            admin_telegram_id,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
            chat_events,
            chat_event_sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn publish_chat_event(
        &self,
        kind: &str,
        conversation_id: i64,
        message_id: i64,
        current_user_id: i64,
        other_user_id: i64,
    ) {
        if conversation_id <= 0
            || message_id <= 0
            || current_user_id <= 0
            || other_user_id <= 0
            || current_user_id == other_user_id
        {
            return;
        }

        let (user1_id, user2_id) = if current_user_id < other_user_id {
            (current_user_id, other_user_id)
        } else {
            (other_user_id, current_user_id)
        };

        let event_id = self
            .chat_event_sequence
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);

        let _ = self.chat_events.send(ChatRealtimeEvent {
            event_id,
            kind: kind.to_string(),
            conversation_id,
            message_id,
            user1_id,
            user2_id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ChatRealtimeEvent;

    #[test]
    fn realtime_event_scope_is_strict() {
        let event = ChatRealtimeEvent {
            event_id: 1,
            kind: "message.created".to_string(),
            conversation_id: 8,
            message_id: 42,
            user1_id: 3,
            user2_id: 9,
        };

        assert!(event.includes_user(3));
        assert!(event.includes_user(9));
        assert!(!event.includes_user(4));
    }
}
