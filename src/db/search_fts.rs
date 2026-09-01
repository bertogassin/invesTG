use rusqlite::{Connection, Result};

/// Full-text index for public search. Profile search indexes only
/// profession and intent, never name or username.
pub fn init_search_fts(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE IF NOT EXISTS resources_fts USING fts5(
            title,
            description,
            category,
            address,
            tokenize = 'unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS resources_fts_ai
        AFTER INSERT ON resources
        BEGIN
            INSERT INTO resources_fts(
                rowid, title, description, category, address
            )
            VALUES (
                new.id, new.title, new.description, new.category, new.address
            );
        END;

        CREATE TRIGGER IF NOT EXISTS resources_fts_ad
        AFTER DELETE ON resources
        BEGIN
            DELETE FROM resources_fts WHERE rowid = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS resources_fts_au
        AFTER UPDATE OF title, description, category, address ON resources
        BEGIN
            DELETE FROM resources_fts WHERE rowid = old.id;
            INSERT INTO resources_fts(
                rowid, title, description, category, address
            )
            VALUES (
                new.id, new.title, new.description, new.category, new.address
            );
        END;

        ",
    )?;

    backfill_resources_fts(conn)?;
    ensure_profiles_fts_parameters_only(conn)?;

    Ok(())
}

fn profiles_fts_column_names(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("PRAGMA table_info(profiles_fts)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(columns)
}

fn profiles_fts_indexes_names(columns: &[String]) -> bool {
    columns
        .iter()
        .any(|column| matches!(column.as_str(), "username" | "first_name" | "last_name"))
}

fn ensure_profiles_fts_parameters_only(conn: &Connection) -> Result<()> {
    let columns = profiles_fts_column_names(conn).unwrap_or_default();
    let needs_rebuild = columns.is_empty()
        || profiles_fts_indexes_names(&columns)
        || !columns.iter().any(|column| column == "category")
        || !columns.iter().any(|column| column == "intent_text");

    if needs_rebuild {
        conn.execute_batch(
            "
            DROP TRIGGER IF EXISTS profiles_fts_ai;
            DROP TRIGGER IF EXISTS profiles_fts_ad;
            DROP TRIGGER IF EXISTS profiles_fts_au;
            DROP TABLE IF EXISTS profiles_fts;

            CREATE VIRTUAL TABLE profiles_fts USING fts5(
                category,
                intent_text,
                tokenize = 'unicode61'
            );
            ",
        )?;
    }

    conn.execute_batch(
        "
        CREATE TRIGGER IF NOT EXISTS profiles_fts_ai
        AFTER INSERT ON profiles
        BEGIN
            INSERT INTO profiles_fts(rowid, category, intent_text)
            VALUES (new.rowid, new.category, new.intent_text);
        END;

        CREATE TRIGGER IF NOT EXISTS profiles_fts_ad
        AFTER DELETE ON profiles
        BEGIN
            DELETE FROM profiles_fts WHERE rowid = old.rowid;
        END;

        CREATE TRIGGER IF NOT EXISTS profiles_fts_au
        AFTER UPDATE OF category, intent_text ON profiles
        BEGIN
            DELETE FROM profiles_fts WHERE rowid = old.rowid;
            INSERT INTO profiles_fts(rowid, category, intent_text)
            VALUES (new.rowid, new.category, new.intent_text);
        END;
        ",
    )?;

    if needs_rebuild {
        conn.execute(
            "INSERT INTO profiles_fts(rowid, category, intent_text)
             SELECT rowid, category, intent_text
             FROM profiles",
            [],
        )?;
    } else {
        backfill_profiles_fts(conn)?;
    }

    Ok(())
}

fn backfill_resources_fts(conn: &Connection) -> Result<()> {
    let indexed: i64 =
        conn.query_row("SELECT COUNT(*) FROM resources_fts", [], |row| row.get(0))?;
    let source: i64 = conn.query_row("SELECT COUNT(*) FROM resources", [], |row| row.get(0))?;

    if indexed == 0 && source > 0 {
        conn.execute(
            "INSERT INTO resources_fts(rowid, title, description, category, address)
             SELECT id, title, description, category, address
             FROM resources",
            [],
        )?;
    }

    Ok(())
}

fn backfill_profiles_fts(conn: &Connection) -> Result<()> {
    let indexed: i64 = conn.query_row("SELECT COUNT(*) FROM profiles_fts", [], |row| row.get(0))?;
    let source: i64 = conn.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))?;

    if indexed == 0 && source > 0 {
        conn.execute(
            "INSERT INTO profiles_fts(rowid, category, intent_text)
             SELECT rowid, category, intent_text
             FROM profiles",
            [],
        )?;
    }

    Ok(())
}

pub fn ensure_profile_home_city_columns(conn: &Connection) -> Result<()> {
    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(profiles)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns
    };

    for (name, sql) in [
        (
            "home_continent_index",
            "ALTER TABLE profiles ADD COLUMN home_continent_index INTEGER",
        ),
        (
            "home_country_index",
            "ALTER TABLE profiles ADD COLUMN home_country_index INTEGER",
        ),
        (
            "home_city_index",
            "ALTER TABLE profiles ADD COLUMN home_city_index INTEGER",
        ),
        (
            "home_city_id",
            "ALTER TABLE profiles ADD COLUMN home_city_id INTEGER",
        ),
    ] {
        if !columns.iter().any(|column| column == name) {
            let _ = conn.execute(sql, []);
        }
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_profiles_home_city
         ON profiles(home_continent_index, home_country_index, home_city_index)",
        [],
    )?;

    conn.execute(
        "UPDATE profiles
         SET home_continent_index = (
                SELECT r.continent_index
                FROM resources AS r
                WHERE r.client_id = profiles.client_id
                ORDER BY r.updated_at DESC, r.id DESC
                LIMIT 1
             ),
             home_country_index = (
                SELECT r.country_index
                FROM resources AS r
                WHERE r.client_id = profiles.client_id
                ORDER BY r.updated_at DESC, r.id DESC
                LIMIT 1
             ),
             home_city_index = (
                SELECT r.city_index
                FROM resources AS r
                WHERE r.client_id = profiles.client_id
                ORDER BY r.updated_at DESC, r.id DESC
                LIMIT 1
             ),
             home_city_id = (
                SELECT r.city_id
                FROM resources AS r
                WHERE r.client_id = profiles.client_id
                  AND r.city_id IS NOT NULL
                ORDER BY r.updated_at DESC, r.id DESC
                LIMIT 1
             )
         WHERE home_continent_index IS NULL
           AND EXISTS (
                SELECT 1
                FROM resources AS r
                WHERE r.client_id = profiles.client_id
           )",
        [],
    )?;

    Ok(())
}
