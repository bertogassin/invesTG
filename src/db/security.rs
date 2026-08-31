use rusqlite::Connection;

pub fn init_security_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS security_risk_state (
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,

            risk_score INTEGER NOT NULL DEFAULT 0
                CHECK(risk_score >= 0),

            last_decay_at INTEGER NOT NULL DEFAULT 0,
            last_warning_at INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            PRIMARY KEY (chat_id, user_id)
        );

        CREATE TABLE IF NOT EXISTS security_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,

            reason TEXT NOT NULL,

            weight INTEGER NOT NULL
                CHECK(weight >= 0),

            risk_score INTEGER NOT NULL DEFAULT 0
                CHECK(risk_score >= 0),

            message_id INTEGER NOT NULL DEFAULT 0,

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        CREATE INDEX IF NOT EXISTS idx_security_events_user
        ON security_events (
            chat_id,
            user_id,
            created_at DESC,
            id DESC
        );

        CREATE INDEX IF NOT EXISTS idx_security_events_created
        ON security_events (
            created_at DESC,
            id DESC
        );

        CREATE INDEX IF NOT EXISTS idx_security_risk_score
        ON security_risk_state (
            risk_score DESC,
            updated_at DESC
        );

        CREATE TABLE IF NOT EXISTS security_moderation_state (
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,

            critical_count INTEGER NOT NULL DEFAULT 0
                CHECK(critical_count >= 0),

            last_critical_at INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

            PRIMARY KEY (chat_id, user_id)
        );

        CREATE INDEX IF NOT EXISTS idx_security_moderation_critical
        ON security_moderation_state (
            critical_count DESC,
            updated_at DESC
        );


        CREATE TABLE IF NOT EXISTS security_moderation_audit (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            message_id INTEGER NOT NULL DEFAULT 0,

            action TEXT NOT NULL,
            reason TEXT NOT NULL,

            risk_score INTEGER NOT NULL DEFAULT 0
                CHECK(risk_score >= 0),

            risk_level TEXT NOT NULL,

            critical_occurrence INTEGER NOT NULL DEFAULT 0
                CHECK(critical_occurrence >= 0),

            mute_seconds INTEGER NOT NULL DEFAULT 0
                CHECK(mute_seconds >= 0),

            success INTEGER NOT NULL DEFAULT 1
                CHECK(success IN (0, 1)),

            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        CREATE INDEX IF NOT EXISTS idx_security_moderation_audit_user
        ON security_moderation_audit (
            chat_id,
            user_id,
            created_at DESC,
            id DESC
        );

        CREATE INDEX IF NOT EXISTS idx_security_moderation_audit_action
        ON security_moderation_audit (
            action,
            created_at DESC,
            id DESC
        );

        CREATE INDEX IF NOT EXISTS idx_security_moderation_audit_created
        ON security_moderation_audit (
            created_at DESC,
            id DESC
        );
        ",
    )
}

#[cfg(feature = "telegram-bot")]
mod bot_moderation {
use crate::db::pool::{get_connection, DbPool};
use rusqlite::{params, OptionalExtension};

pub const DEFAULT_WARNING_COOLDOWN_SECONDS: i64 = 60;
pub const SECURITY_RISK_HISTORY_SECONDS: i64 = 3600;
pub const SECURITY_RISK_DECAY_SECONDS: i64 = 600;

pub type SecurityDbResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CriticalEscalation {
    pub occurrence: u32,
    pub mute_seconds: i64,
}

pub fn mute_seconds_for_critical(occurrence: u32) -> i64 {
    match occurrence {
        0 | 1 => 10 * 60,
        2 => 60 * 60,
        _ => 24 * 60 * 60,
    }
}

#[derive(Debug, Clone)]
pub struct NewModerationAuditEvent<'a> {
    pub chat_id: i64,
    pub user_id: i64,
    pub message_id: i32,
    pub action: &'a str,
    pub reason: &'a str,
    pub risk_score: u32,
    pub risk_level: &'a str,
    pub critical_occurrence: u32,
    pub mute_seconds: i64,
    pub success: bool,
    pub created_at: i64,
}

pub fn update_persistent_risk(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
    reason: &str,
    weight: u32,
    message_id: i32,
    now: i64,
) -> SecurityDbResult<u32> {
    let mut conn = get_connection(pool)?;
    let tx = conn.transaction()?;

    // Same history horizon as the original B3.2 RAM engine.
    tx.execute(
        "
        DELETE FROM security_events
        WHERE chat_id = ?1
          AND user_id = ?2
          AND created_at < ?3
        ",
        params![chat_id, user_id, now - SECURITY_RISK_HISTORY_SECONDS],
    )?;

    let previous_decay_at: Option<i64> = tx
        .query_row(
            "
            SELECT last_decay_at
            FROM security_risk_state
            WHERE chat_id = ?1
              AND user_id = ?2
            ",
            params![chat_id, user_id],
            |row| row.get(0),
        )
        .optional()?;

    match previous_decay_at {
        None => {
            tx.execute(
                "
                INSERT INTO security_risk_state (
                    chat_id,
                    user_id,
                    risk_score,
                    last_decay_at,
                    last_warning_at,
                    updated_at
                )
                VALUES (?1, ?2, 0, ?3, 0, ?3)
                ",
                params![chat_id, user_id, now],
            )?;
        }

        Some(last_decay_at) if now.saturating_sub(last_decay_at) >= SECURITY_RISK_DECAY_SECONDS => {
            // Preserve B3.2 semantics:
            // after the decay interval, remove one oldest
            // still-active violation before adding the new one.
            let oldest_id: Option<i64> = tx
                .query_row(
                    "
                    SELECT id
                    FROM security_events
                    WHERE chat_id = ?1
                      AND user_id = ?2
                    ORDER BY created_at ASC, id ASC
                    LIMIT 1
                    ",
                    params![chat_id, user_id],
                    |row| row.get(0),
                )
                .optional()?;

            if let Some(oldest_id) = oldest_id {
                tx.execute(
                    "DELETE FROM security_events WHERE id = ?1",
                    params![oldest_id],
                )?;
            }

            tx.execute(
                "
                UPDATE security_risk_state
                SET last_decay_at = ?3,
                    updated_at = ?3
                WHERE chat_id = ?1
                  AND user_id = ?2
                ",
                params![chat_id, user_id, now],
            )?;
        }

        Some(_) => {}
    }

    tx.execute(
        "
        INSERT INTO security_events (
            chat_id,
            user_id,
            reason,
            weight,
            risk_score,
            message_id,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)
        ",
        params![chat_id, user_id, reason, i64::from(weight), message_id, now],
    )?;

    let event_id = tx.last_insert_rowid();

    let score_i64: i64 = tx.query_row(
        "
        SELECT COALESCE(SUM(weight), 0)
        FROM security_events
        WHERE chat_id = ?1
          AND user_id = ?2
          AND created_at >= ?3
        ",
        params![chat_id, user_id, now - SECURITY_RISK_HISTORY_SECONDS],
        |row| row.get(0),
    )?;

    let score = u32::try_from(score_i64).unwrap_or(u32::MAX);

    tx.execute(
        "
        UPDATE security_risk_state
        SET risk_score = ?3,
            updated_at = ?4
        WHERE chat_id = ?1
          AND user_id = ?2
        ",
        params![chat_id, user_id, i64::from(score), now],
    )?;

    tx.execute(
        "
        UPDATE security_events
        SET risk_score = ?2
        WHERE id = ?1
        ",
        params![event_id, i64::from(score)],
    )?;

    tx.commit()?;

    Ok(score)
}

pub fn record_moderation_audit(
    pool: &DbPool,
    event: &NewModerationAuditEvent<'_>,
) -> SecurityDbResult<i64> {
    let conn = get_connection(pool)?;

    conn.execute(
        "
        INSERT INTO security_moderation_audit (
            chat_id,
            user_id,
            message_id,
            action,
            reason,
            risk_score,
            risk_level,
            critical_occurrence,
            mute_seconds,
            success,
            created_at
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8, ?9, ?10, ?11
        )
        ",
        params![
            event.chat_id,
            event.user_id,
            event.message_id,
            event.action,
            event.reason,
            i64::from(event.risk_score),
            event.risk_level,
            i64::from(event.critical_occurrence),
            event.mute_seconds,
            if event.success { 1_i64 } else { 0_i64 },
            event.created_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn claim_critical_escalation(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
    now: i64,
) -> SecurityDbResult<CriticalEscalation> {
    let mut conn = get_connection(pool)?;
    let tx = conn.transaction()?;

    tx.execute(
        "
        INSERT INTO security_moderation_state (
            chat_id,
            user_id,
            critical_count,
            last_critical_at,
            updated_at
        )
        VALUES (?1, ?2, 1, ?3, ?3)

        ON CONFLICT(chat_id, user_id)
        DO UPDATE SET
            critical_count =
                security_moderation_state.critical_count + 1,
            last_critical_at = excluded.last_critical_at,
            updated_at = excluded.updated_at
        ",
        params![chat_id, user_id, now],
    )?;

    let count_i64: i64 = tx.query_row(
        "
        SELECT critical_count
        FROM security_moderation_state
        WHERE chat_id = ?1
          AND user_id = ?2
        ",
        params![chat_id, user_id],
        |row| row.get(0),
    )?;

    tx.commit()?;

    let occurrence = u32::try_from(count_i64).unwrap_or(u32::MAX);

    Ok(CriticalEscalation {
        occurrence,
        mute_seconds: mute_seconds_for_critical(occurrence),
    })
}

pub fn claim_warning_slot(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
    now: i64,
    cooldown_seconds: i64,
) -> SecurityDbResult<bool> {
    let conn = get_connection(pool)?;

    let changed = conn.execute(
        "
        UPDATE security_risk_state
        SET
            last_warning_at = ?3,
            updated_at = ?3
        WHERE chat_id = ?1
          AND user_id = ?2
          AND (
              last_warning_at = 0
              OR last_warning_at <= ?4
          )
        ",
        params![chat_id, user_id, now, now - cooldown_seconds,],
    )?;

    Ok(changed == 1)
}

}

#[cfg(feature = "telegram-bot")]
pub use bot_moderation::*;

#[cfg(test)]
mod schema_tests {
    use super::init_security_schema;
    use crate::db::pool::get_connection;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;

    fn test_pool() -> crate::db::pool::DbPool {
        let manager = SqliteConnectionManager::memory().with_init(|conn| {
            init_security_schema(conn)?;
            Ok(())
        });

        Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("test sqlite pool")
    }

    #[test]
    fn schema_creates_security_tables() {
        let pool = test_pool();
        let conn = get_connection(&pool).expect("connection");

        let count: i64 = conn
            .query_row(
                "
                SELECT COUNT(*)
                FROM sqlite_master
                WHERE type = 'table'
                  AND name IN (
                      'security_events',
                      'security_risk_state'
                  )
                ",
                [],
                |row| row.get(0),
            )
            .expect("table count");

        assert_eq!(count, 2);
    }
}

#[cfg(all(test, feature = "telegram-bot"))]
mod tests {
    use super::*;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;

    fn test_pool() -> DbPool {
        let manager = SqliteConnectionManager::memory().with_init(|conn| {
            super::init_security_schema(conn)?;
            Ok(())
        });

        Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("test sqlite pool")
    }

    #[test]
    fn persistent_risk_accumulates() {
        let pool = test_pool();

        let first = update_persistent_risk(&pool, -300001, 3001, "SuspiciousLink", 4, 1, 1000)
            .expect("first persistent risk");

        assert_eq!(first, 4);

        let second = update_persistent_risk(&pool, -300001, 3001, "SuspiciousLink", 4, 2, 1001)
            .expect("second persistent risk");

        assert_eq!(second, 8);

        let conn = get_connection(&pool).expect("connection");

        let stored: i64 = conn
            .query_row(
                "
                SELECT risk_score
                FROM security_risk_state
                WHERE chat_id = ?1
                  AND user_id = ?2
                ",
                params![-300001_i64, 3001_i64],
                |row| row.get(0),
            )
            .expect("stored risk");

        assert_eq!(stored, 8);
    }

    #[test]
    fn persistent_risk_decay_matches_b3_2() {
        let pool = test_pool();

        let first = update_persistent_risk(&pool, -300002, 3002, "SuspiciousLink", 4, 1, 1000)
            .expect("first");

        assert_eq!(first, 4);

        let second = update_persistent_risk(&pool, -300002, 3002, "DuplicateMessage", 1, 2, 1601)
            .expect("second");

        // Old +4 event decays, new +1 remains.
        assert_eq!(second, 1);
    }

    #[test]
    fn persistent_risk_survives_pool_recreation() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();

        let path = std::env::temp_dir().join(format!("resursmap-security-{unique}.db"));

        {
            let manager = SqliteConnectionManager::file(&path).with_init(|conn| {
                init_security_schema(conn)?;
                Ok(())
            });

            let pool = Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("first pool");

            let score = update_persistent_risk(&pool, -300003, 3003, "SuspiciousLink", 4, 1, 2000)
                .expect("first write");

            assert_eq!(score, 4);
        }

        {
            let manager = SqliteConnectionManager::file(&path).with_init(|conn| {
                init_security_schema(conn)?;
                Ok(())
            });

            let pool = Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("second pool");

            let conn = get_connection(&pool).expect("connection");

            let stored: i64 = conn
                .query_row(
                    "
                    SELECT risk_score
                    FROM security_risk_state
                    WHERE chat_id = ?1
                      AND user_id = ?2
                    ",
                    params![-300003_i64, 3003_i64],
                    |row| row.get(0),
                )
                .expect("persistent state");

            assert_eq!(stored, 4);

            drop(conn);

            let score = update_persistent_risk(&pool, -300003, 3003, "SuspiciousLink", 4, 2, 2001)
                .expect("second write");

            assert_eq!(score, 8);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("db-wal"));
        let _ = fs::remove_file(path.with_extension("db-shm"));
    }

    #[test]
    fn warning_cooldown_is_atomic() {
        let pool = test_pool();

        update_persistent_risk(&pool, -100777, 123, "SuspiciousLink", 4, 1, 1000)
            .expect("seed risk");

        assert!(
            claim_warning_slot(&pool, -100777, 123, 2000, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("first claim")
        );

        assert!(
            !claim_warning_slot(&pool, -100777, 123, 2020, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("second claim")
        );

        assert!(
            claim_warning_slot(&pool, -100777, 123, 2061, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("third claim")
        );
    }

    #[test]
    fn warning_cooldown_is_scoped_per_user_and_chat() {
        let pool = test_pool();

        update_persistent_risk(&pool, -1001, 101, "SuspiciousLink", 4, 1, 1000)
            .expect("seed first");

        update_persistent_risk(&pool, -1001, 202, "SuspiciousLink", 4, 2, 1000)
            .expect("seed second");

        update_persistent_risk(&pool, -2002, 101, "SuspiciousLink", 4, 3, 1000)
            .expect("seed third");

        assert!(
            claim_warning_slot(&pool, -1001, 101, 2000, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("first user")
        );

        assert!(
            !claim_warning_slot(&pool, -1001, 101, 2020, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("same user cooldown")
        );

        assert!(
            claim_warning_slot(&pool, -1001, 202, 2020, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("different user")
        );

        assert!(
            claim_warning_slot(&pool, -2002, 101, 2020, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                .expect("different chat")
        );
    }

    #[test]
    fn warning_cooldown_survives_pool_recreation() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();

        let path = std::env::temp_dir().join(format!("resursmap-security-cooldown-{unique}.db"));

        let make_pool = || {
            let manager = SqliteConnectionManager::file(&path).with_init(|conn| {
                init_security_schema(conn)?;
                Ok(())
            });

            Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("file sqlite pool")
        };

        {
            let pool = make_pool();

            update_persistent_risk(&pool, -3003, 303, "SuspiciousLink", 4, 1, 1000)
                .expect("seed risk");

            assert!(
                claim_warning_slot(&pool, -3003, 303, 2000, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                    .expect("first claim")
            );
        }

        {
            let pool = make_pool();

            assert!(
                !claim_warning_slot(&pool, -3003, 303, 2020, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                    .expect("claim after pool recreation")
            );

            assert!(
                claim_warning_slot(&pool, -3003, 303, 2061, DEFAULT_WARNING_COOLDOWN_SECONDS,)
                    .expect("claim after cooldown")
            );
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn critical_escalation_sequence_is_correct() {
        let pool = test_pool();

        let one = claim_critical_escalation(&pool, -4001, 401, 1000).expect("critical 1");

        let two = claim_critical_escalation(&pool, -4001, 401, 1001).expect("critical 2");

        let three = claim_critical_escalation(&pool, -4001, 401, 1002).expect("critical 3");

        let four = claim_critical_escalation(&pool, -4001, 401, 1003).expect("critical 4");

        assert_eq!(one.occurrence, 1);
        assert_eq!(one.mute_seconds, 600);

        assert_eq!(two.occurrence, 2);
        assert_eq!(two.mute_seconds, 3600);

        assert_eq!(three.occurrence, 3);
        assert_eq!(three.mute_seconds, 86400);

        assert_eq!(four.occurrence, 4);
        assert_eq!(four.mute_seconds, 86400);
    }

    #[test]
    fn critical_escalation_is_scoped_per_user_and_chat() {
        let pool = test_pool();

        let first = claim_critical_escalation(&pool, -5001, 501, 1000).expect("first");

        let second = claim_critical_escalation(&pool, -5001, 501, 1001).expect("second");

        let other_user = claim_critical_escalation(&pool, -5001, 502, 1002).expect("other user");

        let other_chat = claim_critical_escalation(&pool, -5002, 501, 1003).expect("other chat");

        assert_eq!(first.occurrence, 1);
        assert_eq!(second.occurrence, 2);
        assert_eq!(other_user.occurrence, 1);
        assert_eq!(other_chat.occurrence, 1);
    }

    #[test]
    fn critical_escalation_survives_pool_recreation() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();

        let path = std::env::temp_dir().join(format!("resursmap-critical-escalation-{unique}.db"));

        let make_pool = || {
            let manager = SqliteConnectionManager::file(&path).with_init(|conn| {
                init_security_schema(conn)?;
                Ok(())
            });

            Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("file pool")
        };

        {
            let pool = make_pool();

            let first = claim_critical_escalation(&pool, -6001, 601, 2000).expect("first");

            assert_eq!(first.occurrence, 1);
            assert_eq!(first.mute_seconds, 600);
        }

        {
            let pool = make_pool();

            let second = claim_critical_escalation(&pool, -6001, 601, 2001).expect("second");

            assert_eq!(second.occurrence, 2);
            assert_eq!(second.mute_seconds, 3600);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn critical_duration_policy_is_stable() {
        assert_eq!(mute_seconds_for_critical(1), 600);
        assert_eq!(mute_seconds_for_critical(2), 3600);
        assert_eq!(mute_seconds_for_critical(3), 86400);
        assert_eq!(mute_seconds_for_critical(4), 86400);
        assert_eq!(mute_seconds_for_critical(100), 86400);
    }

    #[test]
    fn moderation_audit_event_is_persisted() {
        let pool = test_pool();

        let event = NewModerationAuditEvent {
            chat_id: -7001,
            user_id: 701,
            message_id: 77,
            action: "DELETE_MESSAGE",
            reason: "SuspiciousLink",
            risk_score: 12,
            risk_level: "High",
            critical_occurrence: 0,
            mute_seconds: 0,
            success: true,
            created_at: 3000,
        };

        let id = record_moderation_audit(&pool, &event).expect("record audit");

        assert!(id > 0);

        let conn = get_connection(&pool).expect("connection");

        let row = conn
            .query_row(
                "
                SELECT
                    chat_id,
                    user_id,
                    message_id,
                    action,
                    reason,
                    risk_score,
                    risk_level,
                    critical_occurrence,
                    mute_seconds,
                    success,
                    created_at
                FROM security_moderation_audit
                WHERE id = ?1
                ",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i32>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, i64>(9)?,
                        row.get::<_, i64>(10)?,
                    ))
                },
            )
            .expect("audit row");

        assert_eq!(row.0, -7001);
        assert_eq!(row.1, 701);
        assert_eq!(row.2, 77);
        assert_eq!(row.3, "DELETE_MESSAGE");
        assert_eq!(row.4, "SuspiciousLink");
        assert_eq!(row.5, 12);
        assert_eq!(row.6, "High");
        assert_eq!(row.7, 0);
        assert_eq!(row.8, 0);
        assert_eq!(row.9, 1);
        assert_eq!(row.10, 3000);
    }

    #[test]
    fn moderation_audit_records_mute_metadata() {
        let pool = test_pool();

        let event = NewModerationAuditEvent {
            chat_id: -7002,
            user_id: 702,
            message_id: 88,
            action: "MUTE_1_HOUR",
            reason: "Flood",
            risk_score: 16,
            risk_level: "Critical",
            critical_occurrence: 2,
            mute_seconds: 3600,
            success: true,
            created_at: 4000,
        };

        let id = record_moderation_audit(&pool, &event).expect("record mute audit");

        let conn = get_connection(&pool).expect("connection");

        let row = conn
            .query_row(
                "
                SELECT
                    action,
                    critical_occurrence,
                    mute_seconds,
                    success
                FROM security_moderation_audit
                WHERE id = ?1
                ",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .expect("mute audit row");

        assert_eq!(row.0, "MUTE_1_HOUR");
        assert_eq!(row.1, 2);
        assert_eq!(row.2, 3600);
        assert_eq!(row.3, 1);
    }

    #[test]
    fn moderation_audit_records_failed_action() {
        let pool = test_pool();

        let event = NewModerationAuditEvent {
            chat_id: -7003,
            user_id: 703,
            message_id: 99,
            action: "DELETE_MESSAGE",
            reason: "ExcessiveLinks",
            risk_score: 10,
            risk_level: "High",
            critical_occurrence: 0,
            mute_seconds: 0,
            success: false,
            created_at: 5000,
        };

        let id = record_moderation_audit(&pool, &event).expect("record failed action");

        let conn = get_connection(&pool).expect("connection");

        let success: i64 = conn
            .query_row(
                "
                SELECT success
                FROM security_moderation_audit
                WHERE id = ?1
                ",
                params![id],
                |row| row.get(0),
            )
            .expect("failed audit row");

        assert_eq!(success, 0);
    }

    #[test]
    fn moderation_audit_is_scoped_per_user_and_chat() {
        let pool = test_pool();

        let events = [
            NewModerationAuditEvent {
                chat_id: -7101,
                user_id: 711,
                message_id: 1,
                action: "WARNING",
                reason: "Flood",
                risk_score: 5,
                risk_level: "Medium",
                critical_occurrence: 0,
                mute_seconds: 0,
                success: true,
                created_at: 6000,
            },
            NewModerationAuditEvent {
                chat_id: -7101,
                user_id: 712,
                message_id: 2,
                action: "WARNING",
                reason: "Flood",
                risk_score: 5,
                risk_level: "Medium",
                critical_occurrence: 0,
                mute_seconds: 0,
                success: true,
                created_at: 6001,
            },
            NewModerationAuditEvent {
                chat_id: -7102,
                user_id: 711,
                message_id: 3,
                action: "WARNING",
                reason: "Flood",
                risk_score: 5,
                risk_level: "Medium",
                critical_occurrence: 0,
                mute_seconds: 0,
                success: true,
                created_at: 6002,
            },
        ];

        for event in &events {
            record_moderation_audit(&pool, event).expect("record scoped audit");
        }

        let conn = get_connection(&pool).expect("connection");

        let first: i64 = conn
            .query_row(
                "
                SELECT COUNT(*)
                FROM security_moderation_audit
                WHERE chat_id = -7101
                  AND user_id = 711
                ",
                [],
                |row| row.get(0),
            )
            .expect("first count");

        let other_user: i64 = conn
            .query_row(
                "
                SELECT COUNT(*)
                FROM security_moderation_audit
                WHERE chat_id = -7101
                  AND user_id = 712
                ",
                [],
                |row| row.get(0),
            )
            .expect("other user count");

        let other_chat: i64 = conn
            .query_row(
                "
                SELECT COUNT(*)
                FROM security_moderation_audit
                WHERE chat_id = -7102
                  AND user_id = 711
                ",
                [],
                |row| row.get(0),
            )
            .expect("other chat count");

        assert_eq!(first, 1);
        assert_eq!(other_user, 1);
        assert_eq!(other_chat, 1);
    }

    #[test]
    fn moderation_audit_survives_pool_recreation() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();

        let path = std::env::temp_dir().join(format!("resursmap-moderation-audit-{unique}.db"));

        let make_pool = || {
            let manager = SqliteConnectionManager::file(&path).with_init(|conn| {
                init_security_schema(conn)?;
                Ok(())
            });

            Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("file pool")
        };

        {
            let pool = make_pool();

            let event = NewModerationAuditEvent {
                chat_id: -7201,
                user_id: 721,
                message_id: 100,
                action: "MUTE_24_HOURS",
                reason: "SuspiciousLink",
                risk_score: 20,
                risk_level: "Critical",
                critical_occurrence: 3,
                mute_seconds: 86400,
                success: true,
                created_at: 7000,
            };

            record_moderation_audit(&pool, &event).expect("record persistent audit");
        }

        {
            let pool = make_pool();
            let conn = get_connection(&pool).expect("connection");

            let count: i64 = conn
                .query_row(
                    "
                    SELECT COUNT(*)
                    FROM security_moderation_audit
                    WHERE chat_id = -7201
                      AND user_id = 721
                      AND action = 'MUTE_24_HOURS'
                      AND critical_occurrence = 3
                      AND mute_seconds = 86400
                    ",
                    [],
                    |row| row.get(0),
                )
                .expect("persistent audit count");

            assert_eq!(count, 1);
        }

        let _ = std::fs::remove_file(path);
    }
}
