//! 用户仓储层 - 抽取数据库操作逻辑

use sqlx::PgPool;
use thiserror::Error;

/// 用户仓储错误类型
#[derive(Error, Debug)]
pub enum UserRepositoryError {
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("用户不存在" )]
    NotFound,

    #[error("用户已存在" )]
    AlreadyExists,
}

impl From<UserRepositoryError> for common::AppError {
    fn from(err: UserRepositoryError) -> Self {
        match err {
            UserRepositoryError::NotFound => Self::UserNotFound,
            UserRepositoryError::AlreadyExists => {
                Self::UserAlreadyExists("用户已存在".to_string())
            }
            UserRepositoryError::Database(e) => Self::Database(e),
        }
    }
}

/// 用户基本信息（扩展版）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: Option<String>,
    pub status: i32,
    /// 用户角色: admin-管理员, user-普通用户
    pub role: String,
    /// 首次登录需修改密码
    pub must_change_password: bool,
}

/// 用户仓储
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// 创建新的用户仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据用户名查找用户
    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserInfo>, UserRepositoryError> {
        let row = sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, password_hash, email, phone, nickname, status, role, must_change_password FROM users WHERE username = $1"
        )
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    /// 根据用户ID查找用户
    pub async fn find_by_id(&self, user_id: i64) -> Result<Option<UserInfo>, UserRepositoryError> {
        let row = sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, password_hash, email, phone, nickname, status, role, must_change_password FROM users WHERE id = $1"
        )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    /// 创建新用户
    pub async fn create(
        &self,
        username: &str,
        password_hash: &str,
        email: Option<String>,
    ) -> Result<i64, UserRepositoryError> {
        // 先检查用户是否已存在
        if self.find_by_username(username).await?.is_some() {
            return Err(UserRepositoryError::AlreadyExists);
        }

        let row = sqlx::query_scalar::<_, i64>(
            "INSERT INTO users (username, email, password_hash, created_at) VALUES ($1, $2, $3, NOW()) RETURNING id"
        )
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(row)
    }

    /// 检查用户名是否存在
    pub async fn exists(&self, username: &str) -> Result<bool, UserRepositoryError> {
        let row = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"#
        )
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(row)
    }

    /// 更新用户密码
    pub async fn update_password(
        &self,
        user_id: i64,
        new_password_hash: &str,
    ) -> Result<(), UserRepositoryError> {
        let result =
            sqlx::query(
                "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"
            )
            .bind(new_password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(UserRepositoryError::NotFound);
        }

        Ok(())
    }

    /// 更新用户资料信息
    pub async fn update_info(
        &self,
        user_id: i64,
        nickname: Option<&str>,
        _avatar: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> Result<(), UserRepositoryError> {
        // 构建动态更新语句
        let result = sqlx::query(
            "UPDATE users SET 
                nickname = COALESCE($1, nickname),
                phone = COALESCE($2, phone),
                email = COALESCE($3, email),
                updated_at = NOW()
             WHERE id = $4"
        )
            .bind(nickname)
            .bind(phone)
            .bind(email)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(UserRepositoryError::NotFound);
        }

        Ok(())
    }

    /// 清除 must_change_password 标记（修改密码后调用）
    pub async fn clear_must_change_password(
        &self,
        user_id: i64,
    ) -> Result<(), UserRepositoryError> {
        sqlx::query(
            "UPDATE users SET must_change_password = false, updated_at = NOW() WHERE id = $1"
        )
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 更新用户头像
    pub async fn update_avatar(
        &self,
        user_id: i64,
        avatar: &str,
    ) -> Result<(), UserRepositoryError> {
        let result = sqlx::query(
            "UPDATE users SET avatar = $1, updated_at = NOW() WHERE id = $2"
        )
            .bind(avatar)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(UserRepositoryError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_repository_error_display() {
        let err = UserRepositoryError::NotFound;
        assert_eq!(err.to_string(), "用户不存在" );

        let err = UserRepositoryError::AlreadyExists;
        assert_eq!(err.to_string(), "用户已存在" );
    }
}
