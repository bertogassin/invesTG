use rusqlite::{Connection, Result};

/// Full-text index for public search. Profile search indexes only
/// profession and intent, never name or username.
pub fn init_search_fts(conn: &Connection) -> Result<()> {
    ensure_resources_fts(conn)?;
    ensure_profiles_fts_parameters_only(conn)?;

    Ok(())
}

fn resources_fts_has_rubric(conn: &Connection) -> bool {
    conn.prepare("SELECT rubric FROM resources_fts LIMIT 1")
        .is_ok()
}

fn ensure_resources_fts(conn: &Connection) -> Result<()> {
    let table_exists = conn.prepare("SELECT 1 FROM resources_fts LIMIT 1").is_ok();
    let needs_rebuild = table_exists && !resources_fts_has_rubric(conn);

    if needs_rebuild {
        conn.execute_batch(
            "
            DROP TRIGGER IF EXISTS resources_fts_ai;
            DROP TRIGGER IF EXISTS resources_fts_ad;
            DROP TRIGGER IF EXISTS resources_fts_au;
            DROP TABLE IF EXISTS resources_fts;
            ",
        )?;
    }

    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE IF NOT EXISTS resources_fts USING fts5(
            title,
            description,
            category,
            address,
            rubric,
            tokenize = 'unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS resources_fts_ai
        AFTER INSERT ON resources
        BEGIN
            INSERT INTO resources_fts(
                rowid, title, description, category, address, rubric
            )
            VALUES (
                new.id, new.title, new.description, new.category, new.address, new.rubric
            );
        END;

        CREATE TRIGGER IF NOT EXISTS resources_fts_ad
        AFTER DELETE ON resources
        BEGIN
            DELETE FROM resources_fts WHERE rowid = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS resources_fts_au
        AFTER UPDATE OF title, description, category, address, rubric ON resources
        BEGIN
            DELETE FROM resources_fts WHERE rowid = old.id;
            INSERT INTO resources_fts(
                rowid, title, description, category, address, rubric
            )
            VALUES (
                new.id, new.title, new.description, new.category, new.address, new.rubric
            );
        END;
        ",
    )?;

    if needs_rebuild {
        backfill_resources_fts_with_lexicon(conn)?;
    } else {
        backfill_resources_fts(conn)?;
    }

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
        backfill_resources_fts_with_lexicon(conn)?;
    }

    Ok(())
}

fn backfill_resources_fts_with_lexicon(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, category, address, COALESCE(rubric, '')
         FROM resources",
    )?;
    let rows: Vec<(i64, String, String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    conn.execute("DELETE FROM resources_fts", [])?;

    for (id, title, description, category, address, rubric) in rows {
        let lexicon = crate::catalog::search_text_for(&rubric);
        conn.execute(
            "INSERT INTO resources_fts(rowid, title, description, category, address, rubric)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, title, description, category, address, lexicon],
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

pub fn ensure_resource_rubric_column(conn: &Connection) -> Result<()> {
    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(resources)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns
    };

    if !columns.iter().any(|column| column == "rubric") {
        let _ = conn.execute(
            "ALTER TABLE resources ADD COLUMN rubric TEXT NOT NULL DEFAULT ''",
            [],
        );
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resources_rubric
         ON resources(rubric)",
        [],
    )?;

    backfill_catalog_values(conn)?;

    Ok(())
}

fn backfill_catalog_values(conn: &Connection) -> Result<()> {
    let mut profile_stmt =
        conn.prepare("SELECT rowid, category FROM profiles WHERE trim(category) <> ''")?;
    let profiles: Vec<(i64, String)> = profile_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(profile_stmt);

    for (rowid, category) in profiles {
        if crate::catalog::by_id(&category).is_some() {
            continue;
        }

        if let Some(rubric) = crate::catalog::resolve(&category) {
            conn.execute(
                "UPDATE profiles SET category = ?1 WHERE rowid = ?2",
                rusqlite::params![rubric.id, rowid],
            )?;
        }
    }

    let mut resource_stmt = conn.prepare(
        "SELECT id, title, category
         FROM resources
         WHERE trim(rubric) = ''",
    )?;
    let resources: Vec<(i64, String, String)> = resource_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(resource_stmt);

    for (id, title, category) in resources {
        let guessed = crate::catalog::resolve(&title).or_else(|| crate::catalog::resolve(&category));

        if let Some(rubric) = guessed {
            conn.execute(
                "UPDATE resources SET rubric = ?1 WHERE id = ?2",
                rusqlite::params![rubric.id, id],
            )?;
        }
    }

    Ok(())
}
