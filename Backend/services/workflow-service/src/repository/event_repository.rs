//! Workflow 数据仓储层 - 事件
//!
//! 报表和定时任务的 CRUD 操作


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




