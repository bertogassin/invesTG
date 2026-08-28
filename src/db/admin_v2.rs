use rusqlite::{params, Connection, Transaction, TransactionBehavior};
use std::path::Path;
use std::time::Duration;

const ADMIN_V2_FOUNDATION_VERSION: i64 = 1;
pub const ADMIN_V2_SCHEMA_VERSION: i64 = 2;
pub const INITIAL_OWNER_USER_ID: i64 = 8_775_621_311;

const ADMIN_V2_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS geographic_scopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope_type TEXT NOT NULL
        CHECK(scope_type IN ('world','continent','country','city','group')),
    parent_scope_id INTEGER,
    external_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1
        CHECK(is_active IN (0,1)),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(parent_scope_id) REFERENCES geographic_scopes(id),
    UNIQUE(scope_type, external_key)
);

CREATE TABLE IF NOT EXISTS permission_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    permission_key TEXT NOT NULL UNIQUE,
    bit_position INTEGER NOT NULL UNIQUE
        CHECK(bit_position BETWEEN 0 AND 62),
    minimum_level INTEGER NOT NULL
        CHECK(minimum_level BETWEEN 1 AND 5),
    description TEXT NOT NULL,
    is_critical INTEGER NOT NULL DEFAULT 0
        CHECK(is_critical IN (0,1)),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS admin_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    role_level INTEGER NOT NULL
        CHECK(role_level BETWEEN 1 AND 5),
    scope_type TEXT NOT NULL
        CHECK(scope_type IN ('world','continent','country','city','group')),
    scope_id INTEGER NOT NULL,
    permission_mask INTEGER NOT NULL DEFAULT 0
        CHECK(permission_mask >= 0),
    category_restrictions TEXT NOT NULL DEFAULT '[]',
    valid_from INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    valid_until INTEGER,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK(status IN ('pending','active','suspended','revoked','expired')),
    assigned_by_user_id INTEGER NOT NULL,
    assignment_reason TEXT NOT NULL,
    last_change_reason TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(scope_id) REFERENCES geographic_scopes(id),
    FOREIGN KEY(assigned_by_user_id) REFERENCES users(id),
    CHECK(valid_until IS NULL OR valid_until > valid_from),
    UNIQUE(user_id, role_level, scope_type, scope_id)
);

CREATE TABLE IF NOT EXISTS admin_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    assignment_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    is_granted INTEGER NOT NULL
        CHECK(is_granted IN (0,1)),
    granted_by_user_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(assignment_id) REFERENCES admin_assignments(id),
    FOREIGN KEY(permission_id) REFERENCES permission_definitions(id),
    FOREIGN KEY(granted_by_user_id) REFERENCES users(id),
    UNIQUE(assignment_id, permission_id)
);

CREATE TABLE IF NOT EXISTS admin_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_public_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    assignment_id INTEGER NOT NULL,
    session_hash TEXT NOT NULL UNIQUE,
    ip_address TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    device_label TEXT NOT NULL DEFAULT '',
    two_factor_verified INTEGER NOT NULL DEFAULT 0
        CHECK(two_factor_verified IN (0,1)),
    reauthenticated_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    last_seen_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER,
    revoked_by_user_id INTEGER,
    revoke_reason TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(assignment_id) REFERENCES admin_assignments(id),
    FOREIGN KEY(revoked_by_user_id) REFERENCES users(id),
    CHECK(expires_at > created_at)
);

CREATE TABLE IF NOT EXISTS admin_security_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    assignment_id INTEGER,
    session_public_id TEXT NOT NULL DEFAULT '',
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL
        CHECK(severity IN ('info','warning','high','critical')),
    ip_address TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    details TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(assignment_id) REFERENCES admin_assignments(id)
);

CREATE TABLE IF NOT EXISTS admin_action_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_public_id TEXT NOT NULL UNIQUE,
    administrator_user_id INTEGER NOT NULL,
    assignment_id INTEGER,
    administrator_level INTEGER NOT NULL
        CHECK(administrator_level BETWEEN 1 AND 5),
    scope_type TEXT NOT NULL,
    scope_id INTEGER,
    action_type TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL DEFAULT '',
    old_state TEXT NOT NULL DEFAULT '{}',
    new_state TEXT NOT NULL DEFAULT '{}',
    reason TEXT NOT NULL,
    ip_address TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    session_public_id TEXT NOT NULL DEFAULT '',
    result TEXT NOT NULL
        CHECK(result IN ('success','denied','failed')),
    approvers TEXT NOT NULL DEFAULT '[]',
    previous_hash TEXT NOT NULL,
    event_hash TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(administrator_user_id) REFERENCES users(id),
    FOREIGN KEY(assignment_id) REFERENCES admin_assignments(id)
);

CREATE TABLE IF NOT EXISTS admin_additional_scopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    assignment_id INTEGER NOT NULL,
    scope_id INTEGER NOT NULL,
    assigned_by_user_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(assignment_id) REFERENCES admin_assignments(id),
    FOREIGN KEY(scope_id) REFERENCES geographic_scopes(id),
    FOREIGN KEY(assigned_by_user_id) REFERENCES users(id),
    UNIQUE(assignment_id, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_admin_assignments_user_active
    ON admin_assignments(user_id, status, valid_until);

CREATE INDEX IF NOT EXISTS idx_admin_assignments_scope
    ON admin_assignments(scope_type, scope_id, role_level, status);

CREATE UNIQUE INDEX IF NOT EXISTS idx_single_active_global_owner
    ON admin_assignments(role_level)
    WHERE role_level = 5
      AND scope_type = 'world'
      AND status = 'active';

CREATE INDEX IF NOT EXISTS idx_admin_sessions_user_active
    ON admin_sessions(user_id, revoked_at, expires_at);

CREATE INDEX IF NOT EXISTS idx_admin_security_events_user
    ON admin_security_events(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_admin_action_audit_actor
    ON admin_action_audit(administrator_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_admin_action_audit_target
    ON admin_action_audit(target_type, target_id, created_at DESC);

CREATE TRIGGER IF NOT EXISTS admin_action_audit_no_update
BEFORE UPDATE ON admin_action_audit
BEGIN
    SELECT RAISE(ABORT, 'admin_action_audit is append-only');
END;

CREATE TRIGGER IF NOT EXISTS admin_action_audit_no_delete
BEFORE DELETE ON admin_action_audit
BEGIN
    SELECT RAISE(ABORT, 'admin_action_audit is append-only');
END;
"#;

const ADMIN_V2_SECURITY_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS admin_reauth_challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    challenge_public_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    assignment_id INTEGER NOT NULL,
    session_public_id TEXT NOT NULL,
    purpose TEXT NOT NULL
        CHECK(purpose IN (
            'owner_step_up',
            'critical_admin_action',
            'financial_operation',
            'emergency_action'
        )),
    delivery_channel TEXT NOT NULL
        CHECK(delivery_channel IN ('email')),
    destination_hash TEXT NOT NULL,
    code_hash TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0
        CHECK(attempts BETWEEN 0 AND 5),
    expires_at INTEGER NOT NULL,
    consumed_at INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(assignment_id) REFERENCES admin_assignments(id),
    CHECK(expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_admin_reauth_session
    ON admin_reauth_challenges(
        session_public_id,
        purpose,
        created_at DESC
    );

CREATE INDEX IF NOT EXISTS idx_admin_reauth_user_recent
    ON admin_reauth_challenges(
        user_id,
        created_at DESC
    );
"#;

const PERMISSIONS: &[(&str, i64, i64, &str, i64)] = &[
    ("moderation.review", 0, 1, "Проверка материалов", 0),
    ("moderation.reject", 1, 1, "Отклонение материалов", 0),
    (
        "moderation.escalate",
        2,
        1,
        "Передача дела старшему уровню",
        0,
    ),
    ("complaints.review", 3, 1, "Рассмотрение жалоб", 0),
    ("groups.manage", 4, 2, "Управление группами", 0),
    ("assistants.manage", 5, 2, "Управление помощниками", 1),
    ("premium.manage", 6, 2, "Управление Premium", 1),
    ("cities.manage", 7, 3, "Управление городами", 1),
    (
        "city_admins.manage",
        8,
        3,
        "Управление городскими администраторами",
        1,
    ),
    (
        "country_rules.manage",
        9,
        3,
        "Управление правилами страны",
        1,
    ),
    ("countries.manage", 10, 4, "Управление странами", 1),
    (
        "country_admins.manage",
        11,
        4,
        "Управление администраторами стран",
        1,
    ),
    (
        "regional_freeze.execute",
        12,
        4,
        "Региональная заморозка",
        1,
    ),
    (
        "global.admins.manage",
        13,
        5,
        "Глобальное управление администраторами",
        1,
    ),
    (
        "global.users.manage",
        14,
        5,
        "Глобальное управление пользователями",
        1,
    ),
    ("global.settings.manage", 15, 5, "Глобальные настройки", 1),
    (
        "security.sessions.revoke",
        16,
        5,
        "Отзыв административных сессий",
        1,
    ),
    (
        "security.audit.read",
        17,
        4,
        "Просмотр защищённого аудита",
        1,
    ),
    ("finance.read", 18, 3, "Просмотр разрешённой отчётности", 1),
    (
        "finance.operations.propose",
        19,
        5,
        "Создание финансовой операции",
        1,
    ),
    (
        "finance.operations.approve",
        20,
        5,
        "Подтверждение финансовой операции",
        1,
    ),
    ("emergency.execute", 21, 5, "Аварийные действия", 1),
    ("infrastructure.read", 22, 5, "Состояние инфраструктуры", 1),
    ("integrations.manage", 23, 5, "Управление интеграциями", 1),
];

pub fn initialize() -> rusqlite::Result<()> {
    let database_path = Path::new("data/votes.db");
    let mut connection = Connection::open(database_path)?;

    connection.busy_timeout(Duration::from_secs(10))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;

    initialize_connection(&mut connection, INITIAL_OWNER_USER_ID)
}

fn initialize_connection(connection: &mut Connection, owner_user_id: i64) -> rusqlite::Result<()> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

    transaction.execute_batch(ADMIN_V2_SCHEMA)?;
    seed_permissions(&transaction)?;
    seed_world_scope(&transaction)?;
    seed_initial_owner(&transaction, owner_user_id)?;

    transaction.execute(
        "INSERT OR IGNORE INTO schema_migrations (version, name)
         VALUES (?1, 'admin_v2_foundation')",
        params![ADMIN_V2_FOUNDATION_VERSION],
    )?;

    transaction.execute_batch(ADMIN_V2_SECURITY_SCHEMA)?;

    transaction.execute(
        "INSERT OR IGNORE INTO schema_migrations (version, name)
         VALUES (?1, 'admin_v2_owner_step_up')",
        params![ADMIN_V2_SCHEMA_VERSION],
    )?;

    transaction.pragma_update(None, "user_version", ADMIN_V2_SCHEMA_VERSION)?;

    transaction.commit()
}

fn seed_permissions(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    for (key, bit, level, description, critical) in PERMISSIONS {
        transaction.execute(
            "INSERT INTO permission_definitions (
                permission_key,
                bit_position,
                minimum_level,
                description,
                is_critical
             )
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(permission_key) DO UPDATE SET
                bit_position = excluded.bit_position,
                minimum_level = excluded.minimum_level,
                description = excluded.description,
                is_critical = excluded.is_critical",
            params![key, bit, level, description, critical],
        )?;
    }

    Ok(())
}

fn seed_world_scope(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO geographic_scopes (
            scope_type,
            parent_scope_id,
            external_key,
            display_name,
            is_active
         )
         VALUES ('world', NULL, 'world', 'Весь мир', 1)
         ON CONFLICT(scope_type, external_key) DO UPDATE SET
            display_name = excluded.display_name,
            is_active = 1,
            updated_at = strftime('%s','now')",
        [],
    )?;

    Ok(())
}

fn seed_initial_owner(transaction: &Transaction<'_>, owner_user_id: i64) -> rusqlite::Result<()> {
    let owner_exists: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM users WHERE id = ?1 AND is_active = 1",
        params![owner_user_id],
        |row| row.get(0),
    )?;

    if owner_exists != 1 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let world_scope_id: i64 = transaction.query_row(
        "SELECT id
         FROM geographic_scopes
         WHERE scope_type = 'world'
           AND external_key = 'world'
           AND is_active = 1",
        [],
        |row| row.get(0),
    )?;

    transaction.execute(
        "INSERT INTO admin_assignments (
            user_id,
            role_level,
            scope_type,
            scope_id,
            permission_mask,
            category_restrictions,
            valid_from,
            valid_until,
            status,
            assigned_by_user_id,
            assignment_reason,
            last_change_reason
         )
         VALUES (
            ?1,
            5,
            'world',
            ?2,
            9223372036854775807,
            '[]',
            strftime('%s','now'),
            NULL,
            'active',
            ?1,
            'Первоначальный владелец ResursMap',
            'Безопасная миграция Admin V2'
         )
         ON CONFLICT(user_id, role_level, scope_type, scope_id)
         DO UPDATE SET
            permission_mask = excluded.permission_mask,
            status = 'active',
            valid_until = NULL,
            updated_at = strftime('%s','now'),
            last_change_reason = 'Повторная проверка Admin V2'",
        params![owner_user_id, world_scope_id],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory db");

        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE users (
                    id INTEGER PRIMARY KEY,
                    is_active INTEGER NOT NULL DEFAULT 1
                 );
                 INSERT INTO users (id, is_active)
                 VALUES (8775621311, 1);",
            )
            .expect("base schema");

        connection
    }

    #[test]
    fn migration_creates_single_level_five_owner() {
        let mut connection = test_connection();

        initialize_connection(&mut connection, INITIAL_OWNER_USER_ID).expect("admin migration");

        let owner: (i64, i64, String, String) = connection
            .query_row(
                "SELECT
                    user_id,
                    role_level,
                    scope_type,
                    status
                 FROM admin_assignments",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("owner");

        assert_eq!(owner.0, INITIAL_OWNER_USER_ID);
        assert_eq!(owner.1, 5);
        assert_eq!(owner.2, "world");
        assert_eq!(owner.3, "active");
    }

    #[test]
    fn migration_is_idempotent() {
        let mut connection = test_connection();

        initialize_connection(&mut connection, INITIAL_OWNER_USER_ID).expect("first migration");

        initialize_connection(&mut connection, INITIAL_OWNER_USER_ID).expect("second migration");

        let assignments: i64 = connection
            .query_row("SELECT COUNT(*) FROM admin_assignments", [], |row| {
                row.get(0)
            })
            .expect("count");

        assert_eq!(assignments, 1);
    }

    #[test]
    fn audit_rejects_updates_and_deletes() {
        let mut connection = test_connection();

        initialize_connection(&mut connection, INITIAL_OWNER_USER_ID).expect("migration");

        let assignment_id: i64 = connection
            .query_row("SELECT id FROM admin_assignments", [], |row| row.get(0))
            .expect("assignment");

        connection
            .execute(
                "INSERT INTO admin_action_audit (
                    event_public_id,
                    administrator_user_id,
                    assignment_id,
                    administrator_level,
                    scope_type,
                    scope_id,
                    action_type,
                    target_type,
                    target_id,
                    reason,
                    result,
                    previous_hash,
                    event_hash
                 )
                 VALUES (
                    'event-1',
                    ?1,
                    ?2,
                    5,
                    'world',
                    1,
                    'owner_initialized',
                    'admin_assignment',
                    ?3,
                    'test',
                    'success',
                    'genesis',
                    'hash-1'
                 )",
                params![
                    INITIAL_OWNER_USER_ID,
                    assignment_id,
                    assignment_id.to_string()
                ],
            )
            .expect("audit insert");

        assert!(connection
            .execute(
                "UPDATE admin_action_audit
                     SET reason = 'changed'
                     WHERE event_public_id = 'event-1'",
                [],
            )
            .is_err());

        assert!(connection
            .execute(
                "DELETE FROM admin_action_audit
                     WHERE event_public_id = 'event-1'",
                [],
            )
            .is_err());
    }

    #[test]
    fn inactive_owner_is_rejected() {
        let mut connection = test_connection();

        connection
            .execute(
                "UPDATE users SET is_active = 0
                 WHERE id = ?1",
                params![INITIAL_OWNER_USER_ID],
            )
            .expect("deactivate");

        assert!(initialize_connection(&mut connection, INITIAL_OWNER_USER_ID).is_err());
    }

    #[test]
    fn migration_creates_reauthentication_challenges() {
        let mut connection = test_connection();

        initialize_connection(&mut connection, INITIAL_OWNER_USER_ID)
            .expect("admin security migration");

        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*)
                 FROM sqlite_master
                 WHERE type = 'table'
                   AND name = 'admin_reauth_challenges'",
                [],
                |row| row.get(0),
            )
            .expect("table count");

        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("schema version");

        assert_eq!(table_count, 1);
        assert_eq!(version, ADMIN_V2_SCHEMA_VERSION);
    }
}
