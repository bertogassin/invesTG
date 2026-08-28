use crate::state::app_state::AppState;
use axum::http::{header, HeaderMap};
use hmac::{Hmac, Mac};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ADMIN_SESSION_TTL_SECONDS: i64 = 1_800;
const ADMIN_SESSION_COOKIE: &str = "resursmap_admin_v2";

static ADMIN_SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AdminLevel {
    Helper = 1,
    City = 2,
    Country = 3,
    Continent = 4,
    Owner = 5,
}

impl AdminLevel {
    fn from_i64(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Helper),
            2 => Some(Self::City),
            3 => Some(Self::Country),
            4 => Some(Self::Continent),
            5 => Some(Self::Owner),
            _ => None,
        }
    }

    pub(super) fn number(self) -> i64 {
        self as i64
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Helper => "Помощник группы",
            Self::City => "Администратор города",
            Self::Country => "Администратор страны",
            Self::Continent => "Администратор континента",
            Self::Owner => "Global Owner",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AdminPermission {
    ModerationReview = 0,
    ComplaintsReview = 3,
    GroupsManage = 4,
    AssistantsManage = 5,
    PremiumManage = 6,
    CitiesManage = 7,
    CountriesManage = 10,
    GlobalAdminsManage = 13,
    GlobalUsersManage = 14,
    GlobalSettingsManage = 15,
    SessionsRevoke = 16,
    AuditRead = 17,
    FinanceRead = 18,
    FinancePropose = 19,
    FinanceApprove = 20,
    EmergencyExecute = 21,
    InfrastructureRead = 22,
    IntegrationsManage = 23,
}

impl AdminPermission {
    pub(super) fn all() -> &'static [Self] {
        &[
            Self::ModerationReview,
            Self::ComplaintsReview,
            Self::GroupsManage,
            Self::AssistantsManage,
            Self::PremiumManage,
            Self::CitiesManage,
            Self::CountriesManage,
            Self::GlobalAdminsManage,
            Self::GlobalUsersManage,
            Self::GlobalSettingsManage,
            Self::SessionsRevoke,
            Self::AuditRead,
            Self::FinanceRead,
            Self::FinancePropose,
            Self::FinanceApprove,
            Self::EmergencyExecute,
            Self::InfrastructureRead,
            Self::IntegrationsManage,
        ]
    }

    fn mask(self) -> i64 {
        1_i64 << (self as u32)
    }
}

#[derive(Debug, Clone)]
pub(super) struct AdminContext {
    pub assignment_id: i64,
    pub user_id: i64,
    pub level: AdminLevel,
    pub scope_type: String,
    pub scope_id: i64,
    pub scope_name: String,
    pub permission_mask: i64,
}

impl AdminContext {
    pub(super) fn has_permission(&self, permission: AdminPermission) -> bool {
        self.permission_mask & permission.mask() != 0
    }

    #[allow(dead_code)]
    pub(super) fn can_manage(&self, target: &AdminContext) -> bool {
        if self.user_id == target.user_id {
            return false;
        }

        if target.level == AdminLevel::Owner {
            return false;
        }

        self.level > target.level
    }

    pub(super) fn is_owner(&self) -> bool {
        self.level == AdminLevel::Owner && self.scope_type == "world"
    }
}

pub(super) fn load_admin_context(state: &AppState, user_id: i64) -> Option<AdminContext> {
    let connection = state.db_pool.get().ok()?;

    connection
        .query_row(
            "SELECT
                aa.id,
                aa.user_id,
                aa.role_level,
                aa.scope_type,
                aa.scope_id,
                gs.display_name,
                aa.permission_mask
             FROM admin_assignments AS aa
             JOIN geographic_scopes AS gs
               ON gs.id = aa.scope_id
              AND gs.is_active = 1
             WHERE aa.user_id = ?1
               AND aa.status = 'active'
               AND aa.valid_from <= strftime('%s','now')
               AND (
                    aa.valid_until IS NULL
                    OR aa.valid_until > strftime('%s','now')
               )
             ORDER BY aa.role_level DESC, aa.id
             LIMIT 1",
            params![user_id],
            |row| {
                let raw_level: i64 = row.get(2)?;
                let level = AdminLevel::from_i64(raw_level).ok_or(rusqlite::Error::InvalidQuery)?;

                Ok(AdminContext {
                    assignment_id: row.get(0)?,
                    user_id: row.get(1)?,
                    level,
                    scope_type: row.get(3)?,
                    scope_id: row.get(4)?,
                    scope_name: row.get(5)?,
                    permission_mask: row.get(6)?,
                })
            },
        )
        .optional()
        .ok()
        .flatten()
}

pub(super) fn record_denied_access(
    state: &AppState,
    user_id: i64,
    event_type: &str,
    details: &str,
) {
    let Ok(connection) = state.db_pool.get() else {
        return;
    };

    let assignment_id: Option<i64> = connection
        .query_row(
            "SELECT id
             FROM admin_assignments
             WHERE user_id = ?1
             ORDER BY role_level DESC, id
             LIMIT 1",
            params![user_id],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten();

    let _ = connection.execute(
        "INSERT INTO admin_security_events (
            user_id,
            assignment_id,
            session_public_id,
            event_type,
            severity,
            details
         )
         VALUES (?1, ?2, '', ?3, 'warning', ?4)",
        params![user_id, assignment_id, event_type, details],
    );
}

pub(super) fn scope_is_authorized(
    state: &AppState,
    context: &AdminContext,
    target_scope_id: i64,
) -> bool {
    if context.is_owner() {
        return true;
    }

    let Ok(connection) = state.db_pool.get() else {
        return false;
    };

    connection
        .query_row(
            "WITH RECURSIVE ancestry(id, parent_scope_id) AS (
                SELECT id, parent_scope_id
                FROM geographic_scopes
                WHERE id = ?1
                  AND is_active = 1

                UNION ALL

                SELECT parent.id, parent.parent_scope_id
                FROM geographic_scopes AS parent
                JOIN ancestry AS child
                  ON child.parent_scope_id = parent.id
                WHERE parent.is_active = 1
             )
             SELECT CASE WHEN EXISTS (
                SELECT 1
                FROM ancestry
                WHERE id = ?2
                   OR id IN (
                        SELECT scope_id
                        FROM admin_additional_scopes
                        WHERE assignment_id = ?3
                   )
             )
             THEN 1 ELSE 0 END",
            params![target_scope_id, context.scope_id, context.assignment_id],
            |row| row.get::<_, i64>(0),
        )
        .map(|allowed| allowed == 1)
        .unwrap_or(false)
}

pub(super) fn verify_admin_session(
    state: &AppState,
    headers: &HeaderMap,
    user_id: i64,
    assignment_id: i64,
) -> bool {
    let Some(cookie_header) = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };

    let cookie_prefix = format!("{ADMIN_SESSION_COOKIE}=");

    let Some(token) = cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|value| value.strip_prefix(&cookie_prefix))
    else {
        return false;
    };

    let mut token_parts = token.splitn(2, '.');

    let Some(public_id) = token_parts.next() else {
        return false;
    };

    let Some(secret) = token_parts.next() else {
        return false;
    };

    if public_id.len() != 32
        || secret.len() != 64
        || !public_id.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !secret.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return false;
    }

    let session_hash = hash_admin_session_token(token);
    let now = unix_now();

    let Ok(connection) = state.db_pool.get() else {
        return false;
    };

    let valid: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM admin_sessions
             WHERE session_public_id = ?1
               AND session_hash = ?2
               AND user_id = ?3
               AND assignment_id = ?4
               AND revoked_at IS NULL
               AND expires_at > ?5",
            params![public_id, session_hash, user_id, assignment_id, now],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if valid != 1 {
        return false;
    }

    let _ = connection.execute(
        "UPDATE admin_sessions
         SET last_seen_at = ?2
         WHERE session_public_id = ?1",
        params![public_id, now],
    );

    true
}

pub(super) fn create_admin_session(
    state: &AppState,
    context: &AdminContext,
    headers: &HeaderMap,
) -> Result<String, String> {
    type HmacSha256 = Hmac<Sha256>;

    let now = unix_now();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system_time_error".to_string())?
        .as_nanos();

    let sequence = ADMIN_SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);

    let payload = format!(
        "admin-v2:{}:{}:{}:{}:{}",
        context.user_id, context.assignment_id, now, nanos, sequence
    );

    let mut mac = HmacSha256::new_from_slice(state.admin_key.as_bytes())
        .map_err(|_| "admin_key_error".to_string())?;

    mac.update(payload.as_bytes());

    let secret = hex::encode(mac.finalize().into_bytes());

    let public_digest = Sha256::digest(format!("{payload}:{secret}").as_bytes());

    let public_id = hex::encode(public_digest)[..32].to_string();
    let token = format!("{public_id}.{secret}");
    let session_hash = hash_admin_session_token(&token);
    let expires_at = now + ADMIN_SESSION_TTL_SECONDS;

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .unwrap_or("")
        .chars()
        .take(64)
        .collect::<String>();

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .chars()
        .take(255)
        .collect::<String>();

    let connection = state
        .db_pool
        .get()
        .map_err(|_| "database_unavailable".to_string())?;

    connection
        .execute(
            "INSERT INTO admin_sessions (
                session_public_id,
                user_id,
                assignment_id,
                session_hash,
                ip_address,
                user_agent,
                device_label,
                two_factor_verified,
                reauthenticated_at,
                created_at,
                last_seen_at,
                expires_at
             )
             VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, 'ResursMap Admin',
                0, NULL, ?7, ?7, ?8
             )",
            params![
                public_id,
                context.user_id,
                context.assignment_id,
                session_hash,
                ip_address,
                user_agent,
                now,
                expires_at
            ],
        )
        .map_err(|_| "session_store_failed".to_string())?;

    let _ = connection.execute(
        "DELETE FROM admin_sessions
         WHERE expires_at <= ?1
           OR (
                revoked_at IS NOT NULL
                AND revoked_at <= ?1 - 86400
           )",
        params![now],
    );

    Ok(format!(
        "{ADMIN_SESSION_COOKIE}={token}; \
         Path=/app/center; \
         HttpOnly; Secure; SameSite=Strict; \
         Max-Age={ADMIN_SESSION_TTL_SECONDS}"
    ))
}

fn hash_admin_session_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(user_id: i64, level: AdminLevel, scope_type: &str, mask: i64) -> AdminContext {
        AdminContext {
            assignment_id: user_id,
            user_id,
            level,
            scope_type: scope_type.to_string(),
            scope_id: 1,
            scope_name: "Test".to_string(),
            permission_mask: mask,
        }
    }

    #[test]
    fn owner_can_manage_lower_level() {
        let owner = context(1, AdminLevel::Owner, "world", i64::MAX);

        let continent = context(2, AdminLevel::Continent, "continent", 0);

        assert!(owner.can_manage(&continent));
    }

    #[test]
    fn nobody_can_manage_owner() {
        let owner = context(1, AdminLevel::Owner, "world", i64::MAX);

        let other_owner = context(2, AdminLevel::Owner, "world", i64::MAX);

        assert!(!owner.can_manage(&other_owner));
    }

    #[test]
    fn equal_level_cannot_manage_equal_level() {
        let first = context(1, AdminLevel::Country, "country", 0);

        let second = context(2, AdminLevel::Country, "country", 0);

        assert!(!first.can_manage(&second));
    }

    #[test]
    fn permission_mask_is_enforced() {
        let allowed = context(1, AdminLevel::Helper, "group", 1_i64 << 0);

        let denied = context(2, AdminLevel::Helper, "group", 0);

        assert!(allowed.has_permission(AdminPermission::ModerationReview));

        assert!(!denied.has_permission(AdminPermission::ModerationReview));
    }

    #[test]
    fn only_world_level_five_is_owner() {
        let valid = context(1, AdminLevel::Owner, "world", i64::MAX);

        let invalid = context(2, AdminLevel::Owner, "continent", i64::MAX);

        assert!(valid.is_owner());
        assert!(!invalid.is_owner());
    }

    #[test]
    fn admin_session_hash_is_stable_and_not_plaintext() {
        let token = "0123456789abcdef0123456789abcdef.secret";

        let first = hash_admin_session_token(token);
        let second = hash_admin_session_token(token);

        assert_eq!(first, second);
        assert_ne!(first, token);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn different_admin_tokens_have_different_hashes() {
        let first = hash_admin_session_token("session-one");
        let second = hash_admin_session_token("session-two");

        assert_ne!(first, second);
    }
}
