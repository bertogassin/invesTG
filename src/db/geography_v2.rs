use rusqlite::{params, Connection, Result};
use std::time::Duration;

const CONTINENTS: &[(&str, &str)] = &[
    ("AF", "Африка"),
    ("AN", "Антарктида"),
    ("AS", "Азия"),
    ("EU", "Европа"),
    ("NA", "Северная Америка"),
    ("SA", "Южная Америка"),
    ("OC", "Океания"),
];

use super::geography_countries::COUNTRIES;

const CITIES: &[(&str, &str, &str, i64, i64, i64)] = &[
    ("DE", "DE-BERLIN", "Берлин", 0, 0, 0),
    ("DE", "DE-HAMBURG", "Гамбург", 0, 0, 1),
    ("DE", "DE-MUNICH", "Мюнхен", 0, 0, 2),
    ("DE", "DE-COLOGNE", "Кёльн", 0, 0, 3),
    ("IT", "IT-ROME", "Рим", 0, 1, 0),
    ("IT", "IT-MILAN", "Милан", 0, 1, 1),
    ("IT", "IT-NAPLES", "Неаполь", 0, 1, 2),
    ("IT", "IT-TURIN", "Турин", 0, 1, 3),
    ("FR", "FR-PARIS", "Париж", 0, 2, 0),
    ("FR", "FR-MARSEILLE", "Марсель", 0, 2, 1),
    ("FR", "FR-LYON", "Лион", 0, 2, 2),
    ("FR", "FR-TOULOUSE", "Тулуза", 0, 2, 3),
    ("FR", "FR-NICE", "Ницца", 0, 2, 4),
];

pub fn initialize() -> Result<()> {
    let mut connection = Connection::open("data/votes.db")?;

    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    connection.busy_timeout(Duration::from_secs(5))?;

    initialize_connection(&mut connection)
}

pub fn initialize_connection(connection: &mut Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS geography_v2_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now'))
        );

        CREATE TABLE IF NOT EXISTS geo_continents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE
                CHECK(length(code) BETWEEN 2 AND 8),
            name_ru TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
                CHECK(is_active IN (0,1)),
            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now'))
        );

        CREATE TABLE IF NOT EXISTS geo_countries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            continent_id INTEGER NOT NULL,
            iso2 TEXT NOT NULL UNIQUE
                CHECK(length(iso2) = 2),
            iso3 TEXT NOT NULL UNIQUE
                CHECK(length(iso3) = 3),
            name_ru TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
                CHECK(is_active IN (0,1)),
            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            FOREIGN KEY(continent_id)
                REFERENCES geo_continents(id)
        );

        CREATE TABLE IF NOT EXISTS geo_cities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            country_id INTEGER NOT NULL,
            stable_key TEXT NOT NULL UNIQUE,
            name_ru TEXT NOT NULL,
            latitude REAL,
            longitude REAL,
            timezone TEXT NOT NULL DEFAULT '',
            legacy_continent_index INTEGER,
            legacy_country_index INTEGER,
            legacy_city_index INTEGER,
            is_active INTEGER NOT NULL DEFAULT 1
                CHECK(is_active IN (0,1)),
            created_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL
                DEFAULT (strftime('%s','now')),
            FOREIGN KEY(country_id)
                REFERENCES geo_countries(id),
            UNIQUE(
                country_id,
                name_ru
            ),
            UNIQUE(
                legacy_continent_index,
                legacy_country_index,
                legacy_city_index
            )
        );

        CREATE INDEX IF NOT EXISTS idx_geo_countries_continent
        ON geo_countries(continent_id, name_ru);

        CREATE INDEX IF NOT EXISTS idx_geo_cities_country
        ON geo_cities(country_id, name_ru);
        "#,
    )?;

    add_column_if_missing(
        connection,
        "resources",
        "city_id",
        "ALTER TABLE resources ADD COLUMN city_id INTEGER",
    )?;

    add_column_if_missing(
        connection,
        "city_publication_targets",
        "city_id",
        "ALTER TABLE city_publication_targets ADD COLUMN city_id INTEGER",
    )?;

    let transaction = connection.transaction()?;

    for (code, name_ru) in CONTINENTS {
        transaction.execute(
            "INSERT INTO geo_continents (
                code,
                name_ru,
                is_active
             )
             VALUES (?1, ?2, 1)
             ON CONFLICT(code) DO UPDATE SET
                name_ru = excluded.name_ru,
                is_active = 1,
                updated_at = strftime('%s','now')",
            params![code, name_ru],
        )?;
    }

    for (continent_code, iso2, iso3, name_ru) in COUNTRIES {
        transaction.execute(
            "INSERT INTO geo_countries (
                continent_id,
                iso2,
                iso3,
                name_ru,
                is_active
             )
             SELECT
                id,
                ?2,
                ?3,
                ?4,
                1
             FROM geo_continents
             WHERE code = ?1
             ON CONFLICT(iso2) DO UPDATE SET
                continent_id = excluded.continent_id,
                iso3 = excluded.iso3,
                name_ru = excluded.name_ru,
                is_active = 1,
                updated_at = strftime('%s','now')",
            params![continent_code, iso2, iso3, name_ru],
        )?;
    }

    for (country_iso2, stable_key, name_ru, continent_index, country_index, city_index) in CITIES {
        transaction.execute(
            "INSERT INTO geo_cities (
                country_id,
                stable_key,
                name_ru,
                legacy_continent_index,
                legacy_country_index,
                legacy_city_index,
                is_active
             )
             SELECT
                id,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                1
             FROM geo_countries
             WHERE iso2 = ?1
             ON CONFLICT(stable_key) DO UPDATE SET
                country_id = excluded.country_id,
                name_ru = excluded.name_ru,
                legacy_continent_index =
                    excluded.legacy_continent_index,
                legacy_country_index =
                    excluded.legacy_country_index,
                legacy_city_index =
                    excluded.legacy_city_index,
                is_active = 1,
                updated_at = strftime('%s','now')",
            params![
                country_iso2,
                stable_key,
                name_ru,
                continent_index,
                country_index,
                city_index
            ],
        )?;
    }

    transaction.execute(
        "UPDATE resources
         SET city_id = (
            SELECT city.id
            FROM geo_cities AS city
            WHERE city.legacy_continent_index =
                    resources.continent_index
              AND city.legacy_country_index =
                    resources.country_index
              AND city.legacy_city_index =
                    resources.city_index
         )
         WHERE city_id IS NULL",
        [],
    )?;

    transaction.execute(
        "UPDATE city_publication_targets
         SET city_id = (
            SELECT city.id
            FROM geo_cities AS city
            WHERE city.legacy_continent_index =
                    city_publication_targets.continent_index
              AND city.legacy_country_index =
                    city_publication_targets.country_index
              AND city.legacy_city_index =
                    city_publication_targets.city_index
         )
         WHERE city_id IS NULL",
        [],
    )?;

    let unmapped_resources: i64 = transaction.query_row(
        "SELECT COUNT(*)
         FROM resources
         WHERE city_id IS NULL",
        [],
        |row| row.get(0),
    )?;

    let unmapped_targets: i64 = transaction.query_row(
        "SELECT COUNT(*)
         FROM city_publication_targets
         WHERE city_id IS NULL",
        [],
        |row| row.get(0),
    )?;

    if unmapped_resources != 0 || unmapped_targets != 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    transaction.execute(
        "INSERT OR IGNORE INTO geography_v2_migrations (
            version,
            name
         )
         VALUES (
            1,
            'stable_geography_foundation'
         )",
        [],
    )?;

    transaction.commit()?;

    connection.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_resources_city_id
        ON resources(city_id, moderation_status, is_active);

        CREATE INDEX IF NOT EXISTS
            idx_city_publication_targets_city_id
        ON city_publication_targets(city_id, is_active);

        CREATE UNIQUE INDEX IF NOT EXISTS
            idx_one_active_primary_group_per_city
        ON city_publication_targets(city_id)
        WHERE city_id IS NOT NULL
          AND is_active = 1
          AND target_kind = 'group';
        "#,
    )?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("memory database");

        connection
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;

                CREATE TABLE resources (
                    id INTEGER PRIMARY KEY,
                    continent_index INTEGER NOT NULL,
                    country_index INTEGER NOT NULL,
                    city_index INTEGER NOT NULL,
                    moderation_status TEXT NOT NULL
                        DEFAULT 'pending',
                    is_active INTEGER NOT NULL DEFAULT 1
                );

                CREATE TABLE city_publication_targets (
                    id INTEGER PRIMARY KEY,
                    continent_index INTEGER NOT NULL,
                    country_index INTEGER NOT NULL,
                    city_index INTEGER NOT NULL,
                    city_name TEXT NOT NULL,
                    target_name TEXT NOT NULL,
                    telegram_chat_id INTEGER NOT NULL DEFAULT 0,
                    target_kind TEXT NOT NULL DEFAULT 'group',
                    is_active INTEGER NOT NULL DEFAULT 0
                );

                INSERT INTO resources (
                    id,
                    continent_index,
                    country_index,
                    city_index
                )
                VALUES (1, 0, 2, 4);

                INSERT INTO city_publication_targets (
                    id,
                    continent_index,
                    country_index,
                    city_index,
                    city_name,
                    target_name,
                    telegram_chat_id,
                    target_kind,
                    is_active
                )
                VALUES (
                    1,
                    0,
                    2,
                    4,
                    'Ницца',
                    'Основная группа',
                    -1001,
                    'group',
                    1
                );
                "#,
            )
            .expect("base schema");

        connection
    }

    #[test]
    fn world_country_catalog_is_complete_and_unique() {
        assert_eq!(COUNTRIES.len(), 249);

        let mut iso2_codes = COUNTRIES
            .iter()
            .map(|country| country.1)
            .collect::<Vec<_>>();

        iso2_codes.sort_unstable();
        iso2_codes.dedup();

        assert_eq!(iso2_codes.len(), 249);

        assert!(COUNTRIES.iter().any(|country| {
            country.1 == "FR" && country.2 == "FRA" && country.3 == "Франция"
        }));

        assert!(COUNTRIES
            .iter()
            .any(|country| { country.0 == "AN" && country.1 == "AQ" && country.2 == "ATA" }));
    }

    #[test]
    fn migration_backfills_nice_resource_and_group() {
        let mut connection = test_connection();

        initialize_connection(&mut connection).expect("geography migration");

        let resource_city: String = connection
            .query_row(
                "SELECT city.stable_key
                 FROM resources AS resource
                 JOIN geo_cities AS city
                   ON city.id = resource.city_id
                 WHERE resource.id = 1",
                [],
                |row| row.get(0),
            )
            .expect("resource city");

        let target_city: String = connection
            .query_row(
                "SELECT city.stable_key
                 FROM city_publication_targets AS target
                 JOIN geo_cities AS city
                   ON city.id = target.city_id
                 WHERE target.id = 1",
                [],
                |row| row.get(0),
            )
            .expect("target city");

        assert_eq!(resource_city, "FR-NICE");
        assert_eq!(target_city, "FR-NICE");
    }

    #[test]
    fn migration_is_idempotent() {
        let mut connection = test_connection();

        initialize_connection(&mut connection).expect("first migration");

        initialize_connection(&mut connection).expect("second migration");

        let versions: i64 = connection
            .query_row(
                "SELECT COUNT(*)
                 FROM geography_v2_migrations
                 WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .expect("migration count");

        assert_eq!(versions, 1);
    }

    #[test]
    fn duplicate_active_primary_group_is_rejected() {
        let mut connection = test_connection();

        initialize_connection(&mut connection).expect("geography migration");

        let nice_id: i64 = connection
            .query_row(
                "SELECT id
                 FROM geo_cities
                 WHERE stable_key = 'FR-NICE'",
                [],
                |row| row.get(0),
            )
            .expect("Nice");

        let duplicate = connection.execute(
            "INSERT INTO city_publication_targets (
                id,
                continent_index,
                country_index,
                city_index,
                city_id,
                city_name,
                target_name,
                telegram_chat_id,
                target_kind,
                is_active
             )
             VALUES (
                2,
                0,
                2,
                4,
                ?1,
                'Ницца',
                'Вторая основная группа',
                -1002,
                'group',
                1
             )",
            [nice_id],
        );

        assert!(duplicate.is_err());
    }
}
