//! 文件版本管理模块
//!
//! 实现文件版本历史、版本对比、版本回退功能
//! 支持多版本存储、增量存储、版本标签管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 文件版本状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VersionState {
    /// 草稿版本
    Draft,
    /// 正式版本
    #[default]
    Active,
    /// 已归档
    Archived,
    /// 已删除
    Deleted,
}

/// 文件版本元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub version_id: String,
    pub file_id: String,
    pub version_number: u32,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub content_hash: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub state: VersionState,
    pub description: Option<String>,
    pub tags: Vec<String>,
    /// 增量大小（相对于前一版本）
    pub delta_size: Option<u64>,
    /// 是否为初始版本
    pub is_initial: bool,
}

/// 版本差异信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: String,
    pub to_version: String,
    pub diff_type: DiffType,
    pub added_bytes: u64,
    pub removed_bytes: u64,
    pub net_change: i64,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// 完全新增
    Added,
    /// 完全删除
    Removed,
    /// 部分修改
    Modified,
    /// 无变化
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub change_type: String,
    pub position: Option<u64>,
    pub content: Option<String>,
}

/// 版本历史查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistoryQuery {
    pub file_id: String,
    pub from_version: Option<u32>,
    pub to_version: Option<u32>,
    pub state: Option<VersionState>,
    pub created_by: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 版本比较配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareConfig {
    /// 允许比较的最大版本跨度
    pub max_version_span: u32,
    /// 是否启用增量比较
    pub enable_delta_compare: bool,
    /// 最大缓存版本数
    pub max_cached_versions: usize,
    /// 版本保留策略（天数）
    pub retention_days: u64,
}

impl Default for CompareConfig {
    fn default() -> Self {
        Self {
            max_version_span: 100,
            enable_delta_compare: true,
            max_cached_versions: 50,
            retention_days: 365,
        }
    }
}

/// 版本管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    /// 启用版本管理
    pub enabled: bool,
    /// 自动创建版本间隔（秒）
    pub auto_save_interval: u64,
    /// 最大版本数（0 = 无限制）
    pub max_versions: usize,
    /// 启用增量存储
    pub enable_delta_storage: bool,
    /// 版本比较配置
    pub compare: CompareConfig,
    /// 清理过期版本
    pub auto_cleanup: bool,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_save_interval: 300,
            max_versions: 0,
            enable_delta_storage: true,
            compare: CompareConfig::default(),
            auto_cleanup: true,
        }
    }
}

/// 版本回退请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub file_id: String,
    pub target_version: u32,
    pub reason: Option<String>,
    pub create_backup: bool,
}

/// 版本回退结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub new_version_id: String,
    pub new_version_number: u32,
    pub backup_version_id: Option<String>,
    pub message: String,
}

/// 文件版本管理器
pub struct VersionManager {
    /// `版本存储（file_id` -> Vec<FileVersion>）
    versions: Arc<RwLock<HashMap<String, Vec<FileVersion>>>>,
    /// 当前版本快照
    snapshots: Arc<RwLock<HashMap<String, FileVersion>>>,
    /// 配置
    config: VersionConfig,
}

/// 创建新版本参数
#[derive(Debug, Clone)]
pub struct CreateVersionParams {
    pub file_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub content_hash: String,
    pub storage_path: String,
    pub created_by: String,
    pub description: Option<String>,
}

impl VersionManager {
    /// 创建新的 `VersionManager`
    #[must_use]
    pub fn new(config: VersionConfig) -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 创建默认配置的 `VersionManager`
    #[must_use]
    pub fn default_manager() -> Self {
        Self::new(VersionConfig::default())
    }

    /// 创建新版本
    pub async fn create_version(
        &self,
        params: CreateVersionParams,
    ) -> FileVersion {
        let mut versions = self.versions.write().await;

        // 获取当前版本数
        let version_number = versions
            .get(&params.file_id)
            .map_or(1, |v| v.len() as u32 + 1);

        // 计算增量大小
        let delta_size = if version_number > 1 {
            versions
                .get(&params.file_id)
                .and_then(|v| v.last())
                .map(|last| params.file_size.abs_diff(last.file_size))
        } else {
            None
        };

        let version = FileVersion {
            version_id: generate_version_id(&params.file_id, version_number),
            file_id: params.file_id,
            version_number,
            file_name: params.file_name,
            file_path: params.file_path,
            file_size: params.file_size,
            content_hash: params.content_hash,
            storage_path: params.storage_path,
            created_at: Utc::now(),
            created_by: params.created_by,
            state: VersionState::Active,
            description: params.description,
            tags: Vec::new(),
            delta_size,
            is_initial: version_number == 1,
        };

        // 添加到版本历史
        let file_versions = versions.entry(version.file_id.clone()).or_insert_with(Vec::new);
        file_versions.push(version.clone());

        // 更新快照
        drop(versions);
        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(version.file_id.clone(), version.clone());

        version
    }

    /// 获取文件的所有版本
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    ///
    /// # Returns
    /// 版本列表（按版本号降序）
    pub async fn get_versions(&self, file_id: &str) -> Vec<FileVersion> {
        let versions = self.versions.read().await;
        versions
            .get(file_id)
            .map(|v| {
                let mut sorted = v.clone();
                sorted.sort_by_key(|b| std::cmp::Reverse(b.version_number));
                sorted
            })
            .unwrap_or_default()
    }

    /// 获取特定版本
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    /// * `version_number` - 版本号
    ///
    /// # Returns
    /// 版本信息（如果存在）
    pub async fn get_version(&self, file_id: &str, version_number: u32) -> Option<FileVersion> {
        let versions = self.versions.read().await;
        versions.get(file_id).and_then(|v| {
            v.iter()
                .find(|ver| ver.version_number == version_number)
                .cloned()
        })
    }

    /// 获取当前版本
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    ///
    /// # Returns
    /// 当前版本信息（如果存在）
    pub async fn get_current_version(&self, file_id: &str) -> Option<FileVersion> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(file_id).cloned()
    }

    /// 查询版本历史
    ///
    /// # Arguments
    /// * `query` - 查询参数
    ///
    /// # Returns
    /// 符合条件的版本列表
    pub async fn query_versions(&self, query: VersionHistoryQuery) -> Vec<FileVersion> {
        let versions = self.versions.read().await;

        versions
            .get(&query.file_id)
            .map(|file_versions| {
                file_versions
                    .iter()
                    .filter(|v| {
                        // 过滤条件
                        if let Some(from) = query.from_version
                            && v.version_number < from {
                                return false;
                            }
                        if let Some(to) = query.to_version
                            && v.version_number > to {
                                return false;
                            }
                        if let Some(state) = query.state
                            && v.state != state {
                                return false;
                            }
                        if let Some(created_by) = &query.created_by
                            && &v.created_by != created_by {
                                return false;
                            }
                        if let Some(start) = query.start_date
                            && v.created_at < start {
                                return false;
                            }
                        if let Some(end) = query.end_date
                            && v.created_at > end {
                                return false;
                            }
                        if let Some(tags) = &query.tags
                            && !tags.iter().any(|t| v.tags.contains(t)) {
                                return false;
                            }
                        true
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 比较两个版本
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    /// * `from_version` - 起始版本号
    /// * `to_version` - 目标版本号
    ///
    /// # Returns
    /// 版本差异信息
    pub async fn compare_versions(
        &self,
        file_id: &str,
        from_version: u32,
        to_version: u32,
    ) -> Option<VersionDiff> {
        // 检查版本跨度
        if (to_version - from_version) > self.config.compare.max_version_span {
            return None;
        }

        let versions = self.versions.read().await;

        let from_ver = versions
            .get(file_id)
            .and_then(|v| v.iter().find(|ver| ver.version_number == from_version));
        let to_ver = versions
            .get(file_id)
            .and_then(|v| v.iter().find(|ver| ver.version_number == to_version));

        match (from_ver, to_ver) {
            (Some(from), Some(to)) => {
                // 比较两个版本的差异类型
                let diff_type = if from.content_hash == to.content_hash {
                    // 内容相同，未修改
                    DiffType::Unchanged
                } else if from.file_size == 0 && to.file_size > 0 {
                    // 从空文件变为有内容，视为新增
                    DiffType::Added
                } else if to.file_size == 0 && from.file_size > 0 {
                    // 从有内容变为空文件，视为删除
                    DiffType::Removed
                } else {
                    // 内容改变，视为修改
                    DiffType::Modified
                };

                let added_bytes = to.file_size.saturating_sub(from.file_size);

                let removed_bytes = from.file_size.saturating_sub(to.file_size);

                let net_change = to.file_size as i64 - from.file_size as i64;

                Some(VersionDiff {
                    from_version: from.version_id.clone(),
                    to_version: to.version_id.clone(),
                    diff_type,
                    added_bytes,
                    removed_bytes,
                    net_change,
                    changes: vec![
                        DiffChange {
                            change_type: "size_change".to_string(),
                            position: None,
                            content: Some(format!(
                                "文件大小从 {} 字节变为 {} 字节",
                                from.file_size, to.file_size
                            )),
                        },
                        DiffChange {
                            change_type: "hash_change".to_string(),
                            position: None,
                            content: Some(format!(
                                "内容哈希: {} -> {}",
                                from.content_hash, to.content_hash
                            )),
                        },
                    ],
                })
            }
            _ => None,
        }
    }

    /// 回退到指定版本
    ///
    /// # Arguments
    /// * `request` - 回退请求
    ///
    /// # Returns
    /// 回退结果
    pub async fn rollback(&self, request: RollbackRequest) -> RollbackResult {
        let RollbackRequest {
            file_id,
            target_version,
            reason,
            create_backup,
        } = request;

        // 获取目标版本
        let target = self.get_version(&file_id, target_version).await;

        let Some(target_ver) = target else {
            return RollbackResult {
                success: false,
                new_version_id: String::new(),
                new_version_number: 0,
                backup_version_id: None,
                message: format!("版本 {target_version} 不存在"),
            };
        };

        // 创建备份版本（当前版本）
        let backup_version_id = if create_backup {
            if let Some(current) = self.get_current_version(&file_id).await {
                let backup = self
                    .create_version(
                        CreateVersionParams {
                            file_id: file_id.clone(),
                            file_name: current.file_name.clone(),
                            file_path: current.file_path.clone(),
                            file_size: current.file_size,
                            content_hash: current.content_hash.clone(),
                            storage_path: current.storage_path.clone(),
                            created_by: "system".to_string(),
                            description: Some("自动备份".to_string()),
                        },
                    )
                    .await;
                Some(backup.version_id)
            } else {
                None
            }
        } else {
            None
        };

        // 创建新版本，内容与目标版本相同
        let new_version = self
            .create_version(
                CreateVersionParams {
                    file_id: file_id.clone(),
                    file_name: target_ver.file_name.clone(),
                    file_path: target_ver.file_path.clone(),
                    file_size: target_ver.file_size,
                    content_hash: target_ver.content_hash.clone(),
                    storage_path: target_ver.storage_path.clone(),
                    created_by: "system".to_string(),
                    description: Some(format!(
                        "回退到版本 {} - {}",
                        target_version,
                        reason.unwrap_or_else(|| "用户操作".to_string())
                    )),
                },
            )
            .await;

        RollbackResult {
            success: true,
            new_version_id: new_version.version_id,
            new_version_number: new_version.version_number,
            backup_version_id,
            message: format!("成功回退到版本 {target_version}"),
        }
    }

    /// 添加版本标签
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    /// * `version_number` - 版本号
    /// * `tag` - 标签
    ///
    /// # Returns
    /// 是否成功
    pub async fn add_tag(&self, file_id: &str, version_number: u32, tag: &str) -> bool {
        let mut versions = self.versions.write().await;

        if let Some(file_versions) = versions.get_mut(file_id)
            && let Some(version) = file_versions
                .iter_mut()
                .find(|v| v.version_number == version_number)
                && !version.tags.contains(&tag.to_string()) {
                    version.tags.push(tag.to_string());
                    return true;
                }
        false
    }

    /// 移除版本标签
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    /// * `version_number` - 版本号
    /// * `tag` - 标签
    ///
    /// # Returns
    /// 是否成功
    pub async fn remove_tag(&self, file_id: &str, version_number: u32, tag: &str) -> bool {
        let mut versions = self.versions.write().await;

        if let Some(file_versions) = versions.get_mut(file_id)
            && let Some(version) = file_versions
                .iter_mut()
                .find(|v| v.version_number == version_number)
            {
                let pos = version.tags.iter().position(|t| t == tag);
                if let Some(idx) = pos {
                    version.tags.remove(idx);
                    return true;
                }
            }
        false
    }

    /// 归档版本
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    /// * `version_number` - 版本号
    ///
    /// # Returns
    /// 是否成功
    pub async fn archive_version(&self, file_id: &str, version_number: u32) -> bool {
        let mut versions = self.versions.write().await;

        if let Some(file_versions) = versions.get_mut(file_id)
            && let Some(version) = file_versions
                .iter_mut()
                .find(|v| v.version_number == version_number)
            {
                version.state = VersionState::Archived;
                return true;
            }
        false
    }

    /// 删除版本（软删除）
    ///
    /// # Arguments
    /// * `file_id` - 文件 ID
    /// * `version_number` - 版本号
    ///
    /// # Returns
    /// 是否成功
    pub async fn delete_version(&self, file_id: &str, version_number: u32) -> bool {
        let mut versions = self.versions.write().await;

        if let Some(file_versions) = versions.get_mut(file_id)
            && let Some(version) = file_versions
                .iter_mut()
                .find(|v| v.version_number == version_number)
            {
                // 不允许删除初始版本
                if version.is_initial {
                    return false;
                }
                version.state = VersionState::Deleted;
                return true;
            }
        false
    }

    /// 获取版本统计
    pub async fn get_stats(&self) -> VersionStats {
        let versions = self.versions.read().await;

        let total_files = versions.len();
        let total_versions: usize = versions.values().map(std::vec::Vec::len).sum();

        let total_size: u64 = versions
            .values()
            .flat_map(|v| v.iter())
            .map(|v| v.file_size)
            .sum();

        let active_count: usize = versions
            .values()
            .flat_map(|v| v.iter())
            .filter(|v| v.state == VersionState::Active)
            .count();

        let archived_count: usize = versions
            .values()
            .flat_map(|v| v.iter())
            .filter(|v| v.state == VersionState::Archived)
            .count();

        VersionStats {
            total_files,
            total_versions,
            total_size,
            active_count,
            archived_count,
        }
    }

    /// 清理过期版本
    ///
    /// # Arguments
    /// * `retention_days` - 保留天数
    ///
    /// # Returns
    /// 删除的版本数量
    pub async fn cleanup_expired(&self, retention_days: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut versions = self.versions.write().await;
        let mut total_removed = 0;

        for file_versions in versions.values_mut() {
            let before = file_versions.len();
            file_versions.retain(|v| v.state != VersionState::Deleted && v.created_at > cutoff);
            total_removed += before - file_versions.len();

            // 标记删除超过保留期的版本
            for v in file_versions.iter_mut() {
                if v.created_at <= cutoff && v.state == VersionState::Active {
                    v.state = VersionState::Archived;
                }
            }
        }

        total_removed
    }
}

/// 版本统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionStats {
    pub total_files: usize,
    pub total_versions: usize,
    pub total_size: u64,
    pub active_count: usize,
    pub archived_count: usize,
}

/// 生成版本 ID
fn generate_version_id(file_id: &str, version_number: u32) -> String {
    format!("{file_id}_v{version_number:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_version() {
        let manager = VersionManager::default_manager();

        let v1 = manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "document.txt".to_string(),
                    file_path: "/docs/document.txt".to_string(),
                    file_size: 1024,
                    content_hash: "hash_v1".to_string(),
                    storage_path: "/storage/v1".to_string(),
                    created_by: "user_001".to_string(),
                    description: Some("初始版本".to_string()),
                },
            )
            .await;

        assert_eq!(v1.version_number, 1);
        assert_eq!(v1.file_name, "document.txt");
        assert!(v1.is_initial);

        // 创建第二个版本
        let v2 = manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "document.txt".to_string(),
                    file_path: "/docs/document.txt".to_string(),
                    file_size: 2048,
                    content_hash: "hash_v2".to_string(),
                    storage_path: "/storage/v2".to_string(),
                    created_by: "user_001".to_string(),
                    description: Some("更新".to_string()),
                },
            )
            .await;

        assert_eq!(v2.version_number, 2);
        assert!(!v2.is_initial);
        assert_eq!(v2.delta_size, Some(1024));
    }

    #[tokio::test]
    async fn test_get_versions() {
        let manager = VersionManager::default_manager();

        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 100,
                    content_hash: "h1".to_string(),
                    storage_path: "/s/1".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 200,
                    content_hash: "h2".to_string(),
                    storage_path: "/s/2".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 300,
                    content_hash: "h3".to_string(),
                    storage_path: "/s/3".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;

        let versions = manager.get_versions("file_001").await;
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].version_number, 3); // 降序排列
        assert_eq!(versions[1].version_number, 2);
        assert_eq!(versions[2].version_number, 1);
    }

    #[tokio::test]
    async fn test_compare_versions() {
        let manager = VersionManager::default_manager();

        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 1000,
                    content_hash: "h1".to_string(),
                    storage_path: "/s/1".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 2000,
                    content_hash: "h2".to_string(),
                    storage_path: "/s/2".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;

        let diff = manager.compare_versions("file_001", 1, 2).await;
        assert!(diff.is_some());

        let diff = diff.unwrap();
        assert_eq!(diff.diff_type, DiffType::Modified);
        assert_eq!(diff.added_bytes, 1000);
        assert_eq!(diff.removed_bytes, 0);
        assert_eq!(diff.net_change, 1000);
    }

    #[tokio::test]
    async fn test_rollback() {
        let manager = VersionManager::default_manager();

        // 创建 3 个版本
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 1000,
                    content_hash: "h1".to_string(),
                    storage_path: "/s/1".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 2000,
                    content_hash: "h2".to_string(),
                    storage_path: "/s/2".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 3000,
                    content_hash: "h3".to_string(),
                    storage_path: "/s/3".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;

        let result = manager
            .rollback(RollbackRequest {
                file_id: "file_001".to_string(),
                target_version: 1,
                reason: Some("回退到初始版本".to_string()),
                create_backup: true,
            })
            .await;

        assert!(result.success);
        // 回退创建了 2 个版本：备份 (v4) + 回退版本 (v5)
        assert_eq!(result.new_version_number, 5);
        assert!(result.backup_version_id.is_some());

        // 验证新版本内容与目标版本一致
        let current = manager.get_current_version("file_001").await;
        assert!(current.is_some());
        assert_eq!(current.unwrap().file_size, 1000);
    }

    #[tokio::test]
    async fn test_tags() {
        let manager = VersionManager::default_manager();

        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 1000,
                    content_hash: "h1".to_string(),
                    storage_path: "/s/1".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;

        let added = manager.add_tag("file_001", 1, "important").await;
        assert!(added);

        let version = manager.get_version("file_001", 1).await;
        assert!(version.is_some());
        assert!(version.unwrap().tags.contains(&"important".to_string()));

        let removed = manager.remove_tag("file_001", 1, "important").await;
        assert!(removed);

        let version = manager.get_version("file_001", 1).await;
        assert!(version.unwrap().tags.is_empty());
    }

    #[tokio::test]
    async fn test_archive_and_delete() {
        let manager = VersionManager::default_manager();

        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 1000,
                    content_hash: "h1".to_string(),
                    storage_path: "/s/1".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc.txt".to_string(),
                    file_path: "/doc.txt".to_string(),
                    file_size: 2000,
                    content_hash: "h2".to_string(),
                    storage_path: "/s/2".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;

        // 归档版本 1
        let archived = manager.archive_version("file_001", 1).await;
        assert!(archived);

        // 验证状态
        let v1 = manager.get_version("file_001", 1).await;
        assert_eq!(v1.unwrap().state, VersionState::Archived);

        // 删除版本 2（不是初始版本）
        let deleted = manager.delete_version("file_001", 2).await;
        assert!(deleted);

        // 尝试删除初始版本（应该失败）
        let deleted = manager.delete_version("file_001", 1).await;
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = VersionManager::default_manager();

        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_001".to_string(),
                    file_name: "doc1.txt".to_string(),
                    file_path: "/doc1.txt".to_string(),
                    file_size: 1000,
                    content_hash: "h1".to_string(),
                    storage_path: "/s/1".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_002".to_string(),
                    file_name: "doc2.txt".to_string(),
                    file_path: "/doc2.txt".to_string(),
                    file_size: 2000,
                    content_hash: "h2".to_string(),
                    storage_path: "/s/2".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;
        manager
            .create_version(
                CreateVersionParams {
                    file_id: "file_002".to_string(),
                    file_name: "doc2.txt".to_string(),
                    file_path: "/doc2.txt".to_string(),
                    file_size: 2500,
                    content_hash: "h3".to_string(),
                    storage_path: "/s/3".to_string(),
                    created_by: "u1".to_string(),
                    description: None,
                },
            )
            .await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_versions, 3);
        assert_eq!(stats.total_size, 5500);
    }
}
