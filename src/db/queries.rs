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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS resources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            client_id TEXT NOT NULL DEFAULT '',

            continent_index INTEGER NOT NULL,
            country_index INTEGER NOT NULL,
            city_index INTEGER NOT NULL,

            category TEXT NOT NULL,

            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',

            contact TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',

            rating REAL NOT NULL DEFAULT 0,
            votes INTEGER NOT NULL DEFAULT 0,

            is_premium INTEGER NOT NULL DEFAULT 0,
            is_verified INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,

            created_at INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resources_location
         ON resources(continent_index, country_index, city_index)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resources_category
         ON resources(category)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resources_active
         ON resources(is_active)",
        [],
    )?;

    Ok(conn)
}
