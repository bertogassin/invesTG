use rusqlite::{Connection, Result};
use std::time::Duration;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("data/votes.db")?;

    // SQLite production settings.
    //
    // foreign_keys:
    //   реально включает FOREIGN KEY / ON DELETE CASCADE
    //   для этого соединения.
    //
    // WAL:
    //   уменьшает блокировки чтения/записи и лучше подходит
    //   для работающего web-приложения.
    //
    // synchronous=NORMAL:
    //   рекомендуемый баланс надёжности и скорости вместе с WAL.
    //
    // busy_timeout:
    //   SQLite ждёт освобождения блокировки вместо мгновенной
    //   ошибки "database is locked".
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(Duration::from_secs(5))?;

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

            moderation_status TEXT NOT NULL DEFAULT 'pending',
            rejection_reason TEXT NOT NULL DEFAULT '',

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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS resource_votes (
            resource_id INTEGER NOT NULL,
            client_id TEXT NOT NULL,
            score INTEGER NOT NULL CHECK(score >= 1 AND score <= 5),
            updated_at INTEGER NOT NULL DEFAULT 0,

            PRIMARY KEY (resource_id, client_id),
            FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            user_id INTEGER NOT NULL,
            resource_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            PRIMARY KEY (user_id, resource_id),
            FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_favorites_user
         ON favorites(user_id)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS resource_reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            reporter_user_id INTEGER NOT NULL,
            resource_id INTEGER NOT NULL,

            reason TEXT NOT NULL,

            status TEXT NOT NULL DEFAULT 'pending',

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            UNIQUE(reporter_user_id, resource_id),

            FOREIGN KEY (resource_id)
                REFERENCES resources(id)
                ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resource_reports_status
         ON resource_reports(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resource_reports_resource
         ON resource_reports(resource_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resource_votes_resource
         ON resource_votes(resource_id)",
        [],
    )?;

    // Telegram profile fields.
    // Ошибки duplicate column при следующих запусках намеренно игнорируются.
    let _ = conn.execute(
        "ALTER TABLE profiles
         ADD COLUMN first_name TEXT NOT NULL DEFAULT ''",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE profiles
         ADD COLUMN last_name TEXT NOT NULL DEFAULT ''",
        [],
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            user_id INTEGER NOT NULL,
            resource_id INTEGER,

            kind TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL DEFAULT '',
            message TEXT NOT NULL DEFAULT '',

            is_read INTEGER NOT NULL DEFAULT 0,

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_user_notifications_user
         ON user_notifications(user_id, is_read, created_at)",
        [],
    )?;

    // Public profile identifier.
    // Telegram ID никогда не используется в публичном URL.
    let _ = conn.execute(
        "ALTER TABLE profiles
         ADD COLUMN public_id TEXT NOT NULL DEFAULT ''",
        [],
    );

    conn.execute(
        "UPDATE profiles
         SET public_id = lower(hex(randomblob(16)))
         WHERE public_id = ''",
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_profiles_public_id
         ON profiles(public_id)
         WHERE public_id <> ''",
        [],
    )?;

    // ============================================================
    // TASK 7.22B — INTERNAL CONTACT REQUESTS
    // ============================================================

    conn.execute(
        "CREATE TABLE IF NOT EXISTS contact_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            sender_user_id INTEGER NOT NULL,
            receiver_user_id INTEGER NOT NULL,

            message TEXT NOT NULL DEFAULT '',

            status TEXT NOT NULL DEFAULT 'pending',

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            CHECK(sender_user_id <> receiver_user_id),
            CHECK(status IN ('pending','accepted','rejected')),

            UNIQUE(sender_user_id, receiver_user_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_contact_requests_receiver
         ON contact_requests(receiver_user_id, status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_contact_requests_sender
         ON contact_requests(sender_user_id, status)",
        [],
    )?;

    // ============================================================
    // TASK 7.22F — INTERNAL RESURSMAP CHAT
    // ============================================================

    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            user1_id INTEGER NOT NULL,
            user2_id INTEGER NOT NULL,

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            CHECK(user1_id <> user2_id),
            CHECK(user1_id < user2_id),

            UNIQUE(user1_id, user2_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_conversations_user1
         ON conversations(user1_id, updated_at)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_conversations_user2
         ON conversations(user2_id, updated_at)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            conversation_id INTEGER NOT NULL,
            sender_user_id INTEGER NOT NULL,

            message TEXT NOT NULL,

            is_read INTEGER NOT NULL DEFAULT 0,

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            FOREIGN KEY (conversation_id)
                REFERENCES conversations(id)
                ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_conversation
         ON messages(conversation_id, created_at, id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_unread
         ON messages(conversation_id, is_read, sender_user_id)",
        [],
    )?;

    // BOT B3.5A — additive persistent security storage.
    crate::db::security::init_security_schema(&conn)?;

    Ok(conn)
}
