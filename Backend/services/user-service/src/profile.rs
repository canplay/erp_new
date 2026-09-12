//! 用户画像功能模块
//!
//! 实现用户标签、偏好设置、行为分析
//! 
//! @date 2026-05-16

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Instant;

/// 用户标签
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UserTag {
    /// 标签 ID
    pub id: String,
    /// 标签名称
    pub name: String,
    /// 标签分类
    pub category: String,
    /// 标签权重（0-100）
    pub weight: u8,
    /// 创建时间
    pub created_at: Instant,
}

/// 用户偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    /// 偏好类型
    pub preference_type: PreferenceType,
    /// 偏好值
    pub value: String,
    /// 偏好强度（0-100）
    pub intensity: u8,
    /// 最后更新时间
    pub updated_at: Instant,
}

/// 偏好类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PreferenceType {
    /// 主题偏好（深色/浅色）
    Theme,
    /// 语言偏好
    Language,
    /// 通知偏好
    Notification,
    /// 隐私偏好
    Privacy,
    /// 时间偏好
    Time,
    /// 内容偏好
    Content,
    /// 其他
    Other,
}

/// 用户行为记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehavior {
    /// 行为类型
    pub behavior_type: BehaviorType,
    /// 行为对象 ID
    pub object_id: String,
    /// 行为对象类型
    pub object_type: String,
    /// 行为时间
    pub timestamp: Instant,
    /// 持续时间（毫秒）
    pub duration_ms: Option<u64>,
    /// 附加数据
    pub metadata: HashMap<String, String>,
}

/// 行为类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BehaviorType {
    /// 浏览
    View,
    /// 点击
    Click,
    /// 搜索
    Search,
    /// 收藏
    Favorite,
    /// 分享
    Share,
    /// 评论
    Comment,
    /// 购买
    Purchase,
    /// 下载
    Download,
    /// 其他
    Other,
}

/// 用户画像
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// 用户 ID
    pub user_id: i64,
    /// 用户标签
    pub tags: HashSet<UserTag>,
    /// 用户偏好
    pub preferences: Vec<UserPreference>,
    /// 近期行为
    pub recent_behaviors: Vec<UserBehavior>,
    /// 画像分数（综合评估）
    pub profile_score: u32,
    /// 最后更新
    pub updated_at: Instant,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 用户画像管理器
pub struct ProfileManager {
    /// 用户画像存储
    profiles: Arc<RwLock<HashMap<i64, UserProfile>>>,
    /// 标签定义
    tag_definitions: Arc<RwLock<HashMap<String, UserTag>>>,
    /// 配置
    config: ProfileConfig,
}

/// 画像配置
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    /// 最大标签数
    pub max_tags: usize,
    /// 最大偏好数
    pub max_preferences: usize,
    /// 最大行为记录数
    pub max_behaviors: usize,
    /// 行为记录保留天数
    pub behavior_retention_days: u32,
    /// 画像更新间隔（秒）
    pub update_interval_secs: u64,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            max_tags: 50,
            max_preferences: 20,
            max_behaviors: 1000,
            behavior_retention_days: 90,
            update_interval_secs: 300,
        }
    }
}

impl ProfileManager {
    /// 创建新的画像管理器
    pub fn new(config: ProfileConfig) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            tag_definitions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 注册标签定义
    pub fn register_tag(&self, tag: UserTag) {
        let mut definitions = self.tag_definitions.write();
        definitions.insert(tag.id.clone(), tag);
        tracing::info!("注册用户标签: id={}", tag.id);
    }

    /// 批量注册标签
    pub fn register_tags(&self, tags: Vec<UserTag>) {
        let mut definitions = self.tag_definitions.write();
        for tag in tags {
            definitions.insert(tag.id.clone(), tag);
        }
        tracing::info!("批量注册用户标签: count={}", tags.len());
    }

    /// 获取用户画像
    pub fn get_profile(&self, user_id: i64) -> Option<UserProfile> {
        let profiles = self.profiles.read();
        profiles.get(&user_id).cloned()
    }

    /// 获取或创建用户画像
    pub fn get_or_create_profile(&self, user_id: i64) -> UserProfile {
        let mut profiles = self.profiles.write();
        profiles
            .entry(user_id)
            .or_insert_with(|| UserProfile {
                user_id,
                tags: HashSet::new(),
                preferences: Vec::new(),
                recent_behaviors: Vec::new(),
                profile_score: 0,
                updated_at: Instant::now(),
                metadata: HashMap::new(),
            })
            .clone()
    }

    /// 添加用户标签
    pub fn add_tag(&self, user_id: i64, tag_id: &str) -> Result<(), ProfileError> {
        let mut profiles = self.profiles.write();
        
        let profile = profiles
            .entry(user_id)
            .or_insert_with(|| UserProfile {
                user_id,
                tags: HashSet::new(),
                preferences: Vec::new(),
                recent_behaviors: Vec::new(),
                profile_score: 0,
                updated_at: Instant::now(),
                metadata: HashMap::new(),
            });

        // 检查标签数限制
        if profile.tags.len() >= self.config.max_tags {
            return Err(ProfileError::MaxTagsReached);
        }

        // 获取标签定义
        let definitions = self.tag_definitions.read();
        if let Some(tag) = definitions.get(tag_id) {
            profile.tags.insert(tag.clone());
            profile.updated_at = Instant::now();
            tracing::info!("添加用户标签: user_id={}, tag_id={}", user_id, tag_id);
            Ok(())
        } else {
            Err(ProfileError::TagNotFound)
        }
    }

    /// 移除用户标签
    pub fn remove_tag(&self, user_id: i64, tag_id: &str) -> Result<(), ProfileError> {
        let mut profiles = self.profiles.write();
        
        if let Some(profile) = profiles.get_mut(&user_id) {
            profile.tags.retain(|t| t.id != tag_id);
            profile.updated_at = Instant::now();
            tracing::info!("移除用户标签: user_id={}, tag_id={}", user_id, tag_id);
            Ok(())
        } else {
            Err(ProfileError::ProfileNotFound)
        }
    }

    /// 更新用户偏好
    pub fn update_preference(
        &self,
        user_id: i64,
        preference_type: PreferenceType,
        value: &str,
        intensity: u8,
    ) -> Result<(), ProfileError> {
        let mut profiles = self.profiles.write();
        
        let profile = profiles
            .entry(user_id)
            .or_insert_with(|| UserProfile {
                user_id,
                tags: HashSet::new(),
                preferences: Vec::new(),
                recent_behaviors: Vec::new(),
                profile_score: 0,
                updated_at: Instant::now(),
                metadata: HashMap::new(),
            });

        // 检查偏好数限制
        if profile.preferences.len() >= self.config.max_preferences {
            // 移除强度最低的偏好
            profile.preferences.sort_by_key(|p| p.intensity);
            profile.preferences.remove(0);
        }

        // 更新或添加偏好
        let now = Instant::now();
        if let Some(existing) = profile.preferences.iter_mut().find(|p| p.preference_type == preference_type) {
            existing.value = value.to_string();
            existing.intensity = intensity;
            existing.updated_at = now;
        } else {
            profile.preferences.push(UserPreference {
                preference_type,
                value: value.to_string(),
                intensity,
                updated_at: now,
            });
        }

        profile.updated_at = now;
        tracing::info!(
            "更新用户偏好: user_id={}, type={:?}, value={}",
            user_id, preference_type, value
        );
        Ok(())
    }

    /// 记录用户行为
    pub fn record_behavior(
        &self,
        user_id: i64,
        behavior_type: BehaviorType,
        object_id: &str,
        object_type: &str,
        duration_ms: Option<u64>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), ProfileError> {
        let mut profiles = self.profiles.write();
        
        let profile = profiles
            .entry(user_id)
            .or_insert_with(|| UserProfile {
                user_id,
                tags: HashSet::new(),
                preferences: Vec::new(),
                recent_behaviors: Vec::new(),
                profile_score: 0,
                updated_at: Instant::now(),
                metadata: HashMap::new(),
            });

        let behavior = UserBehavior {
            behavior_type,
            object_id: object_id.to_string(),
            object_type: object_type.to_string(),
            timestamp: Instant::now(),
            duration_ms,
            metadata: metadata.unwrap_or_default(),
        };

        profile.recent_behaviors.push(behavior);

        // 限制行为记录数
        while profile.recent_behaviors.len() > self.config.max_behaviors {
            profile.recent_behaviors.remove(0);
        }

        profile.updated_at = Instant::now();
        tracing::debug!(
            "记录用户行为: user_id={}, type={:?}, object={}",
            user_id, behavior_type, object_id
        );
        Ok(())
    }

    /// 计算用户画像分数
    pub fn calculate_profile_score(&self, user_id: i64) -> Result<u32, ProfileError> {
        let mut profiles = self.profiles.write();
        
        let profile = profiles.get_mut(&user_id).ok_or(ProfileError::ProfileNotFound)?;

        let mut score: u32 = 0;

        // 标签分数（每个标签10分）
        score += (profile.tags.len() * 10) as u32;

        // 偏好分数（每个偏好5分）
        score += (profile.preferences.len() * 5) as u32;

        // 行为分数（基于行为类型）
        for behavior in &profile.recent_behaviors {
            let behavior_score = match behavior.behavior_type {
                BehaviorType::View => 1,
                BehaviorType::Click => 2,
                BehaviorType::Search => 3,
                BehaviorType::Favorite => 5,
                BehaviorType::Share => 8,
                BehaviorType::Comment => 6,
                BehaviorType::Purchase => 15,
                BehaviorType::Download => 4,
                BehaviorType::Other => 1,
            };
            score += behavior_score;
        }

        // 活跃度分数（最近7天内有行为）
        let recent_count = profile.recent_behaviors.iter()
            .filter(|b| b.timestamp.elapsed().as_secs() < 7 * 24 * 3600)
            .count();
        if recent_count > 0 {
            score += (recent_count.min(50) * 2) as u32;
        }

        profile.profile_score = score;
        profile.updated_at = Instant::now();

        tracing::info!("计算用户画像分数: user_id={}, score={}", user_id, score);
        Ok(score)
    }

    /// 获取用户推荐（基于标签和偏好）
    pub fn get_recommendations(&self, user_id: i64, limit: usize) -> Result<Vec<String>, ProfileError> {
        let profiles = self.profiles.read();
        
        let profile = profiles.get(&user_id).ok_or(ProfileError::ProfileNotFound)?;

        // 获取用户标签和偏好
        let tags: Vec<String> = profile.tags.iter().map(|t| t.id.clone()).collect();
        let preferences: Vec<String> = profile.preferences.iter().map(|p| p.value.clone()).collect();

        // 简化推荐：返回标签 ID
        let mut recommendations: Vec<String> = tags;
        recommendations.truncate(limit);

        Ok(recommendations)
    }

    /// 获取相似用户
    pub fn find_similar_users(&self, user_id: i64, limit: usize) -> Result<Vec<(i64, u32)>, ProfileError> {
        let profiles = self.profiles.read();
        
        let source_profile = profiles.get(&user_id).ok_or(ProfileError::ProfileNotFound)?;

        let mut similarities: Vec<(i64, u32)> = Vec::new();

        for (uid, profile) in profiles.iter() {
            if *uid == user_id {
                continue;
            }

            // 计算相似度（标签交集）
            let common_tags = source_profile.tags.intersection(&profile.tags).count();
            let total_tags = source_profile.tags.union(&profile.tags).count();
            
            if total_tags > 0 {
                let similarity = (common_tags * 100) as u32 / total_tags as u32;
                similarities.push((*uid, similarity));
            }
        }

        // 按相似度排序
        similarities.sort_by(|a, b| b.1.cmp(&a.1));
        similarities.truncate(limit);

        Ok(similarities)
    }

    /// 删除用户画像
    pub fn delete_profile(&self, user_id: i64) -> bool {
        let mut profiles = self.profiles.write();
        let removed = profiles.remove(&user_id).is_some();
        if removed {
            tracing::info!("删除用户画像: user_id={}", user_id);
        }
        removed
    }

    /// 清理过期行为记录
    pub fn cleanup_expired_behaviors(&self) -> usize {
        let mut profiles = self.profiles.write();
        let cutoff = Instant::now() - std::time::Duration::from_secs(
            (self.config.behavior_retention_days as u64) * 24 * 3600
        );

        let mut total_cleaned = 0;
        for profile in profiles.values_mut() {
            let before = profile.recent_behaviors.len();
            profile.recent_behaviors.retain(|b| b.timestamp > cutoff);
            total_cleaned += before - profile.recent_behaviors.len();
        }

        tracing::debug!("清理过期行为记录: count={}", total_cleaned);
        total_cleaned
    }

    /// 获取用户统计
    pub fn get_stats(&self) -> ProfileStats {
        let profiles = self.profiles.read();
        
        let total_users = profiles.len();
        let total_tags: usize = profiles.values().map(|p| p.tags.len()).sum();
        let total_preferences: usize = profiles.values().map(|p| p.preferences.len()).sum();
        let total_behaviors: usize = profiles.values().map(|p| p.recent_behaviors.len()).sum();
        
        let avg_score = if total_users > 0 {
            profiles.values().map(|p| p.profile_score as usize).sum::<usize>() / total_users
        } else {
            0
        };

        ProfileStats {
            total_users,
            total_tags,
            total_preferences,
            total_behaviors,
            avg_profile_score: avg_score as u32,
        }
    }
}

/// 画像错误
#[derive(Debug, Clone, Copy)]
pub enum ProfileError {
    /// 画像不存在
    ProfileNotFound,
    /// 标签不存在
    TagNotFound,
    /// 达到最大标签数
    MaxTagsReached,
    /// 达到最大偏好数
    MaxPreferencesReached,
    /// 达到最大行为数
    MaxBehaviorsReached,
}

/// 画像统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub total_users: usize,
    pub total_tags: usize,
    pub total_preferences: usize,
    pub total_behaviors: usize,
    pub avg_profile_score: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_manager() {
        let manager = ProfileManager::new(ProfileConfig::default());

        // 注册标签
        manager.register_tag(UserTag {
            id: "tag1".to_string(),
            name: "技术".to_string(),
            category: "interest".to_string(),
            weight: 80,
            created_at: Instant::now(),
        });

        manager.register_tag(UserTag {
            id: "tag2".to_string(),
            name: "音乐".to_string(),
            category: "interest".to_string(),
            weight: 60,
            created_at: Instant::now(),
        });

        // 添加标签
        manager.add_tag(1, "tag1").expect("test assertion");
        manager.add_tag(1, "tag2").expect("test assertion");

        // 更新偏好
        manager.update_preference(1, PreferenceType::Theme, "dark", 90).expect("test assertion");
        manager.update_preference(1, PreferenceType::Language, "zh-CN", 80).expect("test assertion");

        // 记录行为
        manager.record_behavior(1, BehaviorType::View, "article1", "article", None, None).expect("test assertion");
        manager.record_behavior(1, BehaviorType::Favorite, "article2", "article", None, None).expect("test assertion");

        // 计算分数
        let score = manager.calculate_profile_score(1).expect("test assertion");
        assert!(score > 0);

        // 获取统计
        let stats = manager.get_stats();
        assert_eq!(stats.total_users, 1);
        assert_eq!(stats.total_tags, 2);
    }

    #[test]
    fn test_find_similar_users() {
        let manager = ProfileManager::new(ProfileConfig::default());

        // 注册标签
        manager.register_tag(UserTag {
            id: "tech".to_string(),
            name: "技术".to_string(),
            category: "interest".to_string(),
            weight: 80,
            created_at: Instant::now(),
        });

        // 用户1
        manager.add_tag(1, "tech").expect("test assertion");
        
        // 用户2
        manager.add_tag(2, "tech").expect("test assertion");
        
        // 用户3
        manager.add_tag(3, "tech").expect("test assertion");

        // 查找相似用户
        let similar = manager.find_similar_users(1, 5).expect("test assertion");
        assert_eq!(similar.len(), 2); // 用户2和用户3
    }
}