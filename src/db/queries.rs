use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("data/votes.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS profiles (
            client_id TEXT PRIMARY KEY,
            username TEXT NOT NULL DEFAULT '',
            open_contact INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT 0,
            intent_text TEXT NOT NULL DEFAULT '',
            intent_until INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;
    Ok(conn)
}
