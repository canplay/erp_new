//! Workflow 数据仓储层 - 事件
//!
//! 报表和定时任务的 CRUD 操作

use parking_lot::RwLock;
use std::collections::HashMap;

// ============================================================================
// Report Repository
// ============================================================================

pub trait ReportRepository: Send + Sync {
    fn create(
        &self,
        report: &super::Report,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        report: &super::Report,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::Report>, sqlx::Error>> + Send;
    fn list(
        &self,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::Report>, i64), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
}

pub struct InMemoryReportRepository {
    reports: RwLock<HashMap<String, super::Report>>,
}

impl InMemoryReportRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            reports: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryReportRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportRepository for InMemoryReportRepository {
    async fn create(&self, report: &super::Report) -> Result<(), sqlx::Error> {
        let mut reports = self.reports.write();
        reports.insert(report.id.clone(), report.clone());
        Ok(())
    }

    async fn update(&self, report: &super::Report) -> Result<(), sqlx::Error> {
        let mut reports = self.reports.write();
        reports.insert(report.id.clone(), report.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<super::Report>, sqlx::Error> {
        let reports = self.reports.read();
        Ok(reports.get(id).cloned())
    }

    async fn list(&self, page: i64, page_size: i64) -> Result<(Vec<super::Report>, i64), sqlx::Error> {
        let reports = self.reports.read();
        let mut all: Vec<_> = reports.values().cloned().collect();
        let total = all.len() as i64;
        all.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, all.len());
        Ok((all[offset..end].to_vec(), total))
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut reports = self.reports.write();
        reports.remove(id);
        Ok(())
    }
}

// ============================================================================
// ScheduledTask Repository
// ============================================================================

pub trait ScheduledTaskRepository: Send + Sync {
    fn create(
        &self,
        task: &super::ScheduledTask,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update(
        &self,
        task: &super::ScheduledTask,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<super::ScheduledTask>, sqlx::Error>> + Send;
    fn list(
        &self,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<super::ScheduledTask>, i64), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
}

pub struct InMemoryScheduledTaskRepository {
    tasks: RwLock<HashMap<String, super::ScheduledTask>>,
}

impl InMemoryScheduledTaskRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryScheduledTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduledTaskRepository for InMemoryScheduledTaskRepository {
    async fn create(&self, task: &super::ScheduledTask) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn update(&self, task: &super::ScheduledTask) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<super::ScheduledTask>, sqlx::Error> {
        let tasks = self.tasks.read();
        Ok(tasks.get(id).cloned())
    }

    async fn list(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<super::ScheduledTask>, i64), sqlx::Error> {
        let tasks = self.tasks.read();
        let mut all: Vec<_> = tasks.values().cloned().collect();
        let total = all.len() as i64;
        all.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        let offset = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(offset + page_size as usize, all.len());
        Ok((all[offset..end].to_vec(), total))
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut tasks = self.tasks.write();
        tasks.remove(id);
        Ok(())
    }
}
