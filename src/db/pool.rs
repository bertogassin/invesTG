use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::time::Duration;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

pub fn create_pool() -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file("data/votes.db").with_init(|conn| {
        // Эти настройки применяются к КАЖДОМУ соединению пула.
        //
        // foreign_keys является настройкой конкретного SQLite connection,
        // поэтому её обязательно включаем для каждого соединения.
        conn.pragma_update(None, "foreign_keys", "ON")?;

        // WAL позволяет нескольким читателям работать параллельно
        // и уменьшает блокировки между чтением и записью.
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Баланс надёжности и производительности для WAL.
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // Вместо мгновенного "database is locked"
        // SQLite сможет немного подождать освобождения блокировки.
        conn.busy_timeout(Duration::from_secs(5))?;

        Ok(())
    });

    Pool::builder()
        // Сервер сейчас небольшой.
        // Начинаем консервативно, без десятков соединений.
        .max_size(8)
        .min_idle(Some(1))
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
}

pub fn get_connection(pool: &DbPool) -> Result<DbConnection, r2d2::Error> {
    pool.get()
}
