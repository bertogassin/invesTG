use super::admin_v2::INITIAL_OWNER_USER_ID;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::Duration;

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "password_hash_failed".to_string())
}

fn normalize_bootstrap_email(value: &str) -> Option<String> {
    let email = value.trim().to_ascii_lowercase();
    if email.len() < 5 || !email.contains('@') || email.len() > 254 {
        return None;
    }
    Some(email)
}

pub fn bootstrap_owner_from_env() -> rusqlite::Result<()> {
    let email = match std::env::var("OWNER_BOOTSTRAP_EMAIL")
        .ok()
        .and_then(|value| normalize_bootstrap_email(&value))
    {
        Some(email) => email,
        None => return Ok(()),
    };

    let password = match std::env::var("OWNER_BOOTSTRAP_PASSWORD") {
        Ok(value) if value.trim().len() >= 8 => value,
        _ => {
            eprintln!(
                "OWNER_BOOTSTRAP_EMAIL задан, но OWNER_BOOTSTRAP_PASSWORD отсутствует или короче 8 символов"
            );
            return Ok(());
        }
    };

    let mut connection = Connection::open(Path::new("data/votes.db"))?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;

    let owner_exists: i64 = connection.query_row(
        "SELECT COUNT(*) FROM users WHERE id = ?1 AND is_active = 1",
        params![INITIAL_OWNER_USER_ID],
        |row| row.get(0),
    )?;

    if owner_exists == 1 {
        return Ok(());
    }

    let email_taken: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM auth_identities
         WHERE provider = 'email'
           AND email = ?1",
        params![email],
        |row| row.get(0),
    )?;

    if email_taken != 0 {
        eprintln!(
            "OWNER_BOOTSTRAP_EMAIL уже используется другим аккаунтом; owner bootstrap пропущен"
        );
        return Ok(());
    }

    let password_hash = match hash_password(password.trim()) {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("owner bootstrap: {error}");
            return Ok(());
        }
    };

    let now = chrono::Utc::now().timestamp();
    let transaction = connection.transaction()?;

    transaction.execute(
        "INSERT INTO users (id, created_at, updated_at, is_active, telegram_id)
         VALUES (?1, ?2, ?2, 1, NULL)",
        params![INITIAL_OWNER_USER_ID, now],
    )?;

    transaction.execute(
        "INSERT INTO auth_identities (
            user_id, provider, provider_subject, email, password_hash,
            verified_at, created_at, updated_at
         )
         VALUES (?1, 'email', ?2, ?2, ?3, ?4, ?4, ?4)",
        params![INITIAL_OWNER_USER_ID, email, password_hash, now],
    )?;

    let client_id = format!("user:{INITIAL_OWNER_USER_ID}");
    transaction.execute(
        "INSERT OR IGNORE INTO profiles (client_id, user_id, public_id, updated_at)
         VALUES (?1, ?2, lower(hex(randomblob(16))), ?3)",
        params![client_id, INITIAL_OWNER_USER_ID, now],
    )?;

    transaction.commit()?;

    println!(
        "Owner bootstrap: создан email-аккаунт владельца (id={INITIAL_OWNER_USER_ID}). Перезапустите сервер для применения Admin V2."
    );

    Ok(())
}
