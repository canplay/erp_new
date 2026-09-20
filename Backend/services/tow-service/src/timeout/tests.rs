//! 超时模块单元测试
#![cfg(test)]

#[cfg(test)]
mod tests {
    use crate::timeout::*;

    #[tokio::test]
    async fn test_create_task() {
        let manager = TimeoutManager::default_manager();

        let task = manager
            .create_task(
                "task_001" ,
                "data_processing" ,
                Some(60),
                Some(TimeoutStrategy::Retry),
            )
            .await;

        assert_eq!(task.task_id, "task_001" );
        assert_eq!(task.status, TimeoutState::Running);
        assert_eq!(task.timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001" , "type1" , None, None).await;

        let completed = manager.complete_task("task_001" ).await;
        assert!(completed);

        let task = manager.get_task("task_001" ).await;
        assert_eq!(task.expect("task should exist" ).status, TimeoutState::Processed);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001" , "type1" , None, None).await;

        let cancelled = manager.cancel_task("task_001" ).await;
        assert!(cancelled);

        let task = manager.get_task("task_001" ).await;
        assert_eq!(task.expect("task should exist" ).status, TimeoutState::Cancelled);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001" , "type1" , None, None).await;
        manager.create_task("task_002" , "type2" , None, None).await;
        manager.complete_task("task_001" ).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.processed_tasks, 1);
        assert_eq!(stats.running_tasks, 1);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001" , "type1" , None, None).await;
        manager.complete_task("task_001" ).await;

        // 清理已完成的任务
        let removed = manager.cleanup(0).await;
        assert_eq!(removed, 1);
    }

    #[tokio::test]
    async fn test_events() {
        let manager = TimeoutManager::default_manager();

        manager.create_task("task_001" , "type1" , None, None).await;
        manager.complete_task("task_001" ).await;

        let events = manager.get_task_events("task_001" ).await;
        assert!(events.len() >= 2);
    }
}
