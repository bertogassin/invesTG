use super::super::handlers::administrators_panel;
use super::super::handlers::{
    center_panel,
    {
        admin_approve_resource, admin_bulk_resources, admin_close_report,
        admin_hide_reported_resource, admin_login, admin_login_page,
        admin_reject_reported_resource, admin_reject_resource, admin_reports, admin_resources,
        admin_toggle_active, admin_toggle_premium, admin_toggle_verified,
    },
};
use crate::state::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/app/center", get(center_panel))
        .route("/app/center/administrators", get(administrators_panel))
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
