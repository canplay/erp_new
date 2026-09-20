use serde::{Deserialize, Serialize};
use super::types::{TimeoutState, TimeoutStrategy};

pub struct CheckResult {
    pub task_id: String,
    pub status: TimeoutState,
    pub progress: f32,
    pub needs_action: bool,
    pub action: Option<TimeoutStrategy>,
}

/// 处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleResult {
    pub success: bool,
    pub action_taken: TimeoutStrategy,
    pub message: String,
}
