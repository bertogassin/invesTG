use rusqlite::{Connection, Result};
use std::time::Duration;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open(crate::db::path::database_path())?;

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

    // ========================================================
    // Unified ResursMap accounts.
    //
    // Existing Telegram users keep their current numeric ID as
    // the internal account ID. This preserves favorites, chats,
    // notifications and contact requests without destructive
    // remapping.
    // ========================================================

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            is_active INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS auth_identities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            provider TEXT NOT NULL,
            provider_subject TEXT NOT NULL,
            email TEXT NOT NULL DEFAULT '',
            verified_at INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            UNIQUE(provider, provider_subject),
            FOREIGN KEY(user_id) REFERENCES users(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_auth_identities_user
         ON auth_identities(user_id)",
        [],
    )?;

    let user_columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(users)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns
    };

    if !user_columns.iter().any(|name| name == "telegram_id") {
        conn.execute("ALTER TABLE users ADD COLUMN telegram_id INTEGER", [])?;
    }

    conn.execute(
        "UPDATE users
         SET telegram_id = id
         WHERE telegram_id IS NULL
           AND id IN (
             SELECT user_id
             FROM auth_identities
             WHERE provider = 'telegram'
           )",
        [],
    )?;

    let identity_columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(auth_identities)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns
    };

    if !identity_columns.iter().any(|name| name == "password_hash") {
        conn.execute(
            "ALTER TABLE auth_identities
             ADD COLUMN password_hash TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_auth_email_unique
         ON auth_identities(email)
         WHERE provider = 'email' AND email <> ''",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_login_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            code_hash TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            consumed_at INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_login_codes_lookup
         ON email_login_codes(email, created_at DESC)",
        [],
    )?;

    let email_code_columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(email_login_codes)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns
    };

    if !email_code_columns.iter().any(|name| name == "purpose") {
        conn.execute(
            "ALTER TABLE email_login_codes
             ADD COLUMN purpose TEXT NOT NULL DEFAULT 'login'",
            [],
        )?;
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_public_id TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            session_hash TEXT NOT NULL UNIQUE,
            ip_address TEXT NOT NULL DEFAULT '',
            user_agent TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            last_seen_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            expires_at INTEGER NOT NULL,
            revoked_at INTEGER,
            FOREIGN KEY(user_id) REFERENCES users(id),
            CHECK(expires_at > created_at)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_user_sessions_user_active
         ON user_sessions(user_id, revoked_at, expires_at)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_user_sessions_lookup
         ON user_sessions(session_public_id, session_hash)",
        [],
    )?;

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

    // profiles.user_id becomes the stable internal account link.
    let profile_columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(profiles)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns
    };

    if !profile_columns.iter().any(|name| name == "user_id") {
        conn.execute("ALTER TABLE profiles ADD COLUMN user_id INTEGER", [])?;
    }

    // Backfill all existing Telegram profiles.
    conn.execute(
        "INSERT OR IGNORE INTO users (id)
         SELECT CAST(substr(client_id, 4) AS INTEGER)
         FROM profiles
         WHERE client_id LIKE 'tg:%'
           AND CAST(substr(client_id, 4) AS INTEGER) > 0",
        [],
    )?;

    conn.execute(
        "UPDATE profiles
         SET user_id = CAST(substr(client_id, 4) AS INTEGER)
         WHERE user_id IS NULL
           AND client_id LIKE 'tg:%'
           AND CAST(substr(client_id, 4) AS INTEGER) > 0",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO auth_identities (
            user_id,
            provider,
            provider_subject,
            verified_at
         )
         SELECT
            CAST(substr(client_id, 4) AS INTEGER),
            'telegram',
            substr(client_id, 4),
            updated_at
         FROM profiles
         WHERE client_id LIKE 'tg:%'
           AND CAST(substr(client_id, 4) AS INTEGER) > 0",
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_profiles_user_id
         ON profiles(user_id)
         WHERE user_id IS NOT NULL",
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

    // Chat V2 delivery/read timestamps.
    //
    // ALTER TABLE remains idempotent for existing production
    // databases: duplicate-column errors are intentionally ignored.
    let _ = conn.execute(
        "ALTER TABLE messages
         ADD COLUMN delivered_at INTEGER NOT NULL DEFAULT 0",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE messages
         ADD COLUMN read_at INTEGER NOT NULL DEFAULT 0",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_chat_v2_page
         ON messages(conversation_id, id DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_chat_v2_delivery
         ON messages(
             conversation_id,
             sender_user_id,
             delivered_at,
             read_at,
             id
         )",
        [],
    )?;

    // Chat V4 durable message identity and mutation schema.
    //
    // These additive migrations remain safe for existing
    // production databases.
    let _ = conn.execute(
        "ALTER TABLE messages
         ADD COLUMN reply_to_message_id INTEGER",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE messages
         ADD COLUMN edited_at INTEGER NOT NULL DEFAULT 0",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE messages
         ADD COLUMN deleted_at INTEGER NOT NULL DEFAULT 0",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE messages
         ADD COLUMN client_message_id TEXT NOT NULL DEFAULT ''",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_chat_reply
         ON messages(
             conversation_id,
             reply_to_message_id
         )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_chat_owner
         ON messages(
             conversation_id,
             sender_user_id,
             id
         )",
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS
             idx_messages_client_identity
         ON messages(
             sender_user_id,
             client_message_id
         )
         WHERE client_message_id <> ''",
        [],
    )?;

    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN attachment_kind TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN attachment_mime TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN attachment_size INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN attachment_path TEXT NOT NULL DEFAULT ''",
        [],
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_reactions (
            message_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            emoji TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            PRIMARY KEY (message_id, user_id),

            CHECK(message_id > 0),
            CHECK(user_id > 0),
            CHECK(length(emoji) BETWEEN 1 AND 8),

            FOREIGN KEY (message_id)
                REFERENCES messages(id)
                ON DELETE CASCADE,

            FOREIGN KEY (user_id)
                REFERENCES users(id)
                ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_reactions_message
         ON message_reactions(message_id, emoji)",
        [],
    )?;

    // ============================================================
    // USER BLOCKS — messenger privacy boundary
    // ============================================================

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_blocks (
            blocker_user_id INTEGER NOT NULL,
            blocked_user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            PRIMARY KEY (blocker_user_id, blocked_user_id),

            CHECK(blocker_user_id > 0),
            CHECK(blocked_user_id > 0),
            CHECK(blocker_user_id <> blocked_user_id),

            FOREIGN KEY (blocker_user_id)
                REFERENCES users(id)
                ON DELETE CASCADE,

            FOREIGN KEY (blocked_user_id)
                REFERENCES users(id)
                ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_user_blocks_blocked
         ON user_blocks(blocked_user_id, blocker_user_id)",
        [],
    )?;

    // Последняя граница безопасности: если любой участник
    // заблокировал другого, новое сообщение нельзя вставить
    // ни через API, ни через старую HTML-форму.
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS messages_reject_blocked_insert
         BEFORE INSERT ON messages
         WHEN EXISTS (
             SELECT 1
             FROM conversations AS conversation
             JOIN user_blocks AS block
               ON (
                   block.blocker_user_id = conversation.user1_id
                   AND block.blocked_user_id = conversation.user2_id
               )
               OR (
                   block.blocker_user_id = conversation.user2_id
                   AND block.blocked_user_id = conversation.user1_id
               )
             WHERE conversation.id = NEW.conversation_id
         )
         BEGIN
             SELECT RAISE(ABORT, 'user_blocked');
         END;",
    )?;

    // BOT B3.5A — additive persistent security storage.
    crate::db::security::init_security_schema(&conn)?;

    crate::db::moderation_legacy::init_moderation_legacy_schema(&conn)?;

    crate::db::promotions::init_promotion_schema(&conn)?;

    ensure_profile_profession_column(&conn)?;
    crate::db::professions::initialize(&conn)?;
    crate::db::search_fts::ensure_profile_home_city_columns(&conn)?;
    crate::db::search_fts::init_search_fts(&conn)?;

    Ok(conn)
}

/// Профессия профиля. Колонка использовалась в UI/API, но не всегда
/// существовала в старых базах — SELECT молча давал 0 профессий на главной.
pub fn ensure_profile_profession_column(conn: &Connection) -> Result<()> {
    let _ = conn.execute(
        "ALTER TABLE profiles
         ADD COLUMN category TEXT NOT NULL DEFAULT ''",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_profiles_category
         ON profiles(category)",
        [],
    )?;

    conn.execute(
        "UPDATE profiles
         SET category = (
            SELECT r.category
            FROM resources AS r
            WHERE r.client_id = profiles.client_id
              AND trim(r.category) <> ''
            ORDER BY CASE r.moderation_status
                        WHEN 'approved' THEN 0
                        ELSE 1
                     END,
                     r.updated_at DESC
            LIMIT 1
         )
         WHERE trim(category) = ''
           AND EXISTS (
             SELECT 1
             FROM resources AS r
             WHERE r.client_id = profiles.client_id
               AND trim(r.category) <> ''
           )",
        [],
    )?;

    Ok(())
}
