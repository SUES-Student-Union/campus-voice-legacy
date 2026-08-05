use crate::handlers::{
    admin, attachments, auth, department, departments, issues, notifications, public, review, stats,
};
use crate::state::AppState;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/wechat", post(auth::wechat_login))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .route("/issues", post(issues::create_issue))
        .route("/my/issues", get(issues::my_issues))
        .route("/my/issues/{id}", get(issues::my_issue_detail))
        .route("/my/issues/{id}/supplement", post(issues::supplement_issue))
        .route("/my/issues/{id}/rating", post(issues::rate_issue))
        .route(
            "/issues/{id}/attachments",
            post(attachments::upload_attachment),
        )
        .route("/attachments/{id}", get(attachments::download_attachment))
        .route("/admin/issues", get(admin::list_issues))
        .route("/admin/issues/{id}", get(admin::admin_issue_detail))
        .route("/admin/issues/{id}/accept", post(admin::accept_issue))
        .route("/admin/issues/{id}/assign", post(admin::assign_issue))
        .route(
            "/admin/issues/{id}/request-supplement",
            post(admin::request_supplement),
        )
        .route("/admin/issues/{id}/close", post(admin::close_issue))
        .route("/admin/issues/{id}/merge", post(admin::merge_issue))
        .route("/admin/issues/{id}/review", post(review::save_review))
        .route("/admin/audit-logs", get(admin::list_audit_logs))
        .route("/admin/dashboard", get(stats::dashboard))
        .route("/admin/departments", post(departments::create_department))
        .route("/departments", get(departments::list_departments))
        .route("/categories", get(departments::list_categories))
        .route("/department/issues", get(department::dept_issues))
        .route(
            "/department/issues/{id}/progress",
            post(department::update_progress),
        )
        .route(
            "/department/issues/{id}/reply",
            post(department::reply_issue),
        )
        .route("/public/cases", get(public::list_public))
        .route("/public/cases/{id}", get(public::public_detail))
        .route("/notifications", get(notifications::list_notifications))
        .route(
            "/notifications/{id}/read",
            post(notifications::mark_notification_read),
        );

    Router::new()
        .nest("/api/v1", api)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
