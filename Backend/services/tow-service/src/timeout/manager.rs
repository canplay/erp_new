//! 超时管理器实现
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Duration, Utc};
use super::types::*;
use super::results::{CheckResult, HandleResult};

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
                message: format!("任务已重试 (第 {current_retry_count} 次)" ),
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

/// 生成事件 ID
pub(crate) fn generate_event_id() -> String {
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
