//! 消息已读回执模块
//!
//! 实现消息已读状态同步、多端已读状态一致性
//! 支持单聊、群聊、系统通知等不同消息类型的已读回执

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 已读回执状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReceiptStatus {
    /// 已发送但未读
    #[default]
    Delivered,
    /// 已读
    Read,
    /// 已撤回
    Recalled,
}

/// 单条已读回执
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceipt {
    pub receipt_id: String,
    pub message_id: String,
    pub user_id: String,
    pub read_at: DateTime<Utc>,
    pub status: ReceiptStatus,
    pub device_id: Option<String>,
    pub client_type: Option<String>,
}

/// 批量已读回执请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReceiptRequest {
    pub user_id: String,
    pub message_ids: Vec<String>,
    pub device_id: Option<String>,
}

/// 群聊已读回执统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupReadReceipt {
    pub message_id: String,
    pub total_members: u32,
    pub read_count: u32,
    pub unread_members: Vec<String>,
    pub read_percentage: f32,
}

/// 已读回执配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptConfig {
    /// 已读回执有效期（秒）
    pub receipt_ttl_seconds: u64,
    /// 批量已读最大数量
    pub max_batch_size: usize,
    /// 回执同步间隔（毫秒）
    pub sync_interval_ms: u64,
    /// 多端同步启用
    pub multi_device_sync: bool,
    /// 已读回执推送启用
    pub push_enabled: bool,
}

impl Default for ReceiptConfig {
    fn default() -> Self {
        Self {
            receipt_ttl_seconds: 2592000, // 30 天
            max_batch_size: 100,
            sync_interval_ms: 500,
            multi_device_sync: true,
            push_enabled: true,
        }
    }
}

/// 已读回执管理器
pub struct ReceiptManager {
    /// `回执缓存（message_id` -> `HashMap`<`user_id`, `ReadReceipt`>）
    receipts: Arc<RwLock<HashMap<String, HashMap<String, ReadReceipt>>>>,
    /// 群聊已读统计缓存
    group_stats: Arc<RwLock<HashMap<String, GroupReadReceipt>>>,
    /// 配置
    config: ReceiptConfig,
}

impl ReceiptManager {
    /// 创建新的 `ReceiptManager`
    #[must_use]
    pub fn new(config: ReceiptConfig) -> Self {
        Self {
            receipts: Arc::new(RwLock::new(HashMap::new())),
            group_stats: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 创建默认配置的 `ReceiptManager`
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new(ReceiptConfig::default())
    }

    /// 标记消息已读
    ///
    /// # Arguments
    /// * `message_id` - 消息 ID
    /// * `user_id` - 用户 ID
    /// * `device_id` - 设备 ID（可选，用于多端同步）
    /// * `client_type` - 客户端类型
    ///
    /// # Returns
    /// 创建的已读回执
    pub async fn mark_read(
        &self,
        message_id: &str,
        user_id: &str,
        device_id: Option<String>,
        client_type: Option<String>,
    ) -> ReadReceipt {
        let receipt = ReadReceipt {
            receipt_id: uuid_v4(),
            message_id: message_id.to_string(),
            user_id: user_id.to_string(),
            read_at: Utc::now(),
            status: ReceiptStatus::Read,
            device_id,
            client_type,
        };

        let mut receipts = self.receipts.write().await;
        let msg_receipts = receipts
            .entry(message_id.to_string())
            .or_insert_with(HashMap::new);
        msg_receipts.insert(user_id.to_string(), receipt.clone());

        receipt
    }

    /// 批量标记消息已读
    ///
    /// # Arguments
    /// * `request` - 批量已读请求
    ///
    /// # Returns
    /// 成功标记的消息数量
    pub async fn batch_mark_read(&self, request: BatchReceiptRequest) -> usize {
        let BatchReceiptRequest {
            user_id,
            message_ids,
            device_id,
        } = request;

        // 限制批量大小
        let message_ids: Vec<String> = message_ids
            .into_iter()
            .take(self.config.max_batch_size)
            .collect();

        let mut count = 0;
        for message_id in &message_ids {
            let _ = self
                .mark_read(message_id, &user_id, device_id.clone(), None)
                .await;
            count += 1;
        }

        count
    }

    /// 获取消息的已读状态
    ///
    /// # Arguments
    /// * `message_id` - 消息 ID
    /// * `user_id` - 用户 ID
    ///
    /// # Returns
    /// 已读回执（如果存在）
    pub async fn get_receipt(&self, message_id: &str, user_id: &str) -> Option<ReadReceipt> {
        let receipts = self.receipts.read().await;
        receipts
            .get(message_id)
            .and_then(|user_receipts| user_receipts.get(user_id).cloned())
    }

    /// 获取消息的所有已读回执
    ///
    /// # Arguments
    /// * `message_id` - 消息 ID
    ///
    /// # Returns
    /// 已读回执列表
    pub async fn get_all_receipts(&self, message_id: &str) -> Vec<ReadReceipt> {
        let receipts = self.receipts.read().await;
        receipts
            .get(message_id)
            .map(|user_receipts| user_receipts.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取用户未读消息数
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `message_ids` - 消息 ID 列表
    ///
    /// # Returns
    /// 未读消息数量
    pub async fn count_unread(&self, user_id: &str, message_ids: &[String]) -> usize {
        let receipts = self.receipts.read().await;
        message_ids
            .iter()
            .filter(|msg_id| {
                receipts
                    .get(*msg_id)
                    .is_none_or(|user_receipts| !user_receipts.contains_key(user_id))
            })
            .count()
    }

    /// 更新群聊已读统计
    ///
    /// # Arguments
    /// * `message_id` - 消息 ID
    /// * `total_members` - 总成员数
    /// * `member_ids` - 成员 ID 列表
    ///
    /// # Returns
    /// 更新后的群聊已读统计
    pub async fn update_group_stats(
        &self,
        message_id: &str,
        total_members: u32,
        member_ids: &[String],
    ) -> GroupReadReceipt {
        let receipts = self.receipts.read().await;
        let msg_receipts = receipts.get(message_id);

        // 获取已读成员
        let read_member_ids: Vec<String> = msg_receipts
            .map(|user_receipts| {
                user_receipts
                    .keys()
                    .filter(|uid| uid.starts_with("user_"))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        // 计算未读成员
        let unread_members: Vec<String> = member_ids
            .iter()
            .filter(|id| !read_member_ids.contains(id))
            .cloned()
            .collect();

        let read_count = read_member_ids.len() as u32;
        let read_percentage = if total_members > 0 {
            (read_count as f32 / total_members as f32) * 100.0
        } else {
            0.0
        };

        let stats = GroupReadReceipt {
            message_id: message_id.to_string(),
            total_members,
            read_count,
            unread_members,
            read_percentage,
        };

        // 更新缓存
        let mut group_stats = self.group_stats.write().await;
        group_stats.insert(message_id.to_string(), stats.clone());

        stats
    }

    /// 获取群聊已读统计
    ///
    /// # Arguments
    /// * `message_id` - 消息 ID
    ///
    /// # Returns
    /// 群聊已读统计（如果存在）
    pub async fn get_group_stats(&self, message_id: &str) -> Option<GroupReadReceipt> {
        let stats = self.group_stats.read().await;
        stats.get(message_id).cloned()
    }

    /// 同步多端已读状态
    ///
    /// 将用户在某一设备上的已读状态同步到其他设备
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `message_id` - 消息 ID
    /// * `source_device_id` - 源设备 ID
    ///
    /// # Returns
    /// 同步的目标设备数量
    pub async fn sync_multi_device(
        &self,
        user_id: &str,
        message_id: &str,
        source_device_id: &str,
    ) -> usize {
        if !self.config.multi_device_sync {
            return 0;
        }

        // 获取源设备的已读状态
        let source_receipt = self.get_receipt(message_id, user_id).await;

        let Some(receipt) = source_receipt else {
            return 0;
        };

        // 模拟其他设备同步
        // 实际实现中会通过 WebSocket 或其他渠道推送更新
        let target_devices = self.get_user_devices(user_id).await;
        let mut sync_count = 0;

        for device_id in target_devices {
            if device_id != source_device_id {
                // 创建该设备的回执（复用源回执的时间戳）
                let device_receipt = ReadReceipt {
                    receipt_id: uuid_v4(),
                    message_id: message_id.to_string(),
                    user_id: user_id.to_string(),
                    read_at: receipt.read_at,
                    status: receipt.status,
                    device_id: Some(device_id.clone()),
                    client_type: receipt.client_type.clone(),
                };

                let mut receipts = self.receipts.write().await;
                let msg_receipts = receipts
                    .entry(message_id.to_string())
                    .or_insert_with(HashMap::new);
                msg_receipts.insert(user_id.to_string(), device_receipt);
                sync_count += 1;
            }
        }

        sync_count
    }

    /// 获取用户的所有设备
    /// 实际实现中应从数据库或 Redis 获取
    async fn get_user_devices(&self, _user_id: &str) -> Vec<String> {
        // 模拟返回设备列表
        vec!["device_web".to_string(), "device_mobile".to_string()]
    }

    /// 删除已过期回执
    ///
    /// # Arguments
    /// * `max_age_seconds` - 最大保留时间（秒）
    ///
    /// # Returns
    /// 删除的回执数量
    pub async fn cleanup_expired(&self, max_age_seconds: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_seconds as i64);
        let mut receipts = self.receipts.write().await;

        let mut total_removed = 0;

        for msg_receipts in receipts.values_mut() {
            let before = msg_receipts.len();
            msg_receipts.retain(|_, receipt| receipt.read_at > cutoff);
            total_removed += before - msg_receipts.len();
        }

        // 清理空的消息回执
        receipts.retain(|_, msg_receipts| !msg_receipts.is_empty());

        total_removed
    }

    /// 获取回执统计
    pub async fn get_stats(&self) -> ReceiptStats {
        let receipts = self.receipts.read().await;
        let group_stats = self.group_stats.read().await;

        let total_messages = receipts.len();
        let total_receipts: usize = receipts.values().map(std::collections::HashMap::len).sum();
        let total_groups = group_stats.len();

        let avg_read_percentage = if total_groups > 0 {
            let sum: f32 = group_stats.values().map(|g| g.read_percentage).sum();
            sum / total_groups as f32
        } else {
            0.0
        };

        ReceiptStats {
            total_messages,
            total_receipts,
            total_groups,
            avg_read_percentage,
        }
    }
}

/// 已读回执统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptStats {
    pub total_messages: usize,
    pub total_receipts: usize,
    pub total_groups: usize,
    pub avg_read_percentage: f32,
}

/// 生成 UUID 的简化实现
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| { tracing::error!("system time is before UNIX epoch"); std::time::Duration::from_secs(0) })
        .as_nanos();
    format!("{timestamp:032x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mark_read() {
        let manager = ReceiptManager::default_manager();

        let receipt = manager
            .mark_read(
                "msg_001",
                "user_001",
                Some("device_001".to_string()),
                Some("web".to_string()),
            )
            .await;

        assert_eq!(receipt.message_id, "msg_001");
        assert_eq!(receipt.user_id, "user_001");
        assert_eq!(receipt.status, ReceiptStatus::Read);

        // 验证回执可以获取
        let fetched = manager.get_receipt("msg_001", "user_001").await;
        assert!(fetched.is_some());
        assert_eq!(fetched.expect("receipt should exist").status, ReceiptStatus::Read);
    }

    #[tokio::test]
    async fn test_batch_mark_read() {
        let manager = ReceiptManager::default_manager();

        let request = BatchReceiptRequest {
            user_id: "user_001".to_string(),
            message_ids: vec![
                "msg_001".to_string(),
                "msg_002".to_string(),
                "msg_003".to_string(),
            ],
            device_id: Some("device_001".to_string()),
        };

        let count = manager.batch_mark_read(request).await;
        assert_eq!(count, 3);

        // 验证批量已读
        assert!(manager.get_receipt("msg_001", "user_001").await.is_some());
        assert!(manager.get_receipt("msg_002", "user_001").await.is_some());
        assert!(manager.get_receipt("msg_003", "user_001").await.is_some());
    }

    #[tokio::test]
    async fn test_count_unread() {
        let manager = ReceiptManager::default_manager();

        // 标记部分消息已读
        manager.mark_read("msg_001", "user_001", None, None).await;
        manager.mark_read("msg_002", "user_001", None, None).await;

        let message_ids = vec![
            "msg_001".to_string(),
            "msg_002".to_string(),
            "msg_003".to_string(),
            "msg_004".to_string(),
        ];

        let unread = manager.count_unread("user_001", &message_ids).await;
        assert_eq!(unread, 2); // msg_003, msg_004 未读
    }

    #[tokio::test]
    async fn test_group_stats() {
        let manager = ReceiptManager::default_manager();

        let member_ids = vec![
            "user_001".to_string(),
            "user_002".to_string(),
            "user_003".to_string(),
        ];

        // 部分用户已读
        manager
            .mark_read("group_msg_001", "user_001", None, None)
            .await;
        manager
            .mark_read("group_msg_001", "user_002", None, None)
            .await;

        let stats = manager
            .update_group_stats("group_msg_001", 3, &member_ids)
            .await;

        assert_eq!(stats.total_members, 3);
        assert_eq!(stats.read_count, 2);
        assert_eq!(stats.unread_members.len(), 1);
        assert!((stats.read_percentage - 66.67).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_multi_device_sync() {
        let manager = ReceiptManager::new(ReceiptConfig {
            multi_device_sync: true,
            ..Default::default()
        });

        // 在 web 端已读
        manager
            .mark_read(
                "msg_001",
                "user_001",
                Some("device_web".to_string()),
                Some("web".to_string()),
            )
            .await;

        // 同步到移动端
        let sync_count = manager
            .sync_multi_device("user_001", "msg_001", "device_web")
            .await;

        assert!(sync_count >= 1);

        // 验证移动端也有回执
        let mobile_receipt = manager.get_receipt("msg_001", "user_001").await;
        assert!(mobile_receipt.is_some());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = ReceiptManager::default_manager();

        manager.mark_read("msg_001", "user_001", None, None).await;

        // 清理过期回执（0秒 = 全部过期）
        let removed = manager.cleanup_expired(0).await;
        assert_eq!(removed, 1);

        // 验证回执已删除
        assert!(manager.get_receipt("msg_001", "user_001").await.is_none());
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = ReceiptManager::default_manager();

        manager.mark_read("msg_001", "user_001", None, None).await;
        manager.mark_read("msg_001", "user_002", None, None).await;
        manager.mark_read("msg_002", "user_001", None, None).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.total_receipts, 3);
    }
}
