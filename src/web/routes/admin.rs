use super::super::handlers::administrators_panel;
use super::super::handlers::city_admin_panel;
use super::super::handlers::manage_admin_assignment;
use super::super::handlers::revoke_admin_session;
use super::super::handlers::{admin_geography_group_save, admin_geography_page};
use super::super::handlers::{admin_security_page, admin_step_up_request, admin_step_up_verify};
use super::super::handlers::{
    center_panel,
    {
        admin_approve_resource, admin_bulk_resources, admin_close_report,
        admin_hide_reported_resource, admin_login, admin_login_page,
        admin_reject_reported_resource, admin_reject_resource, admin_reports, admin_resources,
        admin_toggle_active, admin_toggle_premium, admin_toggle_verified,
    },
};
use super::super::handlers::{city_helper_create, city_helper_lifecycle, city_helpers_page};
use super::super::handlers::{create_admin_assignment, new_admin_assignment_page};
use super::super::handlers::{group_helper_panel, group_helper_report_action};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/app/center", get(center_panel))
        .route("/app/center/city", get(city_admin_panel))
        .route(
            "/app/center/city/helpers",
            get(city_helpers_page).post(city_helper_create),
        )
        .route(
            "/app/center/city/helpers/{assignment_id}/{action}",
            post(city_helper_lifecycle),
        )
        .route("/app/center/group", get(group_helper_panel))
        .route(
            "/app/center/group/report/{report_id}",
            post(group_helper_report_action),
        )
        .route("/app/center/geography", get(admin_geography_page))
        .route(
            "/app/center/geography/group",
            post(admin_geography_group_save),
        )
        .route(
            "/app/center/administrators",
            get(administrators_panel).post(create_admin_assignment),
        )
        .route(
            "/app/center/administrators/new",
            get(new_admin_assignment_page),
        )
        .route(
            "/app/center/administrators/{assignment_id}/{action}",
            post(manage_admin_assignment),
        )
        .route("/app/center/security", get(admin_security_page))
        .route("/app/center/security/request", post(admin_step_up_request))
        .route("/app/center/security/verify", post(admin_step_up_verify))
        .route(
            "/app/center/sessions/{session_public_id}/revoke",
            post(revoke_admin_session),
        )
        .route("/app/admin/login", get(admin_login_page).post(admin_login))
        .route("/app/admin/resources", get(admin_resources))
        .route("/app/admin/reports", get(admin_reports))
        .route("/app/admin/report/{id}/close", post(admin_close_report))
        .route(
            "/app/admin/report/{id}/hide-resource",
            post(admin_hide_reported_resource),
        )
        .route(
            "/app/admin/report/{id}/reject-resource",
            post(admin_reject_reported_resource),
        )
        .route("/app/admin/resources/bulk", post(admin_bulk_resources))
        .route(
            "/app/admin/resource/{id}/toggle-verified",
            post(admin_toggle_verified),
        )
        .route(
            "/app/admin/resource/{id}/approve",
            post(admin_approve_resource),
        )
        .route(
            "/app/admin/resource/{id}/reject",
            post(admin_reject_resource),
        )
        .route(
            "/app/admin/resource/{id}/toggle-premium",
            post(admin_toggle_premium),
        )
        .route(
            "/app/admin/resource/{id}/toggle-active",
            post(admin_toggle_active),
        )
}
