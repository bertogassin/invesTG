use crate::state::app_state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct ProfessionSuggestQuery {
    q: Option<String>,
    limit: Option<usize>,
}

pub async fn profession_suggestions(
    State(state): State<AppState>,
    Query(query): Query<ProfessionSuggestQuery>,
) -> Json<Value> {
    let q = crate::db::professions::normalize(query.q.as_deref().unwrap_or(""));
    let limit = query.limit.unwrap_or(12).clamp(1, 200);
    if q.chars().count() > 80 || q.chars().any(char::is_control) {
        return Json(json!({"ok": false, "error": "invalid_query"}));
    }
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return Json(json!({"ok": false, "error": "database_unavailable"})),
    };
    let pattern = format!("%{q}%");
    let prefix = format!("{q}%");
    let mut rows = Vec::new();
    if let Ok(mut stmt) = db.prepare(
        "SELECT DISTINCT p.stable_key,p.name_ru,p.name_en,p.name_fr,s.name_ru
         FROM professions p JOIN profession_sectors s ON s.stable_key=p.sector_key
         LEFT JOIN profession_aliases a ON a.profession_id=p.id
         WHERE (?1='' OR p.normalized_ru LIKE ?2 OR p.normalized_en LIKE ?2 OR p.normalized_fr LIKE ?2 OR a.normalized_alias LIKE ?2)
         ORDER BY CASE WHEN p.normalized_ru LIKE ?3 OR p.normalized_en LIKE ?3 OR p.normalized_fr LIKE ?3 THEN 0 ELSE 1 END,s.position,p.name_ru LIMIT ?4",
    ) {
        if let Ok(mapped) = stmt.query_map(rusqlite::params![q, pattern, prefix, limit as i64], |row| {
            Ok(json!({"kind":"profession", "key": row.get::<_,String>(0)?, "name": row.get::<_,String>(1)?, "name_en": row.get::<_,String>(2)?, "name_fr": row.get::<_,String>(3)?, "sector": row.get::<_,String>(4)?}))
        }) {
            rows = mapped.filter_map(Result::ok).collect();
        }
    }
    if rows.len() < limit {
        if let Ok(mut stmt) = db.prepare(
            "SELECT DISTINCT service.stable_key,service.name_ru,service.name_en,service.name_fr,category.name_ru
             FROM services service JOIN service_categories category ON category.stable_key=service.category_key
             LEFT JOIN service_aliases alias ON alias.service_id=service.id
             WHERE (?1='' OR service.normalized_ru LIKE ?2 OR service.normalized_en LIKE ?2 OR service.normalized_fr LIKE ?2 OR category.normalized_ru LIKE ?2 OR category.normalized_en LIKE ?2 OR category.normalized_fr LIKE ?2 OR alias.normalized_alias LIKE ?2)
             ORDER BY CASE WHEN service.normalized_ru LIKE ?3 OR service.normalized_en LIKE ?3 OR service.normalized_fr LIKE ?3 THEN 0 ELSE 1 END,category.position,service.name_ru LIMIT ?4",
        ) {
            let remaining = (limit - rows.len()) as i64;
            if let Ok(mapped) = stmt.query_map(
                rusqlite::params![q, pattern, prefix, remaining],
                |row| {
                    Ok(json!({"kind":"service", "key":row.get::<_,String>(0)?, "name":row.get::<_,String>(1)?, "name_en":row.get::<_,String>(2)?, "name_fr":row.get::<_,String>(3)?, "sector":row.get::<_,String>(4)?}))
                },
            ) {
                rows.extend(mapped.filter_map(Result::ok));
            }
        }
    }
    rows.sort_by(|left, right| {
        left["name"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase()
            .cmp(&right["name"].as_str().unwrap_or_default().to_lowercase())
    });
    Json(json!({"ok": true, "items": rows}))
}

pub async fn profession_sectors(State(state): State<AppState>) -> Json<Value> {
    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => return Json(json!({"ok": false, "error": "database_unavailable"})),
    };
    let mut rows = Vec::new();
    if let Ok(mut stmt) = db.prepare(
        "SELECT stable_key,name_ru,name_en,name_fr FROM profession_sectors ORDER BY position",
    ) {
        if let Ok(mapped) = stmt.query_map([], |row| Ok(json!({"key":row.get::<_,String>(0)?,"name":row.get::<_,String>(1)?,"name_en":row.get::<_,String>(2)?,"name_fr":row.get::<_,String>(3)?}))) {
            rows = mapped.filter_map(Result::ok).collect();
        }
    }
    Json(json!({"ok": true, "items": rows}))
}
