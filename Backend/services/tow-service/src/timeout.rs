//! 任务超时处理模块
//!
//! 实现任务超时检测、超时处理策略、超时告警

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 超时状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeoutState {
    /// 运行中
    #[default]
    Running,
    /// 即将超时
    Warning,
    /// 已超时
    TimedOut,
    /// 已处理
    Processed,
    /// 已取消
    Cancelled,
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub task_type: String,
    pub status: TimeoutState,
    pub created_at: DateTime<Utc>,
    /// 预期完成时间
    pub expected_at: Option<DateTime<Utc>>,
    /// 实际完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 超时处理策略
    pub strategy: TimeoutStrategy,
    pub retry_count: u32,
    pub max_retries: u32,
    /// 警告阈值（百分比）
    pub warning_threshold: f32,
    /// 超时阈值（秒）
    pub timeout_seconds: u64,
    pub context: HashMap<String, String>,
}

/// 超时处理策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeoutStrategy {
    /// 什么都不做
    None,
    /// 自动重试
    Retry,
    /// 自动取消
    Cancel,
    /// 自动完成
    Complete,
    /// 告警通知
    #[default]
    Notify,
    /// 升级处理
    Escalate,
    /// 回退操作
    Rollback,
}

/// 超时事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutEvent {
    pub event_id: String,
    pub task_id: String,
    pub event_type: TimeoutEventType,
    pub state: TimeoutState,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutEventType {
    /// 创建任务
    Created,
    /// 警告
    Warning,
    /// 超时
    TimedOut,
    /// 处理完成
    Processed,
    /// 重试
    Retried,
    /// 取消
    Cancelled,
    /// 完成
    Completed,
    /// 升级
    Escalated,
}

/// 超时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// 检查间隔（秒）
    pub check_interval_seconds: u64,
    /// 默认超时时间（秒）
    pub default_timeout_seconds: u64,
    /// 警告阈值（百分比）
    pub default_warning_threshold: f32,
    /// 最大重试次数
    pub default_max_retries: u32,
    /// 重试间隔（秒）
    pub default_retry_interval_seconds: u64,
    /// 是否启用自动处理
    pub enable_auto_process: bool,
    /// 是否启用升级
    pub enable_escalation: bool,
    /// 升级延迟（秒）
    pub escalation_delay_seconds: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            default_timeout_seconds: 3600,
            default_warning_threshold: 0.8,
            default_max_retries: 3,
            default_retry_interval_seconds: 300,
            enable_auto_process: true,
            enable_escalation: true,
            escalation_delay_seconds: 3600,
        }
    }
}

/// 超时统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutStats {
    pub total_tasks: usize,
    pub running_tasks: usize,
    pub warning_tasks: usize,
    pub timed_out_tasks: usize,
    pub processed_tasks: usize,
    pub total_timeouts: usize,
    pub total_retries: usize,
    pub avg_processing_time_ms: u64,
}

/// 任务超时管理器
pub struct TimeoutManager {
    /// 任务存储
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    /// 超时事件队列
    events: Arc<RwLock<VecDeque<TimeoutEvent>>>,
    /// 配置
    config: TimeoutConfig,
}

impl TimeoutManager {
    /// 创建新的 `TimeoutManager`
    #[must_use]
    pub fn new(config: TimeoutConfig) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(VecDeque::new())),
            config,
        }
    }

    /// 创建默认配置的管理器
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new(TimeoutConfig::default())
    }

    /// 创建任务
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    /// * `task_type` - 任务类型
    /// * `timeout_seconds` - 超时时间（秒）
    /// * `strategy` - 处理策略
    ///
    /// # Returns
    /// 创建的任务
    pub async fn create_task(
        &self,
        task_id: &str,
        task_type: &str,
        timeout_seconds: Option<u64>,
        strategy: Option<TimeoutStrategy>,
    ) -> Task {
        let now = Utc::now();
        let timeout = timeout_seconds.unwrap_or(self.config.default_timeout_seconds);
        let expected_at = now + Duration::seconds(timeout as i64);
        let warning_threshold = self.config.default_warning_threshold;

        let task = Task {
            task_id: task_id.to_string(),
            task_type: task_type.to_string(),
            status: TimeoutState::Running,
            created_at: now,
            expected_at: Some(expected_at),
            completed_at: None,
            strategy: strategy.unwrap_or(TimeoutStrategy::Notify),
            retry_count: 0,
            max_retries: self.config.default_max_retries,
            warning_threshold,
            timeout_seconds: timeout,
            context: HashMap::new(),
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.to_string(), task.clone());

        // 记录事件
        self.record_event(TimeoutEvent {
            event_id: generate_event_id(),
            task_id: task_id.to_string(),
            event_type: TimeoutEventType::Created,
            state: TimeoutState::Running,
            timestamp: now,
            details: HashMap::new(),
        })
        .await;

        task
    }

    /// 获取任务
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    ///
    /// # Returns
    /// 任务信息
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 完成任务
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    ///
    /// # Returns
    /// 是否成功
    pub async fn complete_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TimeoutState::Processed;
            task.completed_at = Some(Utc::now());

            // 记录事件
            drop(tasks);
            let now = Utc::now();
            self.record_event(TimeoutEvent {
                event_id: generate_event_id(),
                task_id: task_id.to_string(),
                event_type: TimeoutEventType::Completed,
                state: TimeoutState::Processed,
                timestamp: now,
                details: HashMap::new(),
            })
            .await;

            return true;
        }
        false
    }

    /// 取消任务
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    ///
    /// # Returns
    /// 是否成功
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TimeoutState::Cancelled;
            task.completed_at = Some(Utc::now());

            drop(tasks);
            let now = Utc::now();
            self.record_event(TimeoutEvent {
                event_id: generate_event_id(),
                task_id: task_id.to_string(),
                event_type: TimeoutEventType::Cancelled,
                state: TimeoutState::Cancelled,
                timestamp: now,
                details: HashMap::new(),
            })
            .await;

            return true;
        }
        false
    }

    /// 检查任务状态
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    ///
    /// # Returns
    /// 当前状态及是否需要处理
    pub async fn check_task(&self, task_id: &str) -> Option<CheckResult> {
        let tasks = self.tasks.read().await;

        if let Some(task) = tasks.get(task_id) {
            let now = Utc::now();

            // 已完成或已取消
            if task.status == TimeoutState::Processed || task.status == TimeoutState::Cancelled {
                return None;
            }

            let expected_at = task.expected_at?;
            let elapsed = now.signed_duration_since(task.created_at);
            let total_seconds = elapsed.num_seconds() as f32;
            let progress = total_seconds / task.timeout_seconds as f32;

            // 检查是否超时
            if now > expected_at {
                return Some(CheckResult {
                    task_id: task_id.to_string(),
                    status: TimeoutState::TimedOut,
                    progress,
                    needs_action: true,
                    action: Some(task.strategy),
                });
            }

            // 检查是否即将超时
            if progress >= task.warning_threshold {
                return Some(CheckResult {
                    task_id: task_id.to_string(),
                    status: TimeoutState::Warning,
                    progress,
                    needs_action: true,
                    action: Some(TimeoutStrategy::Notify),
                });
            }

            Some(CheckResult {
                task_id: task_id.to_string(),
                status: TimeoutState::Running,
                progress,
                needs_action: false,
                action: None,
            })
        } else {
            None
        }
    }

    /// 处理超时任务
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    ///
    /// # Returns
    /// 处理结果
    pub async fn handle_timeout(&self, task_id: &str) -> HandleResult {
        let mut tasks = self.tasks.write().await;

        let task = match tasks.get_mut(task_id) {
            Some(t) => t,
            None => {
                return HandleResult {
                    success: false,
                    action_taken: TimeoutStrategy::None,
                    message: "任务不存在".to_string(),
                };
            }
        };

        // 检查是否可以重试
        if task.retry_count < task.max_retries {
            task.retry_count += 1;
            task.created_at = Utc::now(); // 重置开始时间
            task.expected_at = Some(Utc::now() + Duration::seconds(task.timeout_seconds as i64));
            task.status = TimeoutState::Running;

            // 保存需要的信息，在释放锁之前
            let current_retry_count = task.retry_count;
            let _task_timeout_seconds = task.timeout_seconds;

            drop(tasks);
            let now = Utc::now();
            self.record_event(TimeoutEvent {
                event_id: generate_event_id(),
                task_id: task_id.to_string(),
                event_type: TimeoutEventType::Retried,
                state: TimeoutState::Running,
                timestamp: now,
                details: {
                    let mut m = HashMap::new();
                    m.insert("retry_count".to_string(), current_retry_count.to_string());
                    m
                },
            })
            .await;

            HandleResult {
                success: true,
                action_taken: TimeoutStrategy::Retry,
                message: format!("任务已重试 (第 {current_retry_count} 次)"),
            }
        } else {
            // 执行策略
            match task.strategy {
                TimeoutStrategy::Cancel => {
                    task.status = TimeoutState::Cancelled;
                    task.completed_at = Some(Utc::now());

                    let now = Utc::now();
                    self.record_event(TimeoutEvent {
                        event_id: generate_event_id(),
                        task_id: task_id.to_string(),
                        event_type: TimeoutEventType::Cancelled,
                        state: TimeoutState::Cancelled,
                        timestamp: now,
                        details: HashMap::new(),
                    })
                    .await;

                    HandleResult {
                        success: true,
                        action_taken: TimeoutStrategy::Cancel,
                        message: "任务已取消".to_string(),
                    }
                }
                TimeoutStrategy::Complete => {
                    task.status = TimeoutState::Processed;
                    task.completed_at = Some(Utc::now());

                    let now = Utc::now();
                    self.record_event(TimeoutEvent {
                        event_id: generate_event_id(),
                        task_id: task_id.to_string(),
                        event_type: TimeoutEventType::Completed,
                        state: TimeoutState::Processed,
                        timestamp: now,
                        details: HashMap::new(),
                    })
                    .await;

                    HandleResult {
                        success: true,
                        action_taken: TimeoutStrategy::Complete,
                        message: "任务已自动完成".to_string(),
                    }
                }
                TimeoutStrategy::Escalate => {
                    let now = Utc::now();
                    self.record_event(TimeoutEvent {
                        event_id: generate_event_id(),
                        task_id: task_id.to_string(),
                        event_type: TimeoutEventType::Escalated,
                        state: TimeoutState::TimedOut,
                        timestamp: now,
                        details: HashMap::new(),
                    })
                    .await;

                    HandleResult {
                        success: true,
                        action_taken: TimeoutStrategy::Escalate,
                        message: "任务已升级".to_string(),
                    }
                }
                _ => {
                    task.status = TimeoutState::TimedOut;

                    let now = Utc::now();
                    self.record_event(TimeoutEvent {
                        event_id: generate_event_id(),
                        task_id: task_id.to_string(),
                        event_type: TimeoutEventType::TimedOut,
                        state: TimeoutState::TimedOut,
                        timestamp: now,
                        details: HashMap::new(),
                    })
                    .await;

                    HandleResult {
                        success: true,
                        action_taken: TimeoutStrategy::Notify,
                        message: "超时通知已发送".to_string(),
                    }
                }
            }
        }
    }

    /// 获取所有需要处理的任务
    ///
    /// # Returns
    /// 任务列表
    pub async fn get_pending_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        let now = Utc::now();

        tasks
            .values()
            .filter(|t| {
                t.status == TimeoutState::Running
                    || t.status == TimeoutState::Warning
                    || t.status == TimeoutState::TimedOut
            })
            .filter(|t| {
                if let Some(expected) = t.expected_at {
                    now > expected
                        || now.signed_duration_since(t.created_at).num_seconds() as f32
                            / t.timeout_seconds as f32
                            >= t.warning_threshold
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> TimeoutStats {
        let tasks = self.tasks.read().await;
        let events = self.events.read().await;

        let total = tasks.len();
        let running = tasks
            .values()
            .filter(|t| t.status == TimeoutState::Running)
            .count();
        let warning = tasks
            .values()
            .filter(|t| t.status == TimeoutState::Warning)
            .count();
        let timed_out = tasks
            .values()
            .filter(|t| t.status == TimeoutState::TimedOut)
            .count();
        let processed = tasks
            .values()
            .filter(|t| t.status == TimeoutState::Processed)
            .count();

        let timeouts = events
            .iter()
            .filter(|e| e.event_type == TimeoutEventType::TimedOut)
            .count();
        let retries = events
            .iter()
            .filter(|e| e.event_type == TimeoutEventType::Retried)
            .count();

        // 计算平均处理时间
        let processing_times: Vec<u64> = tasks
            .values()
            .filter_map(|t| {
                t.completed_at
                    .map(|completed| (completed - t.created_at).num_milliseconds() as u64)
            })
            .collect();

        let avg_time = if processing_times.is_empty() {
            0
        } else {
            processing_times.iter().sum::<u64>() / processing_times.len() as u64
        };

        TimeoutStats {
            total_tasks: total,
            running_tasks: running,
            warning_tasks: warning,
            timed_out_tasks: timed_out,
            processed_tasks: processed,
            total_timeouts: timeouts,
            total_retries: retries,
            avg_processing_time_ms: avg_time,
        }
    }

    /// 获取超时事件历史
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    ///
    /// # Returns
    /// 事件列表
    pub async fn get_task_events(&self, task_id: &str) -> Vec<TimeoutEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.task_id == task_id)
            .cloned()
            .collect()
    }

    /// 记录事件
    async fn record_event(&self, event: TimeoutEvent) {
        let mut events = self.events.write().await;
        events.push_back(event);

        // 限制队列大小
        while events.len() > 10000 {
            events.pop_front();
        }
    }

    /// 清理过期任务
    ///
    /// # Arguments
    /// * `retention_hours` - 保留时间（小时）
    ///
    /// # Returns
    /// 清理的数量
    pub async fn cleanup(&self, retention_hours: u64) -> usize {
        let cutoff = Utc::now() - Duration::hours(retention_hours as i64);
        let mut tasks = self.tasks.write().await;

        let before = tasks.len();
        tasks.retain(|_, t| {
            t.status == TimeoutState::Running
                || t.status == TimeoutState::Warning
                || t.created_at > cutoff
        });
        before - tasks.len()
    }
}

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 生成事件 ID
fn generate_event_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {
            tracing::error!("系统时间早于 UNIX epoch，使用 0 作为时间戳");
            std::time::Duration::from_secs(0)
        })
        .as_nanos();
    format!("evt_{timestamp:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task() {
        let manager = TimeoutManager::default_manager();

        let task = manager
            .create_task(
                "task_001",
                "data_processing",
                Some(60),
                Some(TimeoutStrategy::Retry),
            )
            .await;

        assert_eq!(task.task_id, "task_001");
        assert_eq!(task.status, TimeoutState::Running);
        assert_eq!(task.timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001", "type1", None, None).await;

        let completed = manager.complete_task("task_001").await;
        assert!(completed);

        let task = manager.get_task("task_001").await;
        assert_eq!(task.expect("task should exist").status, TimeoutState::Processed);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001", "type1", None, None).await;

        let cancelled = manager.cancel_task("task_001").await;
        assert!(cancelled);

        let task = manager.get_task("task_001").await;
        assert_eq!(task.expect("task should exist").status, TimeoutState::Cancelled);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001", "type1", None, None).await;
        manager.create_task("task_002", "type2", None, None).await;
        manager.complete_task("task_001").await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.processed_tasks, 1);
        assert_eq!(stats.running_tasks, 1);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001", "type1", None, None).await;
        manager.complete_task("task_001").await;

        // 清理已完成的任务
        let removed = manager.cleanup(0).await;
        assert_eq!(removed, 1);
    }

    #[tokio::test]
    async fn test_events() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001", "type1", None, None).await;
        manager.complete_task("task_001").await;

        let events = manager.get_task_events("task_001").await;
        assert!(events.len() >= 2);
    }
}
