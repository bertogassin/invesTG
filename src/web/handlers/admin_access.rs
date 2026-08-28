use crate::state::app_state::AppState;
use rusqlite::{params, OptionalExtension};

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
}
