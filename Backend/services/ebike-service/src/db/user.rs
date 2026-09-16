//! 用户数据仓库

use sqlx::postgres::PgPool;

use crate::model::UserInfo;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserInfo>, sqlx::Error> {
        let row = sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, nickname, password_hash, avatar, phone, email, gender, address, role, status, created_at, updated_at FROM public.users WHERE username = $1" ,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn user_info(&self, id: i64) -> Result<Option<UserInfo>, sqlx::Error> {
        let row = sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, nickname, password_hash, avatar, phone, email, gender, address, role, status, created_at, updated_at FROM public.users WHERE id = $1" ,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }
}
