use log::*;
use std::env;

pub fn parse_admin_ids() -> Vec<i64> {
    let admin_ids_str = env::var("ADMIN_IDS").unwrap_or_default();
    admin_ids_str
        .split(',')
        .filter_map(|id| {
            let trimmed = id.trim();
            match trimmed.parse::<i64>() {
                Ok(id) => Some(id),
                Err(e) => {
                    warn!("Failed to parse admin ID '{}': {}", trimmed, e);
                    None
                }
            }
        })
        .collect()
}

pub fn is_admin(user_id: i64) -> bool {
    let admin_ids = parse_admin_ids();
    admin_ids.contains(&user_id)
}
