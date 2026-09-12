//! 反馈分类与优先级模块
//!
//! 实现反馈分类、优先级评估、状态流转管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 反馈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// 功能建议
    Suggestion,
    /// Bug 报告
    Bug,
    /// 投诉
    Complaint,
    /// 咨询
    Inquiry,
    /// 赞誉
    Praise,
    /// 其他
    Other,
}

impl Default for FeedbackType {
    fn default() -> Self {
        FeedbackType::Suggestion
    }
}

/// 反馈优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// 紧急
    Critical,
    /// 高
    High,
    /// 中
    Medium,
    /// 低
    Low,
    /// 最低
    Lowest,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

impl Priority {
    /// 获取优先级数值（用于排序）
    pub fn level(&self) -> i32 {
        match self {
            Priority::Critical => 5,
            Priority::High => 4,
            Priority::Medium => 3,
            Priority::Low => 2,
            Priority::Lowest => 1,
        }
    }
}

/// 反馈状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackState {
    /// 新建
    New,
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 已回复
    Replied,
    /// 已解决
    Resolved,
    /// 已关闭
    Closed,
    /// 已驳回
    Rejected,
}

impl Default for FeedbackState {
    fn default() -> Self {
        FeedbackState::New
    }
}

/// 反馈项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: String,
    pub title: String,
    pub content: String,
    pub feedback_type: FeedbackType,
    pub priority: Priority,
    pub state: FeedbackState,
    pub user_id: String,
    pub assigned_to: Option<String>,
    pub tags: Vec<String>,
    /// 影响范围评分（1-10）
    pub impact_score: u8,
    /// 紧急程度评分（1-10）
    pub urgency_score: u8,
    /// 计算得出的优先级
    pub calculated_priority: Priority,
    /// 响应时限（小时）
    pub response_deadline_hours: u32,
    /// 解决时限（小时）
    pub resolution_deadline_hours: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// 分类规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub rule_id: String,
    pub name: String,
    pub feedback_type: FeedbackType,
    /// 匹配条件（关键词列表）
    pub keywords: Vec<String>,
    /// 正则表达式匹配
    pub regex_pattern: Option<String>,
    /// 分配的优先级
    pub assigned_priority: Priority,
    /// 分配的部门/人员
    pub assigned_to: Option<String>,
    /// 分配的标签
    pub tags: Vec<String>,
    /// 优先级
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
}

impl ClassificationRule {
    /// 检查是否匹配
    pub fn matches(&self, content: &str, title: &str) -> bool {
        let text = format!("{} {}", title, content).to_lowercase();

        // 检查关键词
        for keyword in &self.keywords {
            if text.contains(&keyword.to_lowercase()) {
                return true;
            }
        }

        // 检查正则表达式
        if let Some(pattern) = &self.regex_pattern {
            if let Ok(re) = regex::Regex::new(pattern) {
                return re.is_match(&text);
            }
        }

        false
    }
}

/// 优先级计算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityConfig {
    /// 影响权重
    pub impact_weight: f32,
    /// 紧急程度权重
    pub urgency_weight: f32,
    /// 严重程度权重
    pub severity_weight: f32,
    /// 优先级阈值配置
    pub threshold_critical: f32,
    pub threshold_high: f32,
    pub threshold_medium: f32,
    pub threshold_low: f32,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            impact_weight: 0.3,
            urgency_weight: 0.4,
            severity_weight: 0.3,
            threshold_critical: 8.0,
            threshold_high: 6.0,
            threshold_medium: 4.0,
            threshold_low: 2.0,
        }
    }
}

/// SLA 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConfig {
    pub feedback_type: FeedbackType,
    pub priority: Priority,
    /// 首次响应时限（小时）
    pub first_response_hours: u32,
    /// 解决时限（小时）
    pub resolution_hours: u32,
    /// 是否启用
    pub enabled: bool,
}

impl Default for SlaConfig {
    fn default() -> Self {
        Self {
            feedback_type: FeedbackType::Suggestion,
            priority: Priority::Medium,
            first_response_hours: 24,
            resolution_hours: 72,
            enabled: true,
        }
    }
}

/// 反馈管理器
pub struct FeedbackManager {
    /// 反馈存储
    feedbacks: Arc<RwLock<HashMap<String, Feedback>>>,
    /// 分类规则
    rules: Arc<RwLock<Vec<ClassificationRule>>>,
    /// SLA 配置
    sla_configs: Arc<RwLock<Vec<SlaConfig>>>,
    /// 优先级计算配置
    priority_config: PriorityConfig,
}

impl FeedbackManager {
    /// 创建新的 FeedbackManager
    pub fn new(config: PriorityConfig) -> Self {
        Self {
            feedbacks: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            sla_configs: Arc::new(RwLock::new(Vec::new())),
            priority_config: config,
        }
    }

    /// 创建默认配置的管理器
    pub fn default_manager() -> Self {
        Self::new(PriorityConfig::default())
    }

    /// 添加分类规则
    ///
    /// # Arguments
    /// * `rule` - 分类规则
    ///
    /// # Returns
    /// 是否成功
    pub async fn add_rule(&self, rule: ClassificationRule) -> bool {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        true
    }

    /// 获取分类规则
    ///
    /// # Returns
    /// 规则列表
    pub async fn get_rules(&self) -> Vec<ClassificationRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// 根据内容自动分类反馈
    ///
    /// # Arguments
    /// * `title` - 标题
    /// * `content` - 内容
    ///
    /// # Returns
    /// 分类结果
    pub async fn classify(&self, title: &str, content: &str) -> ClassificationResult {
        let rules = self.rules.read().await;

        let mut best_match: Option<&ClassificationRule> = None;
        let mut best_priority: u32 = 0;

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if rule.matches(content, title) {
                if best_match.is_none() || rule.priority > best_priority {
                    best_match = Some(rule);
                    best_priority = rule.priority;
                }
            }
        }

        if let Some(rule) = best_match {
            ClassificationResult {
                feedback_type: rule.feedback_type,
                priority: rule.assigned_priority,
                assigned_to: rule.assigned_to.clone(),
                tags: rule.tags.clone(),
                confidence: 0.9, // 简化的置信度
            }
        } else {
            // 默认分类
            ClassificationResult {
                feedback_type: FeedbackType::Other,
                priority: Priority::Medium,
                assigned_to: None,
                tags: vec![],
                confidence: 0.5,
            }
        }
    }

    /// 计算优先级
    ///
    /// # Arguments
    /// * `impact` - 影响范围评分
    /// * `urgency` - 紧急程度评分
    /// * `severity` - 严重程度评分
    ///
    /// # Returns
    /// 计算出的优先级
    pub fn calculate_priority(
        &self,
        impact: u8,
        urgency: u8,
        severity: u8,
    ) -> Priority {
        let score = (impact as f32 * self.priority_config.impact_weight)
            + (urgency as f32 * self.priority_config.urgency_weight)
            + (severity as f32 * self.priority_config.severity_weight);

        if score >= self.priority_config.threshold_critical {
            Priority::Critical
        } else if score >= self.priority_config.threshold_high {
            Priority::High
        } else if score >= self.priority_config.threshold_medium {
            Priority::Medium
        } else if score >= self.priority_config.threshold_low {
            Priority::Low
        } else {
            Priority::Lowest
        }
    }

    /// 创建反馈
    ///
    /// # Arguments
    /// * `title` - 标题
    /// * `content` - 内容
    /// * `user_id` - 用户 ID
    ///
    /// # Returns
    /// 创建的反馈
    pub async fn create_feedback(
        &self,
        title: &str,
        content: &str,
        user_id: &str,
    ) -> Feedback {
        // 自动分类
        let classification = self.classify(title, content).await;

        // 计算优先级
        let impact = 5; // 默认值
        let urgency = 5;
        let severity = 5;
        let calculated_priority = self.calculate_priority(impact, urgency, severity);

        // 获取 SLA 配置
        let sla = self.get_sla(classification.feedback_type, calculated_priority).await;

        let feedback = Feedback {
            id: generate_id(),
            title: title.to_string(),
            content: content.to_string(),
            feedback_type: classification.feedback_type,
            priority: classification.priority,
            state: FeedbackState::New,
            user_id: user_id.to_string(),
            assigned_to: classification.assigned_to,
            tags: classification.tags,
            impact_score: impact,
            urgency_score: urgency,
            calculated_priority,
            response_deadline_hours: sla.first_response_hours,
            resolution_deadline_hours: sla.resolution_hours,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            resolved_at: None,
            metadata: HashMap::new(),
        };

        let mut feedbacks = self.feedbacks.write().await;
        feedbacks.insert(feedback.id.clone(), feedback.clone());

        feedback
    }

    /// 获取反馈
    ///
    /// # Arguments
    /// * `id` - 反馈 ID
    ///
    /// # Returns
    /// 反馈信息
    pub async fn get_feedback(&self, id: &str) -> Option<Feedback> {
        let feedbacks = self.feedbacks.read().await;
        feedbacks.get(id).cloned()
    }

    /// 更新反馈状态
    ///
    /// # Arguments
    /// * `id` - 反馈 ID
    /// * `state` - 新状态
    ///
    /// # Returns
    /// 是否成功
    pub async fn update_state(&self, id: &str, state: FeedbackState) -> bool {
        let mut feedbacks = self.feedbacks.write().await;

        if let Some(feedback) = feedbacks.get_mut(id) {
            feedback.state = state;
            feedback.updated_at = Utc::now();

            if state == FeedbackState::Resolved {
                feedback.resolved_at = Some(Utc::now());
            }

            return true;
        }
        false
    }

    /// 分配反馈
    ///
    /// # Arguments
    /// * `id` - 反馈 ID
    /// * `assignee` - 分配给谁
    ///
    /// # Returns
    /// 是否成功
    pub async fn assign(&self, id: &str, assignee: &str) -> bool {
        let mut feedbacks = self.feedbacks.write().await;

        if let Some(feedback) = feedbacks.get_mut(id) {
            feedback.assigned_to = Some(assignee.to_string());
            feedback.updated_at = Utc::now();
            return true;
        }
        false
    }

    /// 添加标签
    ///
    /// # Arguments
    /// * `id` - 反馈 ID
    /// * `tag` - 标签
    ///
    /// # Returns
    /// 是否成功
    pub async fn add_tag(&self, id: &str, tag: &str) -> bool {
        let mut feedbacks = self.feedbacks.write().await;

        if let Some(feedback) = feedbacks.get_mut(id) {
            if !feedback.tags.contains(&tag.to_string()) {
                feedback.tags.push(tag.to_string());
                feedback.updated_at = Utc::now();
            }
            return true;
        }
        false
    }

    /// 获取 SLA 配置
    ///
    /// # Arguments
    /// * `feedback_type` - 反馈类型
    /// * `priority` - 优先级
    ///
    /// # Returns
    /// SLA 配置
    pub async fn get_sla(
        &self,
        feedback_type: FeedbackType,
        priority: Priority,
    ) -> SlaConfig {
        let configs = self.sla_configs.read().await;
        configs
            .iter()
            .find(|c| c.feedback_type == feedback_type && c.priority == priority)
            .cloned()
            .unwrap_or_else(|| SlaConfig {
                feedback_type,
                priority,
                first_response_hours: 24,
                resolution_hours: 72,
                enabled: true,
            })
    }

    /// 设置 SLA 配置
    ///
    /// # Arguments
    /// * `config` - SLA 配置
    ///
    /// # Returns
    /// 是否成功
    pub async fn set_sla(&self, config: SlaConfig) -> bool {
        let mut configs = self.sla_configs.write().await;

        // 移除现有配置
        configs.retain(|c| {
            !(c.feedback_type == config.feedback_type && c.priority == config.priority)
        });

        configs.push(config);
        true
    }

    /// 按状态查询反馈
    ///
    /// # Arguments
    /// * `state` - 状态
    ///
    /// # Returns
    /// 反馈列表
    pub async fn get_by_state(&self, state: FeedbackState) -> Vec<Feedback> {
        let feedbacks = self.feedbacks.read().await;
        feedbacks
            .values()
            .filter(|f| f.state == state)
            .cloned()
            .collect()
    }

    /// 按优先级查询反馈
    ///
    /// # Arguments
    /// * `priority` - 优先级
    ///
    /// # Returns
    /// 反馈列表
    pub async fn get_by_priority(&self, priority: Priority) -> Vec<Feedback> {
        let feedbacks = self.feedbacks.read().await;
        feedbacks
            .values()
            .filter(|f| f.priority == priority || f.calculated_priority == priority)
            .cloned()
            .collect()
    }

    /// 获取待分配的反馈
    ///
    /// # Returns
    /// 待分配反馈列表
    pub async fn get_unassigned(&self) -> Vec<Feedback> {
        let feedbacks = self.feedbacks.read().await;
        feedbacks
            .values()
            .filter(|f| f.assigned_to.is_none())
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> FeedbackStats {
        let feedbacks = self.feedbacks.read().await;

        let total = feedbacks.len();
        let by_state: HashMap<String, usize> = feedbacks
            .values()
            .fold(HashMap::new(), |mut acc, f| {
                *acc.entry(format!("{:?}", f.state)).or_insert(0) += 1;
                acc
            });

        let by_priority: HashMap<String, usize> = feedbacks
            .values()
            .fold(HashMap::new(), |mut acc, f| {
                *acc.entry(format!("{:?}", f.priority)).or_insert(0) += 1;
                acc
            });

        let avg_response_time = 0.0; // 简化计算
        let avg_resolution_time = 0.0;

        FeedbackStats {
            total,
            by_state,
            by_priority,
            avg_response_time_hours: avg_response_time,
            avg_resolution_time_hours: avg_resolution_time,
        }
    }
}

impl Default for FeedbackManager {
    fn default() -> Self {
        Self::new(PriorityConfig::default())
    }
}

/// 分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub feedback_type: FeedbackType,
    pub priority: Priority,
    pub assigned_to: Option<String>,
    pub tags: Vec<String>,
    pub confidence: f32,
}

/// 反馈统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackStats {
    pub total: usize,
    pub by_state: HashMap<String, usize>,
    pub by_priority: HashMap<String, usize>,
    pub avg_response_time_hours: f32,
    pub avg_resolution_time_hours: f32,
}

/// 生成唯一 ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after UNIX epoch")
        .as_nanos();
    format!("fb_{:016x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_rule() {
        let manager = FeedbackManager::default_manager();

        let rule = ClassificationRule {
            rule_id: "rule_001".to_string(),
            name: "Bug 报告规则".to_string(),
            feedback_type: FeedbackType::Bug,
            keywords: vec!["错误".to_string(), "崩溃".to_string(), "bug".to_string()],
            regex_pattern: Some(r"error|crash|exception".to_string()),
            assigned_priority: Priority::High,
            assigned_to: Some("dev_team".to_string()),
            tags: vec!["技术".to_string()],
            priority: 10,
            enabled: true,
        };

        let result = manager.add_rule(rule).await;
        assert!(result);

        let rules = manager.get_rules().await;
        assert_eq!(rules.len(), 1);
    }

    #[tokio::test]
    async fn test_classify() {
        let manager = FeedbackManager::default_manager();

        // 添加规则
        let rule = ClassificationRule {
            rule_id: "rule_001".to_string(),
            name: "Bug 规则".to_string(),
            feedback_type: FeedbackType::Bug,
            keywords: vec!["崩溃".to_string()],
            regex_pattern: None,
            assigned_priority: Priority::High,
            assigned_to: Some("dev".to_string()),
            tags: vec!["技术".to_string()],
            priority: 10,
            enabled: true,
        };
        manager.add_rule(rule).await;

        // 测试分类
        let result = manager.classify("程序崩溃", "程序在运行时崩溃了").await;
        assert_eq!(result.feedback_type, FeedbackType::Bug);
        assert_eq!(result.priority, Priority::High);
    }

    #[tokio::test]
    async fn test_create_feedback() {
        let manager = FeedbackManager::default_manager();

        let feedback = manager.create_feedback(
            "功能建议",
            "希望添加一个功能",
            "user_001",
        ).await;

        assert_eq!(feedback.title, "功能建议");
        assert_eq!(feedback.user_id, "user_001");
        assert_eq!(feedback.state, FeedbackState::New);
    }

    #[tokio::test]
    async fn test_update_state() {
        let manager = FeedbackManager::default_manager();

        let feedback = manager.create_feedback("标题", "内容", "user_001").await;

        let updated = manager.update_state(&feedback.id, FeedbackState::Processing).await;
        assert!(updated);

        let fetched = manager.get_feedback(&feedback.id).await;
        assert_eq!(fetched.expect("fetched feedback should exist").state, FeedbackState::Processing);
    }

    #[tokio::test]
    async fn test_assign() {
        let manager = FeedbackManager::default_manager();

        let feedback = manager.create_feedback("标题", "内容", "user_001").await;

        let assigned = manager.assign(&feedback.id, "admin_001").await;
        assert!(assigned);

        let fetched = manager.get_feedback(&feedback.id).await;
        assert_eq!(fetched.expect("fetched feedback should exist").assigned_to, Some("admin_001".to_string()));
    }

    #[tokio::test]
    async fn test_get_by_state() {
        let manager = FeedbackManager::default_manager();

        let f1 = manager.create_feedback("标题1", "内容1", "user_001").await;
        let f2 = manager.create_feedback("标题2", "内容2", "user_001").await;

        manager.update_state(&f1.id, FeedbackState::New).await;
        manager.update_state(&f2.id, FeedbackState::Processing).await;

        let new_feedbacks = manager.get_by_state(FeedbackState::New).await;
        assert_eq!(new_feedbacks.len(), 1);
    }

    #[tokio::test]
    async fn test_priority_calculation() {
        let manager = FeedbackManager::default_manager();

        // 测试不同评分组合
        let p1 = manager.calculate_priority(9, 9, 9); // Critical
        assert_eq!(p1, Priority::Critical);

        let p2 = manager.calculate_priority(5, 5, 5); // Medium
        assert_eq!(p2, Priority::Medium);

        let p3 = manager.calculate_priority(1, 1, 1); // Lowest
        assert_eq!(p3, Priority::Lowest);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = FeedbackManager::default_manager();

        manager.create_feedback("标题1", "内容1", "user_001").await;
        manager.create_feedback("标题2", "内容2", "user_001").await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 2);
    }
}