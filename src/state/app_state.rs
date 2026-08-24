use crate::db::pool::DbPool;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;

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
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        bot_token: String,
        admin_key: String,
        admin_telegram_id: i64,
    ) -> Self {
        Self {
            db_pool,
            bot_token,
            admin_key,
            admin_telegram_id,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
