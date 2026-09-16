//! 敏感操作实时告警模块
//!
//! 实现敏感操作监控、实时告警、异常访问检测

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AlertLevel {
    /// 信息
    #[default]
    Info,
    /// 警告
    Warning,
    /// 危险
    Danger,
    /// 严重
    Critical,
}

/// 告警类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AlertType {
    /// 登录失败
    LoginFailed,
    /// 异常访问
    AbnormalAccess,
    /// 权限提升
    PrivilegeEscalation,
    /// 数据访问异常
    DataAccessAnomaly,
    /// 批量操作
    BatchOperation,
    /// 敏感操作
    #[default]
    SensitiveOperation,
    /// API 滥用
    ApiAbuse,
    /// 账户异常
    AccountAnomaly,
}

/// 告警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub alert_id: String,
    pub alert_type: AlertType,
    pub level: AlertLevel,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub details: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub rule_id: String,
    pub name: String,
    pub alert_type: AlertType,
    pub level: AlertLevel,
    /// 触发条件
    pub conditions: Vec<AlertCondition>,
    /// 阈值（触发告警的次数）
    pub threshold: u32,
    /// 时间窗口（秒）
    pub time_window_seconds: u32,
    /// 是否启用
    pub enabled: bool,
    /// 告警消息模板
    pub message_template: String,
}

/// 告警条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConditionOperator {
    #[default]
    Eq, // 等于
    Ne,       // 不等于
    Gt,       // 大于
    Lt,       // 小于
    Contains, // 包含
    StartsWith,
    EndsWith,
    Regex,
}

/// 登录失败事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginFailureEvent {
    pub user_id: String,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
    pub attempts: u32,
}

/// 异常访问事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalAccessEvent {
    pub user_id: String,
    pub ip_address: String,
    pub resource: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub risk_score: f32,
    pub details: HashMap<String, String>,
}

/// 告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// 是否启用
    pub enabled: bool,
    /// 登录失败告警阈值
    pub login_failure_threshold: u32,
    /// 登录失败时间窗口（秒）
    pub login_failure_window_seconds: u32,
    /// 异常访问告警阈值
    pub abnormal_access_threshold: u32,
    /// 异常访问时间窗口（秒）
    pub abnormal_access_window_seconds: u32,
    /// 批量操作告警阈值
    pub batch_operation_threshold: u32,
    /// 是否启用实时推送
    pub push_enabled: bool,
    /// 保留天数
    pub retention_days: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            login_failure_threshold: 5,
            login_failure_window_seconds: 300, // 5 分钟
            abnormal_access_threshold: 10,
            abnormal_access_window_seconds: 3600, // 1 小时
            batch_operation_threshold: 100,
            push_enabled: true,
            retention_days: 90,
        }
    }
}

/// 告警历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistory {
    pub alert_id: String,
    pub event: AlertEvent,
    pub notified_users: Vec<String>,
    pub notified_channels: Vec<String>,
    pub resolved: bool,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
}

/// 安全告警管理器
pub struct AlertManager {
    /// 告警规则
    rules: Arc<RwLock<Vec<AlertRule>>>,
    /// 告警事件
    events: Arc<RwLock<HashMap<String, AlertEvent>>>,
    /// 登录失败历史
    login_failures: Arc<RwLock<HashMap<String, Vec<LoginFailureEvent>>>>,
    /// 异常访问历史
    abnormal_access: Arc<RwLock<HashMap<String, Vec<AbnormalAccessEvent>>>>,
    /// 告警历史
    #[allow(dead_code)]
    alert_history: Arc<RwLock<HashMap<String, AlertHistory>>>,
    // NOTE: kept for future use (querying/alert details)
    /// 配置
    config: AlertConfig,
}

impl AlertManager {
    /// 创建新的 `AlertManager`
    #[must_use]
    pub fn new(config: AlertConfig) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
            login_failures: Arc::new(RwLock::new(HashMap::new())),
            abnormal_access: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 创建默认配置的管理器
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new(AlertConfig::default())
    }

    /// 添加告警规则
    ///
    /// # Arguments
    /// * `rule` - 告警规则
    ///
    /// # Returns
    /// 是否成功
    pub async fn add_rule(&self, rule: AlertRule) -> bool {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        true
    }

    /// 获取告警规则
    ///
    /// # Returns
    /// 规则列表
    pub async fn get_rules(&self) -> Vec<AlertRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// 检查登录失败
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `ip_address` - IP 地址
    /// * `reason` - 失败原因
    ///
    /// # Returns
    /// 是否触发了告警
    pub async fn check_login_failure(
        &self,
        user_id: &str,
        ip_address: &str,
        reason: &str,
    ) -> Option<AlertEvent> {
        let mut failures = self.login_failures.write().await;

        let now = Utc::now();
        let cutoff =
            now - chrono::Duration::seconds(i64::from(self.config.login_failure_window_seconds));

        // 获取用户失败记录
        let user_failures = failures.entry(user_id.to_string()).or_insert_with(Vec::new);

        // 清理过期记录
        user_failures.retain(|f| f.timestamp > cutoff);

        // 添加新失败记录
        let attempts = user_failures.len() as u32 + 1;
        user_failures.push(LoginFailureEvent {
            user_id: user_id.to_string(),
            ip_address: ip_address.to_string(),
            timestamp: now,
            reason: reason.to_string(),
            attempts,
        });

        // 检查是否触发告警
        if attempts >= self.config.login_failure_threshold {
            let alert = AlertEvent {
                alert_id: generate_alert_id(),
                alert_type: AlertType::LoginFailed,
                level: if attempts >= self.config.login_failure_threshold * 2 {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                },
                user_id: Some(user_id.to_string()),
                ip_address: Some(ip_address.to_string()),
                resource: None,
                action: "login".to_string(),
                details: {
                    let mut m = HashMap::new();
                    m.insert("attempts".to_string(), attempts.to_string());
                    m.insert("reason".to_string(), reason.to_string());
                    m.insert(
                        "window_seconds".to_string(),
                        self.config.login_failure_window_seconds.to_string(),
                    );
                    m
                },
                timestamp: now,
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
            };

            // 存储事件
            let mut events = self.events.write().await;
            events.insert(alert.alert_id.clone(), alert.clone());

            Some(alert)
        } else {
            None
        }
    }

    /// 检查异常访问
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `ip_address` - IP 地址
    /// * `resource` - 资源
    /// * `action` - 操作
    /// * `risk_score` - 风险评分
    ///
    /// # Returns
    /// 是否触发了告警
    pub async fn check_abnormal_access(
        &self,
        user_id: &str,
        ip_address: &str,
        resource: &str,
        action: &str,
        risk_score: f32,
    ) -> Option<AlertEvent> {
        let mut access_log = self.abnormal_access.write().await;

        let now = Utc::now();
        let cutoff =
            now - chrono::Duration::seconds(i64::from(self.config.abnormal_access_window_seconds));

        // 获取用户访问记录
        let key = format!("{user_id}:{ip_address}" );
        let user_access = access_log.entry(key).or_insert_with(Vec::new);

        // 清理过期记录
        user_access.retain(|a| a.timestamp > cutoff);

        // 添加新记录
        user_access.push(AbnormalAccessEvent {
            user_id: user_id.to_string(),
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            timestamp: now,
            risk_score,
            details: HashMap::new(),
        });

        // 检查风险评分
        if risk_score >= 0.8 {
            let alert = AlertEvent {
                alert_id: generate_alert_id(),
                alert_type: AlertType::AbnormalAccess,
                level: if risk_score >= 0.95 {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Danger
                },
                user_id: Some(user_id.to_string()),
                ip_address: Some(ip_address.to_string()),
                resource: Some(resource.to_string()),
                action: action.to_string(),
                details: {
                    let mut m = HashMap::new();
                    m.insert("risk_score".to_string(), risk_score.to_string());
                    m
                },
                timestamp: now,
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
            };

            let mut events = self.events.write().await;
            events.insert(alert.alert_id.clone(), alert.clone());

            Some(alert)
        } else {
            None
        }
    }

    /// 记录敏感操作
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `ip_address` - IP 地址
    /// * `action` - 操作
    /// * `resource` - 资源
    /// * `details` - 详情
    ///
    /// # Returns
    /// 告警事件
    pub async fn record_sensitive_operation(
        &self,
        user_id: &str,
        ip_address: &str,
        action: &str,
        resource: &str,
        details: HashMap<String, String>,
    ) -> AlertEvent {
        let now = Utc::now();

        let alert = AlertEvent {
            alert_id: generate_alert_id(),
            alert_type: AlertType::SensitiveOperation,
            level: AlertLevel::Info,
            user_id: Some(user_id.to_string()),
            ip_address: Some(ip_address.to_string()),
            resource: Some(resource.to_string()),
            action: action.to_string(),
            details,
            timestamp: now,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };

        let mut events = self.events.write().await;
        events.insert(alert.alert_id.clone(), alert.clone());

        alert
    }

    /// 确认告警
    ///
    /// # Arguments
    /// * `alert_id` - 告警 ID
    /// * `acknowledged_by` - 确认人
    ///
    /// # Returns
    /// 是否成功
    pub async fn acknowledge(&self, alert_id: &str, acknowledged_by: &str) -> bool {
        let mut events = self.events.write().await;

        if let Some(event) = events.get_mut(alert_id) {
            event.acknowledged = true;
            event.acknowledged_by = Some(acknowledged_by.to_string());
            event.acknowledged_at = Some(Utc::now());
            return true;
        }
        false
    }

    /// 获取告警事件
    ///
    /// # Arguments
    /// * `alert_id` - 告警 ID
    ///
    /// # Returns
    /// 告警事件
    pub async fn get_alert(&self, alert_id: &str) -> Option<AlertEvent> {
        let events = self.events.read().await;
        events.get(alert_id).cloned()
    }

    /// 获取未确认的告警
    ///
    /// # Returns
    /// 告警列表
    pub async fn get_unacknowledged(&self) -> Vec<AlertEvent> {
        let events = self.events.read().await;
        events
            .values()
            .filter(|e| !e.acknowledged)
            .cloned()
            .collect()
    }

    /// 按级别获取告警
    ///
    /// # Arguments
    /// * `level` - 告警级别
    ///
    /// # Returns
    /// 告警列表
    pub async fn get_by_level(&self, level: AlertLevel) -> Vec<AlertEvent> {
        let events = self.events.read().await;
        events
            .values()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// 按类型获取告警
    ///
    /// # Arguments
    /// * `alert_type` - 告警类型
    ///
    /// # Returns
    /// 告警列表
    pub async fn get_by_type(&self, alert_type: AlertType) -> Vec<AlertEvent> {
        let events = self.events.read().await;
        events
            .values()
            .filter(|e| e.alert_type == alert_type)
            .cloned()
            .collect()
    }

    /// 获取用户的告警历史
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    ///
    /// # Returns
    /// 告警列表
    pub async fn get_user_alerts(&self, user_id: &str) -> Vec<AlertEvent> {
        let events = self.events.read().await;
        events
            .values()
            .filter(|e| e.user_id.as_ref() == Some(&user_id.to_string()))
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> AlertStats {
        let events = self.events.read().await;
        let total = events.len();

        let unacknowledged = events.values().filter(|e| !e.acknowledged).count();
        let critical = events
            .values()
            .filter(|e| e.level == AlertLevel::Critical)
            .count();
        let danger = events
            .values()
            .filter(|e| e.level == AlertLevel::Danger)
            .count();
        let warning = events
            .values()
            .filter(|e| e.level == AlertLevel::Warning)
            .count();

        let by_type: HashMap<String, usize> = events.values().fold(HashMap::new(), |mut acc, e| {
            *acc.entry(format!("{:?}" , e.alert_type)).or_insert(0) += 1;
            acc
        });

        AlertStats {
            total_alerts: total,
            unacknowledged,
            critical,
            danger,
            warning,
            by_type,
        }
    }

    /// 清理过期告警
    ///
    /// # Arguments
    /// * `retention_days` - 保留天数
    ///
    /// # Returns
    /// 清理的数量
    pub async fn cleanup_expired(&self, retention_days: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut events = self.events.write().await;

        let before = events.len();
        events.retain(|_, e| e.timestamp > cutoff || !e.acknowledged);
        before - events.len()
    }
}

/// 告警统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub total_alerts: usize,
    pub unacknowledged: usize,
    pub critical: usize,
    pub danger: usize,
    pub warning: usize,
    pub by_type: HashMap<String, usize>,
}

/// 生成告警 ID
fn generate_alert_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {
            tracing::error!("系统时间早于 UNIX epoch" );
            std::time::Duration::from_secs(0)
        })
        .as_nanos();
    format!("alert_{timestamp:016x}" )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_rule() {
        let manager = AlertManager::default_manager();

        let rule = AlertRule {
            rule_id: "rule_001".to_string(),
            name: "多次登录失败告警".to_string(),
            alert_type: AlertType::LoginFailed,
            level: AlertLevel::Warning,
            conditions: vec![AlertCondition {
                field: "attempts".to_string(),
                operator: ConditionOperator::Gt,
                value: "5".to_string(),
            }],
            threshold: 5,
            time_window_seconds: 300,
            enabled: true,
            message_template: "用户 {user_id} 在 {time_window} 秒内登录失败 {attempts} 次"
                .to_string(),
        };

        let result = manager.add_rule(rule).await;
        assert!(result);

        let rules = manager.get_rules().await;
        assert_eq!(rules.len(), 1);
    }

    #[tokio::test]
    async fn test_login_failure_alert() {
        let manager = AlertManager::default_manager();

        // 模拟多次登录失败
        for i in 1..=5 {
            let alert = manager
                .check_login_failure("user_001" , "192.0.2.100" , "密码错误" )
                .await;

            if i == 5 {
                assert!(alert.is_some());
                let a = alert.expect("alert should exist" );
                assert_eq!(a.alert_type, AlertType::LoginFailed);
                assert_eq!(a.level, AlertLevel::Warning);
            } else {
                assert!(alert.is_none());
            }
        }
    }

    #[tokio::test]
    async fn test_abnormal_access_alert() {
        let manager = AlertManager::default_manager();

        // 高风险访问
        let alert = manager
            .check_abnormal_access("user_001" , "192.0.2.100" , "/admin/settings" , "read" , 0.9)
            .await;

        assert!(alert.is_some());
        assert_eq!(alert.expect("alert should exist" ).alert_type, AlertType::AbnormalAccess);
    }

    #[tokio::test]
    async fn test_sensitive_operation() {
        let manager = AlertManager::default_manager();

        let mut details = HashMap::new();
        details.insert("table".to_string(), "users".to_string());
        details.insert("operation".to_string(), "delete".to_string());

        let alert = manager
            .record_sensitive_operation(
                "user_001" ,
                "192.0.2.100" ,
                "delete" ,
                "/api/users" ,
                details,
            )
            .await;

        assert_eq!(alert.alert_type, AlertType::SensitiveOperation);
    }

    #[tokio::test]
    async fn test_acknowledge() {
        let manager = AlertManager::default_manager();

        let alert = manager
            .record_sensitive_operation(
                "user_001" ,
                "192.0.2.100" ,
                "delete" ,
                "/api/users" ,
                HashMap::new(),
            )
            .await;

        let acknowledged = manager.acknowledge(&alert.alert_id, "admin_001" ).await;
        assert!(acknowledged);

        let fetched = manager.get_alert(&alert.alert_id).await;
        assert!(fetched.expect("fetched alert should exist" ).acknowledged);
    }

    #[tokio::test]
    async fn test_get_unacknowledged() {
        let manager = AlertManager::default_manager();

        // 创建多个告警
        let a1 = manager
            .record_sensitive_operation("u1" , "ip1" , "op1" , "r1" , HashMap::new())
            .await;
        let a2 = manager
            .record_sensitive_operation("u2" , "ip2" , "op2" , "r2" , HashMap::new())
            .await;

        // 确认一个
        manager.acknowledge(&a1.alert_id, "admin" ).await;

        let unacknowledged = manager.get_unacknowledged().await;
        assert_eq!(unacknowledged.len(), 1);
        assert_eq!(unacknowledged[0].alert_id, a2.alert_id);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = AlertManager::default_manager();

        // record_sensitive_operation 总是创建告警
        manager
            .record_sensitive_operation("u1" , "ip1" , "op1" , "r1" , HashMap::new())
            .await;

        // check_login_failure 需要达到阈值才会触发告警（默认阈值是 5）
        // 所以这里只验证 record_sensitive_operation 的效果
        manager.check_login_failure("u2" , "ip2" , "failed" ).await;

        let stats = manager.get_stats().await;
        // 只期望 1 个告警，因为 check_login_failure 未达到阈值
        assert_eq!(stats.total_alerts, 1);
    }
}
