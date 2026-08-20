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
            FOREIGN KEY(category_id) REFERENCES categories(id),
            UNIQUE(category_id, name)
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

    let continent_count: i32 =
        conn.query_row("SELECT COUNT(*) FROM continents", [], |row| row.get(0))?;

    if continent_count == 0 {
        seed_data(&conn)?;
        info!("Database seeded with initial data");
    } else {
        // На уже существующей БД всё равно досеем пункты
        seed_items(&conn)?;
    }

    Ok(())
}

fn seed_data(conn: &Connection) -> Result<()> {
    let continents = vec!["Европа", "Азия", "Африка", "Америка", "Океания"];
    for continent in &continents {
        conn.execute(
            "INSERT OR IGNORE INTO continents (name) VALUES (?1)",
            params![continent],
        )?;
    }

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

    let categories = vec![
        ("Профессии", Some("Рынок труда и профессии")),
        ("Финансы", Some("Капитал и инвестиции")),
        ("Команда", Some("Команда и нетворкинг")),
        ("Жильё", Some("Недвижимость и аренда")),
        ("Транспорт", Some("Авто и перевозки")),
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

    seed_items(conn)?;
    Ok(())
}

fn seed_items(conn: &Connection) -> Result<()> {
    // category name -> items
    let data: Vec<(&str, Vec<&str>)> = vec![
        (
            "Профессии",
            vec![
                "Водитель",
                "Охранник",
                "Электрик",
                "Сантехник",
                "Программист",
                "Повар",
                "Строитель",
            ],
        ),
        (
            "Финансы",
            vec![
                "Капитал до 1000€",
                "1000–5000€",
                "5000–10000€",
                "10000€+",
            ],
        ),
        (
            "Команда",
            vec!["Готов собрать команду", "Ищу единомышленников"],
        ),
        (
            "Жильё",
            vec!["Сдаю квартиру", "Ищу квартиру", "Есть помещение"],
        ),
        (
            "Транспорт",
            vec!["Есть авто", "Ищу водителя", "Грузоперевозки"],
        ),
    ];

    for (cat_name, items) in data {
        let cat_id: i32 = match conn.query_row(
            "SELECT id FROM categories WHERE name = ?1",
            params![cat_name],
            |row| row.get(0),
        ) {
            Ok(id) => id,
            Err(_) => continue,
        };
        for name in items {
            conn.execute(
                "INSERT OR IGNORE INTO items (category_id, name) VALUES (?1, ?2)",
                params![cat_id, name],
            )?;
        }
    }
    Ok(())
}

pub fn get_continents(conn: &Connection) -> Result<Vec<Continent>> {
    let mut stmt = conn.prepare("SELECT id, name FROM continents ORDER BY name")?;
    let continents = stmt
        .query_map([], |row| {
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
    let countries = stmt
        .query_map(params![continent_id], |row| {
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
    let cities = stmt
        .query_map(params![country_id], |row| {
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
    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(categories)
}

pub fn get_category_name(conn: &Connection, category_id: i32) -> Result<String> {
    let name: String = conn.query_row(
        "SELECT name FROM categories WHERE id = ?1",
        params![category_id],
        |row| row.get(0),
    )?;
    Ok(name)
}

pub fn get_items_by_category(conn: &Connection, category_id: i32) -> Result<Vec<Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, category_id, name FROM items WHERE category_id = ?1 ORDER BY name",
    )?;
    let items = stmt
        .query_map(params![category_id], |row| {
            Ok(Item {
                id: row.get(0)?,
                category_id: row.get(1)?,
                name: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
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
                    current_menu_state: row
                        .get::<_, Option<String>>(6)?
                        .unwrap_or_else(|| "city_menu".to_string()),
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

pub fn toggle_vote(conn: &Connection, user_id: i64, city_id: i32, item_id: i32) -> Result<bool> {
    let existing: Option<i32> = conn
        .query_row(
            "SELECT id FROM votes WHERE user_id = ?1 AND city_id = ?2 AND item_id = ?3",
            params![user_id, city_id, item_id],
            |row| row.get(0),
        )
        .optional()?;

    if existing.is_some() {
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

pub fn get_user_item_votes_for_city(
    conn: &Connection,
    user_id: i64,
    city_id: i32,
) -> Result<Vec<i32>> {
    let mut stmt =
        conn.prepare("SELECT item_id FROM votes WHERE user_id = ?1 AND city_id = ?2")?;
    let ids = stmt
        .query_map(params![user_id, city_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

pub fn get_user_item_votes_for_category(
    conn: &Connection,
    user_id: i64,
    city_id: i32,
    category_id: i32,
) -> Result<Vec<i32>> {
    let mut stmt = conn.prepare(
        "SELECT v.item_id FROM votes v
         JOIN items i ON i.id = v.item_id
         WHERE v.user_id = ?1 AND v.city_id = ?2 AND i.category_id = ?3",
    )?;
    let ids = stmt
        .query_map(params![user_id, city_id, category_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

pub fn get_user_marks(
    conn: &Connection,
    user_id: i64,
    city_id: i32,
) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT c.name, i.name FROM votes v
         JOIN items i ON i.id = v.item_id
         JOIN categories c ON c.id = i.category_id
         WHERE v.user_id = ?1 AND v.city_id = ?2
         ORDER BY c.name, i.name",
    )?;
    let rows = stmt
        .query_map(params![user_id, city_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_vote_count(conn: &Connection) -> Result<i32> {
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM votes", [], |row| row.get(0))?;
    Ok(count)
}

pub fn get_city_stats(conn: &Connection, city_id: i32) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT i.name, COUNT(v.id) FROM items i
         LEFT JOIN votes v ON i.id = v.item_id AND v.city_id = ?1
         GROUP BY i.id
         HAVING COUNT(v.id) > 0
         ORDER BY COUNT(v.id) DESC, i.name",
    )?;
    let rows = stmt
        .query_map(params![city_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_votes_by_city(conn: &Connection, city_id: i32) -> Result<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM votes WHERE city_id = ?1",
        params![city_id],
        |row| row.get(0),
    )?;
    Ok(count)
}
