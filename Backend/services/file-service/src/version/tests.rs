#[cfg(test)]
mod version_tests {
    use crate::version::*;

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
        assert_eq!(v1.file_name, "document.txt" );
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

        let versions = manager.get_versions("file_001" ).await;
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

        let diff = manager.compare_versions("file_001" , 1, 2).await;
        assert!(diff.is_some());

        let diff = diff.expect("diff should exist" );
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
        let current = manager.get_current_version("file_001" ).await;
        assert!(current.is_some());
        assert_eq!(current.expect("current version should exist" ).file_size, 1000);
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

        let added = manager.add_tag("file_001" , 1, "important" ).await;
        assert!(added);

        let version = manager.get_version("file_001" , 1).await;
        assert!(version.is_some());
        assert!(version.expect("version should exist" ).tags.contains(&"important".to_string()));

        let removed = manager.remove_tag("file_001" , 1, "important" ).await;
        assert!(removed);

        let version = manager.get_version("file_001" , 1).await;
        assert!(version.expect("version should exist" ).tags.is_empty());
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
        let archived = manager.archive_version("file_001" , 1).await;
        assert!(archived);

        // 验证状态
        let v1 = manager.get_version("file_001" , 1).await;
        assert_eq!(v1.expect("v1 should exist" ).state, VersionState::Archived);

        // 删除版本 2（不是初始版本）
        let deleted = manager.delete_version("file_001" , 2).await;
        assert!(deleted);

        // 尝试删除初始版本（应该失败）
        let deleted = manager.delete_version("file_001" , 1).await;
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
