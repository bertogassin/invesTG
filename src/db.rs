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

        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY,
            category_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            FOREIGN KEY(category_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            city_id INTEGER NOT NULL,
            item_id INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY(city_id) REFERENCES cities(id),
            FOREIGN KEY(item_id) REFERENCES items(id),
            UNIQUE(user_id, city_id, item_id)
        );

        CREATE TABLE IF NOT EXISTS user_sessions (
            user_id INTEGER PRIMARY KEY,
            current_state TEXT NOT NULL,
            current_continent_id INTEGER,
            current_country_id INTEGER,
            current_city_id INTEGER,
            current_category_id INTEGER,
            current_menu_state TEXT NOT NULL DEFAULT 'city_menu',
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
    // Continents (Russian)
    let continents = vec![
        "Европа",
        "Азия",
        "Африка",
        "Америка",
        "Океания",
    ];

    for continent in &continents {
        conn.execute(
            "INSERT OR IGNORE INTO continents (name) VALUES (?1)",
            params![continent],
        )?;
    }

    // Countries (Russian names, mapped to continent IDs: Европа=1, Азия=2, Африка=3, Америка=4, Океания=5)
    let countries = vec![
        ("Россия", 1),
        ("Германия", 1),
        ("Франция", 1),
        ("Испания", 1),
        ("Италия", 1),
        ("Польша", 1),
        ("Великобритания", 1),
        ("США", 4),
        ("Канада", 4),
        ("Мексика", 4),
        ("Бразилия", 4),
        ("Китай", 2),
        ("Япония", 2),
        ("Индия", 2),
        ("Австралия", 5),
    ];

    for (country, continent_id) in &countries {
        conn.execute(
            "INSERT OR IGNORE INTO countries (name, continent_id) VALUES (?1, ?2)",
            params![country, continent_id],
        )?;
    }

    // Cities (Russian names, mapped to country IDs)
    // Россия=1, Германия=2, Франция=3, Испания=4, Италия=5, Польша=6, Великобритания=7
    // США=8, Канада=9, Мексика=10, Бразилия=11, Китай=12, Япония=13, Индия=14, Австралия=15
    let cities = vec![
        ("Москва", 1),
        ("Санкт-Петербург", 1),
        ("Берлин", 2),
        ("Париж", 3),
        ("Мадрид", 4),
        ("Рим", 5),
        ("Варшава", 6),
        ("Лондон", 7),
        ("Нью-Йорк", 8),
        ("Торонто", 9),
        ("Мехико", 10),
        ("Сан-Паулу", 11),
        ("Пекин", 12),
        ("Токио", 13),
        ("Дели", 14),
        ("Сидней", 15),
    ];

    for (city, country_id) in &cities {
        conn.execute(
            "INSERT OR IGNORE INTO cities (name, country_id) VALUES (?1, ?2)",
            params![city, country_id],
        )?;
    }

    // Categories (Russian)
    let categories = vec![
        ("Профессии", Some("Рынок труда и профессии")),
        ("Финансы", Some("Банки и финансовые услуги")),
        ("Команда", Some("Командная работа и нетворкинг")),
        ("Жильё", Some("Недвижимость и аренда")),
        ("Транспорт", Some("Транспорт и перевозки")),
        ("Инфраструктура", Some("Инфраструктура города")),
        ("Образование", Some("Образование и наука")),
        ("Здравоохранение", Some("Медицина и здоровье")),
        ("Развлечения", Some("Культура и досуг")),
    ];

    for (category, desc) in &categories {
        conn.execute(
            "INSERT OR IGNORE INTO categories (name, description) VALUES (?1, ?2)",
            params![category, desc],
        )?;
    }

    // Items (Russian) per category
    // category IDs are assigned by insertion order above
    let items: Vec<(&str, &str)> = vec![
        // Профессии
        ("Профессии", "Водитель"),
        ("Профессии", "Охранник"),
        ("Профессии", "Электрик"),
        ("Профессии", "Сантехник"),
        ("Профессии", "Программист"),
        ("Профессии", "Повар"),
        ("Профессии", "Строитель"),
        // Финансы
        ("Финансы", "Капитал до 1000€"),
        ("Финансы", "1000–5000€"),
        ("Финансы", "5000–10000€"),
        ("Финансы", "10000€+"),
        // Команда
        ("Команда", "Готов собрать команду"),
        ("Команда", "Ищу единомышленников"),
        // Жильё
        ("Жильё", "Сдаю квартиру"),
        ("Жильё", "Ищу квартиру"),
        ("Жильё", "Есть помещение"),
        // Транспорт
        ("Транспорт", "Есть авто"),
        ("Транспорт", "Ищу водителя"),
        ("Транспорт", "Грузоперевозки"),
        // Инфраструктура
        ("Инфраструктура", "Интернет кафе"),
        ("Инфраструктура", "Коворкинг"),
        ("Инфраструктура", "Общественный транспорт"),
        ("Инфраструктура", "Парковка"),
        // Образование
        ("Образование", "Школа"),
        ("Образование", "Университет"),
        ("Образование", "Языковые курсы"),
        ("Образование", "ПТУ"),
        // Здравоохранение
        ("Здравоохранение", "Больница"),
        ("Здравоохранение", "Аптека"),
        ("Здравоохранение", "Стоматолог"),
        ("Здравоохранение", "Поликлиника"),
        // Развлечения
        ("Развлечения", "Бар"),
        ("Развлечения", "Ресторан"),
        ("Развлечения", "Кино"),
        ("Развлечения", "Клуб"),
    ];

    for (cat_name, item_name) in &items {
        conn.execute(
            "INSERT OR IGNORE INTO items (category_id, name) SELECT id, ?1 FROM categories WHERE name = ?2",
            params![item_name, cat_name],
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

pub fn get_city_name(conn: &Connection, city_id: i32) -> Result<String> {
    let name: String = conn.query_row(
        "SELECT name FROM cities WHERE id = ?1",
        params![city_id],
        |row| row.get(0),
    )?;
    Ok(name)
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
            "SELECT user_id, current_state, current_continent_id, current_country_id, current_city_id, current_category_id, current_menu_state FROM user_sessions WHERE user_id = ?1",
            params![user_id],
            |row| {
                Ok(UserSession {
                    user_id: row.get(0)?,
                    current_state: row.get(1)?,
                    current_continent_id: row.get(2)?,
                    current_country_id: row.get(3)?,
                    current_city_id: row.get(4)?,
                    current_category_id: row.get(5)?,
                    current_menu_state: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "city_menu".to_string()),
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
        "INSERT OR REPLACE INTO user_sessions (user_id, current_state, current_continent_id, current_country_id, current_city_id, current_category_id, current_menu_state, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            session.user_id,
            session.current_state,
            session.current_continent_id,
            session.current_country_id,
            session.current_city_id,
            session.current_category_id,
            session.current_menu_state,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn get_items_by_category(conn: &Connection, category_id: i32) -> Result<Vec<crate::models::Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, category_id, name FROM items WHERE category_id = ?1 ORDER BY id",
    )?;
    let items = stmt.query_map(params![category_id], |row| {
        Ok(crate::models::Item {
            id: row.get(0)?,
            category_id: row.get(1)?,
            name: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

pub fn check_vote_exists(conn: &Connection, user_id: i64, city_id: i32, item_id: i32) -> Result<bool> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM votes WHERE user_id = ?1 AND city_id = ?2 AND item_id = ?3",
        params![user_id, city_id, item_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn toggle_vote(
    conn: &Connection,
    user_id: i64,
    city_id: i32,
    item_id: i32,
) -> Result<bool> {
    let exists = check_vote_exists(conn, user_id, city_id, item_id)?;
    if exists {
        conn.execute(
            "DELETE FROM votes WHERE user_id = ?1 AND city_id = ?2 AND item_id = ?3",
            params![user_id, city_id, item_id],
        )?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO votes (user_id, city_id, item_id, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![user_id, city_id, item_id, Utc::now().to_rfc3339()],
        )?;
        Ok(true)
    }
}

pub fn get_city_votes(
    conn: &Connection,
    user_id: i64,
    city_id: i32,
) -> Result<Vec<i32>> {
    let mut stmt = conn.prepare(
        "SELECT item_id FROM votes WHERE user_id = ?1 AND city_id = ?2",
    )?;
    let ids = stmt
        .query_map(params![user_id, city_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

pub fn get_vote_count(conn: &Connection) -> Result<i32> {
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM votes", [], |row| row.get(0))?;
    Ok(count)
}

pub fn get_city_stats(conn: &Connection, city_id: i32) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT i.name, COUNT(v.id) FROM items i LEFT JOIN votes v ON i.id = v.item_id AND v.city_id = ?1 GROUP BY i.id ORDER BY i.category_id, i.id",
    )?;
    let rows = stmt
        .query_map(params![city_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
