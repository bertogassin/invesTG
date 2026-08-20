use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Continent {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub continent_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub id: i32,
    pub name: String,
    pub country_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: i32,
    pub user_id: i64,
    pub city_id: i32,
    pub category_id: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: i64,
    pub current_state: String,
    pub current_continent_id: Option<i32>,
    pub current_country_id: Option<i32>,
    pub current_city_id: Option<i32>,
    pub current_category_id: Option<i32>,
    pub current_menu_state: String,
}

impl Default for UserSession {
    fn default() -> Self {
        Self {
            user_id: 0,
            current_state: "start".to_string(),
            current_continent_id: None,
            current_country_id: None,
            current_city_id: None,
            current_category_id: None,
            current_menu_state: "city_menu".to_string(),
        }
    }
}
