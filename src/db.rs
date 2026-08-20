use crate::models::*;
use anyhow::Result;
use chrono::Utc;
use log::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::env;
pub fn get_db() -> Result<Connection> {
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "./invesTG.db".to_string());
    Ok(Connection::open(&db_path)?)
}

pub fn init_db() -> Result<()> {
    let conn = get_db()?;

    // Create tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS continents (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );

        CREATE TABLE IF NOT EXISTS countries (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            continent_id INTEGER NOT NULL,
            FOREIGN KEY(continent_id) REFERENCES continents(id),
            UNIQUE(name, continent_id)
        );

        CREATE TABLE IF NOT EXISTS cities (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            country_id INTEGER NOT NULL,
            FOREIGN KEY(country_id) REFERENCES countries(id),
            UNIQUE(name, country_id)
        );

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            city_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
            timestamp TEXT NOT NULL,
            FOREIGN KEY(city_id) REFERENCES cities(id),
            FOREIGN KEY(category_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS user_sessions (
            user_id INTEGER PRIMARY KEY,
            current_state TEXT NOT NULL,
            current_continent_id INTEGER,
            current_country_id INTEGER,
            current_city_id INTEGER,
            current_category_id INTEGER,
            updated_at TEXT NOT NULL
        );
    "#,
    )?;

    // Seed data if tables are empty
    let continent_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM continents",
        [],
        |row| row.get(0),
    )?;

    if continent_count == 0 {
        seed_data(&conn)?;
        info!("Database seeded with initial data");
    }

    Ok(())
}

fn seed_data(conn: &Connection) -> Result<()> {
    // Continents
    let continents = vec![
        "Africa",
        "Asia",
        "Europe",
        "North America",
        "South America",
    ];

    for continent in &continents {
        conn.execute(
            "INSERT OR IGNORE INTO continents (name) VALUES (?1)",
            params![continent],
        )?;
    }

    // Countries
    let countries = vec![
        ("Nigeria", 1),
        ("Kenya", 1),
        ("Egypt", 1),
        ("South Africa", 1),
        ("China", 2),
        ("Japan", 2),
        ("India", 2),
        ("Singapore", 2),
        ("South Korea", 2),
        ("Thailand", 2),
        ("United Kingdom", 3),
        ("Germany", 3),
        ("France", 3),
        ("Netherlands", 3),
        ("Switzerland", 3),
        ("Sweden", 3),
        ("United States", 4),
        ("Canada", 4),
        ("Mexico", 4),
        ("Brazil", 5),
        ("Argentina", 5),
        ("Chile", 5),
    ];

    for (country, continent_id) in &countries {
        conn.execute(
            "INSERT OR IGNORE INTO countries (name, continent_id) VALUES (?1, ?2)",
            params![country, continent_id],
        )?;
    }

    // Cities
    let cities = vec![
        ("Lagos", 1),
        ("Nairobi", 2),
        ("Cairo", 3),
        ("Johannesburg", 4),
        ("Beijing", 5),
        ("Tokyo", 6),
        ("Mumbai", 7),
        ("Singapore", 8),
        ("Seoul", 9),
        ("Bangkok", 10),
        ("London", 11),
        ("Berlin", 12),
        ("Paris", 13),
        ("Amsterdam", 14),
        ("Zurich", 15),
        ("Stockholm", 16),
        ("New York", 17),
        ("Toronto", 18),
        ("Mexico City", 19),
        ("São Paulo", 20),
        ("Buenos Aires", 21),
        ("Santiago", 22),
        ("San Francisco", 17),
        ("Los Angeles", 17),
        ("Chicago", 17),
        ("Hong Kong", 5),
        ("Shanghai", 5),
        ("Dubai", 2),
        ("Toronto", 18),
        ("Vancouver", 18),
    ];

    for (city, country_id) in &cities {
        conn.execute(
            "INSERT OR IGNORE INTO cities (name, country_id) VALUES (?1, ?2)",
            params![city, country_id],
        )?;
    }

    // Categories
    let categories = vec![
        ("Technology", Some("Tech startups and innovation")),
        ("Real Estate", Some("Property and land investment")),
        ("Finance", Some("Banking and financial services")),
        ("Healthcare", Some("Medical and biotech investments")),
        ("Energy", Some("Oil, gas, and renewable energy")),
        ("Retail", Some("Commerce and consumer goods")),
        ("Agriculture", Some("Farming and agribusiness")),
        ("Manufacturing", Some("Industrial production")),
        ("Education", Some("EdTech and learning institutions")),
        ("Transportation", Some("Logistics and mobility solutions")),
        ("Tourism", Some("Hospitality and travel")),
        ("Entertainment", Some("Media and entertainment")),
    ];

    for (category, desc) in &categories {
        conn.execute(
            "INSERT OR IGNORE INTO categories (name, description) VALUES (?1, ?2)",
            params![category, desc],
        )?;
    }

    Ok(())
}

pub fn get_continents(conn: &Connection) -> Result<Vec<Continent>> {
    let mut stmt = conn.prepare("SELECT id, name FROM continents ORDER BY name")?;
    let continents = stmt.query_map([], |row| {
        Ok(Continent {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(continents)
}

pub fn get_countries_by_continent(conn: &Connection, continent_id: i32) -> Result<Vec<Country>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, continent_id FROM countries WHERE continent_id = ?1 ORDER BY name",
    )?;
    let countries = stmt.query_map(params![continent_id], |row| {
        Ok(Country {
            id: row.get(0)?,
            name: row.get(1)?,
            continent_id: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(countries)
}

pub fn get_cities_by_country(conn: &Connection, country_id: i32) -> Result<Vec<City>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, country_id FROM cities WHERE country_id = ?1 ORDER BY name",
    )?;
    let cities = stmt.query_map(params![country_id], |row| {
        Ok(City {
            id: row.get(0)?,
            name: row.get(1)?,
            country_id: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(cities)
}

pub fn get_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, name, description FROM categories ORDER BY name")?;
    let categories = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(categories)
}

pub fn get_user_session(conn: &Connection, user_id: i64) -> Result<UserSession> {
    let result: Option<UserSession> = conn
        .query_row(
            "SELECT user_id, current_state, current_continent_id, current_country_id, current_city_id, current_category_id FROM user_sessions WHERE user_id = ?1",
            params![user_id],
            |row| {
                Ok(UserSession {
                    user_id: row.get(0)?,
                    current_state: row.get(1)?,
                    current_continent_id: row.get(2)?,
                    current_country_id: row.get(3)?,
                    current_city_id: row.get(4)?,
                    current_category_id: row.get(5)?,
                })
            },
        )
        .optional()?;

    Ok(result.unwrap_or(UserSession {
        user_id,
        ..Default::default()
    }))
}

pub fn update_user_session(conn: &Connection, session: &UserSession) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO user_sessions (user_id, current_state, current_continent_id, current_country_id, current_city_id, current_category_id, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            session.user_id,
            session.current_state,
            session.current_continent_id,
            session.current_country_id,
            session.current_city_id,
            session.current_category_id,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn save_vote(
    conn: &Connection,
    user_id: i64,
    city_id: i32,
    category_id: i32,
    rating: i32,
) -> Result<()> {
    conn.execute(
        "INSERT INTO votes (user_id, city_id, category_id, rating, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![user_id, city_id, category_id, rating, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn get_vote_count(conn: &Connection) -> Result<i32> {
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM votes", [], |row| row.get(0))?;
    Ok(count)
}

pub fn get_votes_by_category(conn: &Connection, category_id: i32) -> Result<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM votes WHERE category_id = ?1",
        params![category_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn get_votes_by_city(conn: &Connection, city_id: i32) -> Result<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM votes WHERE city_id = ?1",
        params![city_id],
        |row| row.get(0),
    )?;
    Ok(count)
}
