use rusqlite::{Connection, Result};

pub fn init_promotion_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS city_publication_targets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            continent_index INTEGER NOT NULL
                CHECK(continent_index >= 0),
            country_index INTEGER NOT NULL
                CHECK(country_index >= 0),
            city_index INTEGER NOT NULL
                CHECK(city_index >= 0),

            city_name TEXT NOT NULL,
            target_name TEXT NOT NULL,

            telegram_chat_id INTEGER NOT NULL DEFAULT 0,
            telegram_url TEXT NOT NULL DEFAULT '',

            target_kind TEXT NOT NULL DEFAULT 'group'
                CHECK(target_kind IN ('group', 'channel')),

            is_active INTEGER NOT NULL DEFAULT 0
                CHECK(is_active IN (0, 1)),

            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),

            UNIQUE (
                continent_index,
                country_index,
                city_index,
                target_name
            )
        );

        CREATE INDEX IF NOT EXISTS
            idx_city_publication_targets_location
        ON city_publication_targets (
            continent_index,
            country_index,
            city_index,
            is_active
        );

        CREATE UNIQUE INDEX IF NOT EXISTS
            idx_city_publication_targets_chat
        ON city_publication_targets (telegram_chat_id)
        WHERE telegram_chat_id <> 0;

        CREATE TABLE IF NOT EXISTS resource_promotion_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            resource_id INTEGER NOT NULL,
            requester_user_id INTEGER NOT NULL,
            target_id INTEGER NOT NULL,

            client_request_id TEXT NOT NULL UNIQUE,

            status TEXT NOT NULL DEFAULT 'pending'
                CHECK(status IN (
                    'pending',
                    'approved',
                    'publishing',
                    'published',
                    'rejected',
                    'failed',
                    'cancelled'
                )),

            payment_status TEXT NOT NULL DEFAULT 'not_required'
                CHECK(payment_status IN (
                    'not_required',
                    'pending',
                    'paid',
                    'failed',
                    'refunded'
                )),

            price_minor INTEGER NOT NULL DEFAULT 0
                CHECK(price_minor >= 0),

            currency TEXT NOT NULL DEFAULT 'EUR'
                CHECK(length(currency) = 3),

            telegram_message_id INTEGER NOT NULL DEFAULT 0,
            failure_reason TEXT NOT NULL DEFAULT '',

            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            published_at INTEGER NOT NULL DEFAULT 0,

            FOREIGN KEY(resource_id)
                REFERENCES resources(id)
                ON DELETE CASCADE,

            FOREIGN KEY(requester_user_id)
                REFERENCES users(id),

            FOREIGN KEY(target_id)
                REFERENCES city_publication_targets(id)
        );

        CREATE INDEX IF NOT EXISTS
            idx_resource_promotions_owner
        ON resource_promotion_requests (
            requester_user_id,
            created_at DESC,
            id DESC
        );

        CREATE INDEX IF NOT EXISTS
            idx_resource_promotions_moderation
        ON resource_promotion_requests (
            status,
            payment_status,
            created_at,
            id
        );

        CREATE INDEX IF NOT EXISTS
            idx_resource_promotions_resource
        ON resource_promotion_requests (
            resource_id,
            created_at DESC,
            id DESC
        );

        CREATE UNIQUE INDEX IF NOT EXISTS
            idx_resource_promotions_active_unique
        ON resource_promotion_requests (
            resource_id,
            target_id
        )
        WHERE status IN (
            'pending',
            'approved',
            'publishing'
        );

        CREATE TABLE IF NOT EXISTS resource_promotion_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            promotion_request_id INTEGER NOT NULL,
            actor_user_id INTEGER NOT NULL DEFAULT 0,

            event_kind TEXT NOT NULL,
            previous_status TEXT NOT NULL DEFAULT '',
            new_status TEXT NOT NULL DEFAULT '',
            details TEXT NOT NULL DEFAULT '',

            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),

            FOREIGN KEY(promotion_request_id)
                REFERENCES resource_promotion_requests(id)
                ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS
            idx_resource_promotion_events_request
        ON resource_promotion_events (
            promotion_request_id,
            created_at,
            id
        );

        CREATE TRIGGER IF NOT EXISTS
            resource_promotion_events_no_update
        BEFORE UPDATE ON resource_promotion_events
        BEGIN
            SELECT RAISE(
                ABORT,
                'promotion_events_are_append_only'
            );
        END;

        CREATE TRIGGER IF NOT EXISTS
            resource_promotion_events_no_delete
        BEFORE DELETE ON resource_promotion_events
        BEGIN
            SELECT RAISE(
                ABORT,
                'promotion_events_are_append_only'
            );
        END;
        "#,
    )?;

    apply_promotion_flow_migrations(connection)?;

    Ok(())
}

fn add_column_if_missing(
    connection: &Connection,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> Result<()> {
    let sql = format!("PRAGMA table_info({table})");
    let mut statement = connection.prepare(&sql)?;

    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns.iter().any(|name| name == column) {
        connection.execute(alter_sql, [])?;
    }

    Ok(())
}

fn apply_promotion_flow_migrations(connection: &Connection) -> Result<()> {
    add_column_if_missing(
        connection,
        "resource_promotion_requests",
        "bot_check_status",
        "ALTER TABLE resource_promotion_requests ADD COLUMN bot_check_status TEXT NOT NULL DEFAULT 'unknown'",
    )?;

    add_column_if_missing(
        connection,
        "resource_promotion_requests",
        "bot_check_reason",
        "ALTER TABLE resource_promotion_requests ADD COLUMN bot_check_reason TEXT NOT NULL DEFAULT ''",
    )?;

    add_column_if_missing(
        connection,
        "resources",
        "listing_type",
        "ALTER TABLE resources ADD COLUMN listing_type TEXT NOT NULL DEFAULT 'general'",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("memory database");

        connection
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;

                CREATE TABLE users (
                    id INTEGER PRIMARY KEY
                );

                CREATE TABLE resources (
                    id INTEGER PRIMARY KEY
                );
                "#,
            )
            .expect("base schema");

        init_promotion_schema(&connection).expect("promotion schema");

        connection
    }

    #[test]
    fn promotion_schema_creates_required_tables() {
        let connection = test_connection();

        for table in [
            "city_publication_targets",
            "resource_promotion_requests",
            "resource_promotion_events",
        ] {
            let exists: i64 = connection
                .query_row(
                    "SELECT COUNT(*)
                     FROM sqlite_master
                     WHERE type = 'table'
                       AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("table lookup");

            assert_eq!(exists, 1, "{table}");
        }
    }

    #[test]
    fn duplicate_active_promotion_is_rejected() {
        let connection = test_connection();

        connection
            .execute("INSERT INTO users (id) VALUES (1)", [])
            .expect("user");

        connection
            .execute("INSERT INTO resources (id) VALUES (10)", [])
            .expect("resource");

        connection
            .execute(
                "INSERT INTO city_publication_targets (
                    continent_index,
                    country_index,
                    city_index,
                    city_name,
                    target_name
                 )
                 VALUES (0, 0, 0, 'Ницца', 'Основная группа')",
                [],
            )
            .expect("target");

        connection
            .execute(
                "INSERT INTO resource_promotion_requests (
                    resource_id,
                    requester_user_id,
                    target_id,
                    client_request_id
                 )
                 VALUES (10, 1, 1, 'request-one')",
                [],
            )
            .expect("first request");

        let duplicate = connection.execute(
            "INSERT INTO resource_promotion_requests (
                resource_id,
                requester_user_id,
                target_id,
                client_request_id
             )
             VALUES (10, 1, 1, 'request-two')",
            [],
        );

        assert!(duplicate.is_err());
    }

    #[test]
    fn promotion_event_log_is_append_only() {
        let connection = test_connection();

        connection
            .execute("INSERT INTO users (id) VALUES (1)", [])
            .expect("user");

        connection
            .execute("INSERT INTO resources (id) VALUES (10)", [])
            .expect("resource");

        connection
            .execute(
                "INSERT INTO city_publication_targets (
                    continent_index,
                    country_index,
                    city_index,
                    city_name,
                    target_name
                 )
                 VALUES (0, 0, 0, 'Ницца', 'Основная группа')",
                [],
            )
            .expect("target");

        connection
            .execute(
                "INSERT INTO resource_promotion_requests (
                    resource_id,
                    requester_user_id,
                    target_id,
                    client_request_id
                 )
                 VALUES (10, 1, 1, 'request-one')",
                [],
            )
            .expect("request");

        connection
            .execute(
                "INSERT INTO resource_promotion_events (
                    promotion_request_id,
                    actor_user_id,
                    event_kind,
                    new_status
                 )
                 VALUES (1, 1, 'created', 'pending')",
                [],
            )
            .expect("event");

        assert!(connection
            .execute(
                "UPDATE resource_promotion_events
                     SET details = 'changed'
                     WHERE id = 1",
                [],
            )
            .is_err());

        assert!(connection
            .execute(
                "DELETE FROM resource_promotion_events
                     WHERE id = 1",
                [],
            )
            .is_err());
    }
}
