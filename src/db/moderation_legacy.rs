use rusqlite::{Connection, Result};

/// Legacy moderator panel (`/app/moderator`) and audit log for group-helper actions.
pub fn init_moderation_legacy_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS moderator_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            user_id INTEGER NOT NULL,
            level INTEGER NOT NULL DEFAULT 1
                CHECK(level BETWEEN 1 AND 4),

            scope_continent_index INTEGER,
            scope_country_index INTEGER,
            scope_city_index INTEGER,

            is_active INTEGER NOT NULL DEFAULT 1
                CHECK(is_active IN (0, 1)),

            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),

            FOREIGN KEY (user_id) REFERENCES users(id)
                ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_moderator_roles_user
        ON moderator_roles (user_id, is_active);

        CREATE TABLE IF NOT EXISTS moderation_actions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            moderator_id INTEGER NOT NULL,
            action_type TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id INTEGER NOT NULL,
            details TEXT NOT NULL DEFAULT '',

            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),

            FOREIGN KEY (moderator_id) REFERENCES users(id)
                ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_moderation_actions_moderator
        ON moderation_actions (moderator_id, created_at);
        "#,
    )?;

    Ok(())
}
