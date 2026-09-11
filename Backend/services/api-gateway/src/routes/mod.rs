//! Route modules - one module per upstream service
//!
//! Each module defines its own handlers and exports a `routes()` function
//! that returns an `axum::Router<Arc<AppState>>`.
//! The `all_routes()` function merges all sub-routers into one.

pub mod api_key_routes;
pub mod audit_routes;
pub mod auth_routes;
pub mod browser_routes;
pub mod clean_routes;
pub mod cms_routes;
pub mod ctp_routes;
pub mod device_routes;
pub mod ebike_routes;
pub mod feedback_routes;
pub mod file_routes;
pub mod health_routes;
pub mod helpers;
pub mod hik_routes;
pub mod lpr_routes;
pub mod lpr_query_routes;
pub mod message_routes;
pub mod pay_routes;
pub mod security_routes;
pub mod social_ops_routes;
pub mod system_routes;
pub mod tenant_routes;
pub mod tow_routes;
pub mod user_routes;
pub mod workflow_routes;
pub mod xlt_routes;
pub mod options_routes;
pub use options_routes::routes as options_routes;

use std::sync::Arc;
use axum::Router;
use crate::AppState;

pub use api_key_routes::routes as api_key_routes;
pub use audit_routes::routes as audit_routes;
pub use auth_routes::routes as auth_routes;
pub use browser_routes::routes as browser_routes;
pub use clean_routes::routes as clean_routes;
pub use cms_routes::routes as cms_routes;
pub use ctp_routes::routes as ctp_routes;
pub use device_routes::routes as device_routes;
pub use ebike_routes::routes as ebike_routes;
pub use feedback_routes::routes as feedback_routes;
pub use file_routes::routes as file_routes;
pub use health_routes::routes as health_routes;
pub use hik_routes::routes as hik_routes;
pub use lpr_routes::routes as lpr_routes;
pub use lpr_query_routes::routes as lpr_query_routes;
pub use message_routes::routes as message_routes;
pub use pay_routes::routes as pay_routes;
pub use security_routes::routes as security_routes;
pub use social_ops_routes::routes as social_ops_routes;
pub use system_routes::routes as system_routes;
pub use tenant_routes::routes as tenant_routes;
pub use tow_routes::routes as tow_routes;
pub use user_routes::routes as user_routes;
pub use workflow_routes::routes as workflow_routes;
pub use xlt_routes::routes as xlt_routes;

/// Merge all service route modules into a single router
pub fn all_routes() -> Router<Arc<AppState>> {
    health_routes()
        .merge(system_routes())
        .merge(auth_routes())
        .merge(browser_routes())
        .merge(user_routes())
        .merge(cms_routes())
        .merge(message_routes())
        .merge(feedback_routes())
        .merge(tenant_routes())
        .merge(file_routes())
        .merge(workflow_routes())
        .merge(audit_routes())
        .merge(api_key_routes())
        .merge(social_ops_routes())
        .merge(device_routes())
        .merge(security_routes())
        .merge(lpr_routes())
        .merge(lpr_query_routes())
        .merge(tow_routes())
        .merge(pay_routes())
        .merge(clean_routes())
        .merge(ctp_routes())
        .merge(ebike_routes())
        .merge(hik_routes())
        .merge(xlt_routes())
        .merge(options_routes())
}
