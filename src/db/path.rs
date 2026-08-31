/// SQLite database file path from `DATABASE_URL` (`sqlite:…`) or default.
pub fn database_path() -> String {
    std::env::var("DATABASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .strip_prefix("sqlite:")
                .unwrap_or(value.as_str())
                .to_string()
        })
        .unwrap_or_else(|| "data/votes.db".to_string())
}
