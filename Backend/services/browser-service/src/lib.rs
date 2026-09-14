//! Browser Service — 浏览器自动化管理微服务
//!
//! 提供 Headless 浏览器池、页面操作、Cookie 持久化能力，
//! 供 social-ops-service 及其他需要外部数据采集的服务共享使用。

pub mod browser_pool;
pub mod cookie_store;
pub mod grpc_handlers;
pub mod grpc_server;
pub mod helpers;
pub mod http_server;
