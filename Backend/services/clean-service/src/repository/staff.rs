// 员工仓储
// Staff repository

use sqlx::PgPool;

use crate::error::{CleanServiceError};
use crate::error::CleanResult;
use crate::models::Staff;

/// 员工仓储
pub struct StaffRepository {pool: PgPool}

impl StaffRepository {
  /// 创建员工仓储实例
  /// 审计修复 (H8-架构评审): 注入共享 PgPool, 不再每仓储自建连接池(原 4 池 x 10 连接)
  #[must_use]
 pub fn new(pool: PgPool) -> Self {
 Self { pool}
 }

 /// 根据ID查询员工
 pub async fn get_by_id(&self, id: &str) -> CleanResult<Option<Staff>> {let staff = sqlx::query_as::<_, Staff>("SELECT * FROM t_s_base_user WHERE id = $1")
 .bind(id)
 .fetch_optional(&self.pool)
 .await
 .map_err(|e: sqlx::Error| CleanServiceError::DatabaseError(e.to_string()))?;

 Ok(staff)}
}
