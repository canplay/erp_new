//! 拖车超时管理模块
//!
//! 提供任务超时状态机、超时策略、事件处理

mod manager;
mod results;
mod tests;
mod types;

pub use types::TimeoutManager;
pub use results::{CheckResult, HandleResult};
pub use types::{
    Task, TimeoutConfig, TimeoutEvent, TimeoutEventType, TimeoutState, TimeoutStats,
    TimeoutStrategy,
};
