//! 版本管理器实现
//!
//! 提供版本创建、查询、对比、回退、标签管理、统计与清理

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::helper::generate_version_id;
use super::types::{
    DiffChange, DiffType, FileVersion, RollbackRequest, RollbackResult,
    VersionConfig, VersionDiff, VersionHistoryQuery, VersionState, VersionStats,
};

/// 版本管理器
#[derive(Clone)]
pub struct VersionManager {
    /// 版本存储（file_id -> Vec<FileVersion>）
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
                                "文件大小从 {} 字节变为 {} 字节" ,
                                from.file_size, to.file_size
                            )),
                        },
                        DiffChange {
                            change_type: "hash_change".to_string(),
                            position: None,
                            content: Some(format!(
                                "内容哈希: {} -> {}" ,
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
                message: format!("版本 {target_version} 不存在" ),
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
                        "回退到版本 {} - {}" ,
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
            message: format!("成功回退到版本 {target_version}" ),
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
