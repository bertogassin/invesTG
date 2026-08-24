use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub bot_token: String,
    pub admin_key: String,
    pub admin_telegram_id: i64,
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
        }
    }
}
