use rusqlite::Connection;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
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
}

impl AppState {
    pub fn new(
        db: Connection,
        bot_token: String,
        admin_key: String,
        admin_telegram_id: i64,
    ) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            bot_token,
            admin_key,
            admin_telegram_id,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
