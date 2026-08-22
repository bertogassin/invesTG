use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub bot_token: String,
}

impl AppState {
    pub fn new(db: Connection, bot_token: String) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            bot_token,
        }
    }
}
