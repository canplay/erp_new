//! Route modules - organized by business domain
//!
//! Each module defines its own handlers and exports a `routes()` function
//! that returns an `axum::Router<Arc<AppState>>`.
//! The `all_routes()` function merges all sub-routers into one.

pub mod health_routes;
pub mod observability_routes;
pub mod permission_routes;
pub mod auth_routes;
pub mod user_admin_routes;
pub mod role_routes;
pub mod dept_routes;
pub mod content_routes;
pub mod config_routes;
pub mod dictionary_routes;
pub mod security_routes;
pub mod scheduled_task_routes;
pub mod export_report_routes;
pub mod ebike_routes;
pub mod device_integration_routes;
pub mod payment_routes;
pub mod lpr_all_routes;
pub mod api_key_routes;
pub mod audit_routes;
pub mod clean_routes;
pub mod cms_routes;
pub mod device_routes;
pub mod feedback_routes;
pub mod file_routes;
pub mod message_routes;
pub mod tenant_routes;
pub mod tow_routes;
pub mod workflow_routes;
pub mod xlt_routes;
pub mod helpers;

use std::sync::Arc;
use axum::{Json, Router};
use crate::AppState;

async fn metrics_handler() -> Json<serde_json::Value> {
    Json(common::metrics::get_metrics())
}

pub use health_routes::routes as health_routes;
pub use observability_routes::routes as observability_routes;
pub use permission_routes::routes as permission_routes;
pub use auth_routes::routes as auth_routes;
pub use user_admin_routes::routes as user_admin_routes;
pub use role_routes::routes as role_routes;
pub use dept_routes::routes as dept_routes;
pub use content_routes::routes as content_routes;
pub use config_routes::routes as config_routes;
pub use dictionary_routes::routes as dictionary_routes;
pub use security_routes::routes as security_routes;
pub use scheduled_task_routes::routes as scheduled_task_routes;
pub use export_report_routes::routes as export_report_routes;
pub use ebike_routes::routes as ebike_routes;
pub use device_integration_routes::routes as device_integration_routes;
pub use payment_routes::routes as payment_routes;
pub use lpr_all_routes::routes as lpr_all_routes;
pub use api_key_routes::routes as api_key_routes;
pub use audit_routes::routes as audit_routes;
pub use clean_routes::routes as clean_routes;
pub use cms_routes::routes as cms_routes;
pub use device_routes::routes as device_routes;
pub use feedback_routes::routes as feedback_routes;
pub use file_routes::routes as file_routes;
pub use message_routes::routes as message_routes;
pub use tenant_routes::routes as tenant_routes;
pub use tow_routes::routes as tow_routes;
pub use workflow_routes::routes as workflow_routes;
pub use xlt_routes::routes as xlt_routes;

/// Merge all service route modules into a single router
pub fn all_routes() -> Router<Arc<AppState>> {
    health_routes()
        .merge(observability_routes())
        .merge(permission_routes())
        .merge(auth_routes())
        .merge(user_admin_routes())
        .merge(role_routes())
        .merge(dept_routes())
        .merge(content_routes())
        .merge(config_routes())
        .merge(dictionary_routes())
        .merge(security_routes())
        .merge(scheduled_task_routes())
        .merge(export_report_routes())
        .merge(ebike_routes())
        .merge(device_integration_routes())
        .merge(payment_routes())
        .merge(lpr_all_routes())
        .merge(api_key_routes())
        .merge(audit_routes())
        .merge(clean_routes())
        .merge(cms_routes())
        .merge(device_routes())
        .merge(feedback_routes())
        .merge(file_routes())
        .merge(message_routes())
        .merge(tenant_routes())
        .merge(tow_routes())
        .merge(workflow_routes())
        .merge(xlt_routes())
        .route("/metrics", axum::routing::get(metrics_handler))
}
