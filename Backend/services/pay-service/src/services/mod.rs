//! 业务逻辑层
//!
//! 提供建行（CCB）和银联（UMS）支付渠道的集成实现。
//! 提供订阅计费管理服务。

pub mod ccb;
pub mod pay;
pub mod subscription;
pub mod ums;
