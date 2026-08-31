use rusqlite::{Connection, Result};
use std::time::Duration;

pub fn initialize() -> Result<()> {
    let mut connection = Connection::open(crate::db::path::database_path())?;

    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(30))?;

    synchronize(&mut connection)
}

pub fn synchronize(connection: &mut Connection) -> Result<()> {
    let required_tables: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM sqlite_master
         WHERE type = 'table'
           AND name IN (
               'geographic_scopes',
               'geo_continents',
               'geo_countries',
               'geo_cities',
               'geography_v2_migrations'
           )",
        [],
        |row| row.get(0),
    )?;

    if required_tables != 5 {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let already_applied: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM geography_v2_migrations
         WHERE version = 3",
        [],
        |row| row.get(0),
    )?;

    if already_applied != 0 {
        return Ok(());
    }

    let transaction = connection.transaction()?;

    let world_scope_id: i64 = transaction.query_row(
        "SELECT id
         FROM geographic_scopes
         WHERE scope_type = 'world'
           AND external_key = 'world'
           AND is_active = 1",
        [],
        |row| row.get(0),
    )?;

    // Сохраняем ID прежних областей, меняя только их
    // индексные ключи на стабильные коды Geography V2.
    transaction.execute(
        "UPDATE geographic_scopes AS scope
         SET external_key = (
                 SELECT 'city:' || city.stable_key
                 FROM geo_cities AS city
                 WHERE scope.external_key =
                       printf(
                           'city:%d:%d:%d',
                           city.legacy_continent_index,
                           city.legacy_country_index,
                           city.legacy_city_index
                       )
             ),
             display_name = (
                 SELECT city.name_ru
                 FROM geo_cities AS city
                 WHERE scope.external_key =
                       printf(
                           'city:%d:%d:%d',
                           city.legacy_continent_index,
                           city.legacy_country_index,
                           city.legacy_city_index
                       )
             ),
             updated_at = strftime('%s','now')
         WHERE scope.scope_type = 'city'
           AND scope.external_key LIKE 'city:%:%:%'
           AND EXISTS (
               SELECT 1
               FROM geo_cities AS city
               WHERE scope.external_key =
                     printf(
                         'city:%d:%d:%d',
                         city.legacy_continent_index,
                         city.legacy_country_index,
                         city.legacy_city_index
                     )
           )",
        [],
    )?;

    transaction.execute(
        "UPDATE geographic_scopes AS scope
         SET external_key = (
                 SELECT 'group:' || city.stable_key || ':main'
                 FROM geo_cities AS city
                 WHERE scope.external_key =
                       printf(
                           'group:%d:%d:%d:main',
                           city.legacy_continent_index,
                           city.legacy_country_index,
                           city.legacy_city_index
                       )
             ),
             display_name = (
                 SELECT 'Городская группа · ' || city.name_ru
                 FROM geo_cities AS city
                 WHERE scope.external_key =
                       printf(
                           'group:%d:%d:%d:main',
                           city.legacy_continent_index,
                           city.legacy_country_index,
                           city.legacy_city_index
                       )
             ),
             updated_at = strftime('%s','now')
         WHERE scope.scope_type = 'group'
           AND scope.external_key LIKE 'group:%:%:%:main'
           AND EXISTS (
               SELECT 1
               FROM geo_cities AS city
               WHERE scope.external_key =
                     printf(
                         'group:%d:%d:%d:main',
                         city.legacy_continent_index,
                         city.legacy_country_index,
                         city.legacy_city_index
                     )
           )",
        [],
    )?;

    transaction.execute(
        "UPDATE geographic_scopes
         SET external_key = 'country:DE',
             display_name = 'Германия',
             updated_at = strftime('%s','now')
         WHERE scope_type = 'country'
           AND external_key = 'country:0:0'",
        [],
    )?;

    transaction.execute(
        "UPDATE geographic_scopes
         SET external_key = 'country:IT',
             display_name = 'Италия',
             updated_at = strftime('%s','now')
         WHERE scope_type = 'country'
           AND external_key = 'country:0:1'",
        [],
    )?;

    transaction.execute(
        "UPDATE geographic_scopes
         SET external_key = 'country:FR',
             display_name = 'Франция',
             updated_at = strftime('%s','now')
         WHERE scope_type = 'country'
           AND external_key = 'country:0:2'",
        [],
    )?;

    transaction.execute(
        "UPDATE geographic_scopes
         SET external_key = 'continent:EU',
             display_name = 'Европа',
             updated_at = strftime('%s','now')
         WHERE scope_type = 'continent'
           AND external_key = 'continent:0'",
        [],
    )?;

    transaction.execute(
        "INSERT INTO geographic_scopes (
             scope_type,
             parent_scope_id,
             external_key,
             display_name,
             is_active
         )
         SELECT
             'continent',
             ?1,
             'continent:' || continent.code,
             continent.name_ru,
             continent.is_active
         FROM geo_continents AS continent
         WHERE 1 = 1
         ON CONFLICT(scope_type, external_key) DO UPDATE SET
             parent_scope_id = excluded.parent_scope_id,
             display_name = excluded.display_name,
             is_active = excluded.is_active,
             updated_at = strftime('%s','now')",
        [world_scope_id],
    )?;

    transaction.execute(
        "INSERT INTO geographic_scopes (
             scope_type,
             parent_scope_id,
             external_key,
             display_name,
             is_active
         )
         SELECT
             'country',
             continent_scope.id,
             'country:' || country.iso2,
             country.name_ru,
             country.is_active
         FROM geo_countries AS country
         JOIN geo_continents AS continent
           ON continent.id = country.continent_id
         JOIN geographic_scopes AS continent_scope
           ON continent_scope.scope_type = 'continent'
          AND continent_scope.external_key =
              'continent:' || continent.code
         WHERE 1 = 1
         ON CONFLICT(scope_type, external_key) DO UPDATE SET
             parent_scope_id = excluded.parent_scope_id,
             display_name = excluded.display_name,
             is_active = excluded.is_active,
             updated_at = strftime('%s','now')",
        [],
    )?;

    transaction.execute(
        "INSERT INTO geographic_scopes (
             scope_type,
             parent_scope_id,
             external_key,
             display_name,
             is_active
         )
         SELECT
             'city',
             country_scope.id,
             'city:' || city.stable_key,
             city.name_ru,
             city.is_active
         FROM geo_cities AS city
         JOIN geo_countries AS country
           ON country.id = city.country_id
         JOIN geographic_scopes AS country_scope
           ON country_scope.scope_type = 'country'
          AND country_scope.external_key =
              'country:' || country.iso2
         WHERE 1 = 1
         ON CONFLICT(scope_type, external_key) DO UPDATE SET
             parent_scope_id = excluded.parent_scope_id,
             display_name = excluded.display_name,
             is_active = excluded.is_active,
             updated_at = strftime('%s','now')",
        [],
    )?;

    // Области групп создаём только там, где группа уже
    // существует. Пустые города не получают фиктивную группу.
    transaction.execute(
        "INSERT INTO geographic_scopes (
             scope_type,
             parent_scope_id,
             external_key,
             display_name,
             is_active
         )
         SELECT
             'group',
             city_scope.id,
             'group:' || city.stable_key || ':main',
             CASE
                 WHEN target.target_name <> ''
                 THEN target.target_name || ' · ' || city.name_ru
                 ELSE 'Городская группа · ' || city.name_ru
             END,
             target.is_active
         FROM city_publication_targets AS target
         JOIN geo_cities AS city
           ON city.id = target.city_id
         JOIN geographic_scopes AS city_scope
           ON city_scope.scope_type = 'city'
          AND city_scope.external_key =
              'city:' || city.stable_key
         WHERE target.target_kind = 'group'
         ON CONFLICT(scope_type, external_key) DO UPDATE SET
             parent_scope_id = excluded.parent_scope_id,
             display_name = excluded.display_name,
             is_active = excluded.is_active,
             updated_at = strftime('%s','now')",
        [],
    )?;

    let continent_count: i64 = transaction.query_row(
        "SELECT COUNT(*)
         FROM geographic_scopes
         WHERE scope_type = 'continent'
           AND is_active = 1",
        [],
        |row| row.get(0),
    )?;

    let country_count: i64 = transaction.query_row(
        "SELECT COUNT(*)
         FROM geographic_scopes
         WHERE scope_type = 'country'
           AND is_active = 1",
        [],
        |row| row.get(0),
    )?;

    let city_count: i64 = transaction.query_row(
        "SELECT COUNT(*)
         FROM geographic_scopes
         WHERE scope_type = 'city'
           AND is_active = 1",
        [],
        |row| row.get(0),
    )?;

    if continent_count != 7 || country_count != 249 || city_count != 34_104 {
        return Err(rusqlite::Error::InvalidQuery);
    }

    transaction.execute(
        "INSERT INTO geography_v2_migrations (
             version,
             name
         )
         VALUES (
             3,
             'admin_geography_scope_sync'
         )",
        [],
    )?;

    transaction.commit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synchronization_requires_complete_schema() {
        let mut connection = Connection::open_in_memory().expect("memory database");

        let result = synchronize(&mut connection);

        assert!(result.is_err());
    }
}
