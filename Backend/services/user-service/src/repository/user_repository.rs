//! 用户仓储层

use chrono::Utc;
use common::constants::{MAX_PAGE_SIZE, MIN_PAGE};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use thiserror::Error;

/// 用户仓储错误类型
#[derive(Error, Debug)]
pub(crate) enum UserRepositoryError {
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("用户不存在" )]
    NotFound,

    #[error("用户已存在" )]
    AlreadyExists,
}

/// 用户详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UserDetail {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub gender: Option<i32>,
    pub address: Option<String>,
    pub role: String,
    pub status: i32,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// 用户列表项（不含敏感信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UserListItem {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub gender: Option<i32>,
    pub address: Option<String>,
    pub role: String,
    pub status: i32,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// 分页结果
pub(crate) struct PaginatedUsers {
    pub users: Vec<UserListItem>,
    pub total: i64,
}

/// 批量操作结果
#[derive(Debug)]
pub(crate) struct BatchOperationResult {
    pub success_count: usize,
    pub fail_count: usize,
    pub errors: Vec<BatchError>,
}

/// 批量操作错误
#[derive(Debug)]
pub(crate) struct BatchError {
    pub id: i64,
    pub message: String,
}

/// 导入导出操作结果
#[derive(Debug, Default)]
pub(crate) struct ImportExportResult {
    pub total: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub errors: Vec<String>,
}

impl From<UserRepositoryError> for common::AppError {
    fn from(err: UserRepositoryError) -> Self {
        match err {
            UserRepositoryError::NotFound => Self::UserNotFound,
            UserRepositoryError::AlreadyExists => {
                Self::UserAlreadyExists("用户名已存在".to_string())
            }
            UserRepositoryError::Database(e) => Self::Database(e),
        }
    }
}

/// 用户仓储
#[derive(Clone)]
pub(crate) struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// 创建新的用户仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建用户（使用 INSERT ... ON CONFLICT 避免重复查询）
    pub(crate) async fn create(
        &self,
        username: &str,
        password_hash: &str,
        email: Option<String>,
        nickname: Option<String>,
        phone: Option<String>,
        gender: Option<i32>,
    ) -> Result<i64, UserRepositoryError> {
        // 使用 INSERT ... ON CONFLICT 直接处理，避免额外查询
        let row = sqlx::query!(
            r"INSERT INTO users (username, password_hash, email, nickname, phone, gender, role, status)
               VALUES ($1, $2, $3, $4, $5, $6, 'user', 1)
               ON CONFLICT (username) DO NOTHING
               RETURNING id" ,
            username,
            password_hash,
            email.as_deref(),
            nickname.as_deref(),
            phone.as_deref(),
            gender,
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.id)
            .ok_or(UserRepositoryError::AlreadyExists)
    }

    /// 根据用户名查找用户
    pub(crate) async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserDetail>, UserRepositoryError> {
        let row = sqlx::query_as!(
            UserDetail,
            r#"SELECT id, username, nickname, avatar, phone, email, gender, address,
                       COALESCE(role, 'user') AS "role!" ,
                       COALESCE(status, 1) AS "status!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
                FROM users WHERE username = $1"#,
            username,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 根据用户ID查找用户
    pub(crate) async fn find_by_id(
        &self,
        user_id: i64,
    ) -> Result<Option<UserDetail>, UserRepositoryError> {
        let row = sqlx::query_as!(
            UserDetail,
            r#"SELECT id, username, nickname, avatar, phone, email, gender, address,
                       COALESCE(role, 'user') AS "role!" ,
                       COALESCE(status, 1) AS "status!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
                FROM users WHERE id = $1"#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 批量根据用户ID查找用户（N+1 修复：使用 WHERE id = ANY($1)）
    pub(crate) async fn find_all_by_ids(
        &self,
        user_ids: &[i64],
    ) -> Result<Vec<UserDetail>, UserRepositoryError> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(
            r#"SELECT id, username, nickname, avatar, phone, email, gender, address,
                      COALESCE(role, 'user') AS "role",
                      COALESCE(status, 1) AS "status",
                      COALESCE(created_at, NOW()) AS "created_at",
                      COALESCE(updated_at, NOW()) AS "updated_at"
               FROM users WHERE id = ANY($1)"#
        )
        .bind(user_ids)
        .fetch_all(&self.pool)
        .await?;

        let users: Vec<UserDetail> = rows.iter().map(|row| {
            UserDetail {
                id: row.get("id"),
                username: row.get("username"),
                nickname: row.get("nickname"),
                avatar: row.get("avatar"),
                phone: row.get("phone"),
                email: row.get("email"),
                gender: row.get("gender"),
                address: row.get("address"),
                role: row.get("role"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        Ok(users)
    }

    /// 更新用户信息
    pub(crate) async fn update(
        &self,
        user_id: i64,
        nickname: Option<String>,
        gender: Option<i32>,
        address: Option<String>,
        avatar: Option<String>,
    ) -> Result<Option<UserDetail>, UserRepositoryError> {
                let row = sqlx::query_as!(
            UserDetail,
            r#"UPDATE users
               SET nickname = COALESCE($1, nickname),
                   gender = COALESCE($2, gender),
                   address = COALESCE($3, address),
                   avatar = COALESCE($4, avatar),
                   updated_at = NOW()
               WHERE id = $5
               RETURNING id, username, nickname, avatar, phone, email, gender, address,
                         COALESCE(role, 'user') AS "role!" ,
                         COALESCE(status, 1) AS "status!" ,
                         COALESCE(created_at, NOW()) AS "created_at!" ,
                         COALESCE(updated_at, NOW()) AS "updated_at!" "#,
            nickname.as_deref(),
            gender,
            address.as_deref(),
            avatar.as_deref(),
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 删除用户
    pub(crate) async fn delete(&self, user_id: i64) -> Result<bool, UserRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1" ,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页查询用户列表
        /// 分页查询用户列表
    pub(crate) async fn list(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedUsers, UserRepositoryError> {
        // 规范化分页参数
        let page = page.max(MIN_PAGE);
        let page_size = page_size.clamp(1, MAX_PAGE_SIZE);
        let offset = (page - 1) * page_size;

        // 查询总数
        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users" )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        // 查询列表
        let users: Vec<UserListItem> = sqlx::query_as!(
            UserListItem,
            r#"SELECT id, username, nickname, avatar, phone, email, gender, address,
                       COALESCE(role, 'user') AS "role!" ,
                       COALESCE(status, 1) AS "status!" ,
                       COALESCE(created_at, NOW()) AS "created_at!" ,
                       COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM users
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedUsers { users, total })
    }

    /// 更新用户状态
    pub(crate) async fn update_status(
        &self,
        user_id: i64,
        status: i32,
    ) -> Result<bool, UserRepositoryError> {
        let result =
            sqlx::query!(
                r"UPDATE users SET status = $1, updated_at = NOW() WHERE id = $2" ,
                status,
                user_id,
            )
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 更新用户角色
    pub(crate) async fn update_role(&self, user_id: i64, role: &str) -> Result<bool, UserRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2" ,
            role,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 更新用户密码
    pub(crate) async fn update_password(
        &self,
        user_id: i64,
        password_hash: &str,
    ) -> Result<bool, UserRepositoryError> {
        let result =
            sqlx::query!(
                r"UPDATE users SET password_hash = $1, must_change_password = false, updated_at = NOW() WHERE id = $2" ,
                password_hash,
                user_id,
            )
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量更新状态（使用事务提高性能）
    pub(crate) async fn batch_update_status(
        &self,
        user_ids: &[i64],
        status: i32,
    ) -> Result<BatchOperationResult, UserRepositoryError> {
        if user_ids.is_empty() {
            return Ok(BatchOperationResult {
                success_count: 0,
                fail_count: 0,
                errors: vec![],
            });
        }

        let mut tx = self.pool.begin().await?;
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut errors = Vec::new();

        for user_id in user_ids {
            match sqlx::query!(
                "UPDATE users SET status = $1, updated_at = NOW() WHERE id = $2" ,
                status,
                user_id,
            )
            .execute(&mut *tx)
            .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        success_count += 1;
                    } else {
                        fail_count += 1;
                        errors.push(BatchError {
                            id: *user_id,
                            message: "用户不存在".to_string(),
                        });
                    }
                }
                Err(e) => {
                    fail_count += 1;
                    errors.push(BatchError {
                        id: *user_id,
                        message: e.to_string(),
                    });
                }
            }
        }

        tx.commit().await?;

        Ok(BatchOperationResult {
            success_count,
            fail_count,
            errors,
        })
    }

    /// 批量更新角色（使用事务提高性能）
    pub(crate) async fn batch_update_role(
        &self,
        user_ids: &[i64],
        role: &str,
    ) -> Result<BatchOperationResult, UserRepositoryError> {
        if user_ids.is_empty() {
            return Ok(BatchOperationResult {
                success_count: 0,
                fail_count: 0,
                errors: vec![],
            });
        }

        let mut tx = self.pool.begin().await?;
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut errors = Vec::new();

        for user_id in user_ids {
            match sqlx::query!(
                "UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2" ,
                role,
                user_id,
            )
            .execute(&mut *tx)
            .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        success_count += 1;
                    } else {
                        fail_count += 1;
                        errors.push(BatchError {
                            id: *user_id,
                            message: "用户不存在".to_string(),
                        });
                    }
                }
                Err(e) => {
                    fail_count += 1;
                    errors.push(BatchError {
                        id: *user_id,
                        message: e.to_string(),
                    });
                }
            }
        }

        tx.commit().await?;

        Ok(BatchOperationResult {
            success_count,
            fail_count,
            errors,
        })
    }

    /// 批量删除用户（使用事务提高性能）
    pub(crate) async fn batch_delete(
        &self,
        user_ids: &[i64],
    ) -> Result<BatchOperationResult, UserRepositoryError> {
        if user_ids.is_empty() {
            return Ok(BatchOperationResult {
                success_count: 0,
                fail_count: 0,
                errors: vec![],
            });
        }

        let mut tx = self.pool.begin().await?;
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut errors = Vec::new();

        for user_id in user_ids {
            match sqlx::query!(
                "DELETE FROM users WHERE id = $1" ,
                user_id,
            )
            .execute(&mut *tx)
            .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        success_count += 1;
                    } else {
                        fail_count += 1;
                        errors.push(BatchError {
                            id: *user_id,
                            message: "用户不存在".to_string(),
                        });
                    }
                }
                Err(e) => {
                    fail_count += 1;
                    errors.push(BatchError {
                        id: *user_id,
                        message: e.to_string(),
                    });
                }
            }
        }

        tx.commit().await?;

        Ok(BatchOperationResult {
            success_count,
            fail_count,
            errors,
        })
    }

    /// 从 CSV 批量创建用户（stub，待实现）
    pub(crate) async fn batch_create_from_csv(
        &self,
        _data_base64: &str,
        _format: &str,
        _update_mode: &str,
    ) -> Result<ImportExportResult, UserRepositoryError> {
        Ok(ImportExportResult {
            total: 0,
            success_count: 0,
            fail_count: 0,
            errors: vec!["导入功能待实现".to_string()],
        })
    }

    /// 导出用户为 CSV（stub，待实现）
    pub(crate) async fn export_to_csv(
        &self,
        _keyword: &str,
        _status: i32,
        _role: &str,
        _format: &str,
    ) -> Result<(Vec<u8>, String), UserRepositoryError> {
        let csv_content = "id,username,nickname,email,phone,status,role\n";
        Ok((csv_content.as_bytes().to_vec(), "users.csv".to_string()))
    }
}
