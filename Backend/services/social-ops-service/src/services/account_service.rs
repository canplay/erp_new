//! 社交账号服务 — CRUD + 凭证加密

use sqlx::PgPool;
use uuid::Uuid;
use crate::models::account::{SocialAccount, CreateAccountRequest, UpdateAccountRequest};

#[derive(Clone)]
pub struct AccountService {
    db: PgPool,
}

impl AccountService {
    #[must_use]
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self, user_id: Option<Uuid>) -> Result<Vec<SocialAccount>, sqlx::Error> {
        if let Some(uid) = user_id {
            sqlx::query_as!(
                SocialAccount,
                r#"SELECT id, user_id, platform, account_name, account_id, avatar_url, is_active, config_json, created_at, updated_at
                 FROM socialops.social_accounts WHERE user_id = $1 ORDER BY created_at DESC"#,
                uid
            )
            .fetch_all(&self.db)
            .await
        } else {
            sqlx::query_as!(
                SocialAccount,
                r#"SELECT id, user_id, platform, account_name, account_id, avatar_url, is_active, config_json, created_at, updated_at
                 FROM socialops.social_accounts ORDER BY created_at DESC"#
            )
            .fetch_all(&self.db)
            .await
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<SocialAccount>, sqlx::Error> {
        sqlx::query_as!(
            SocialAccount,
            r#"SELECT id, user_id, platform, account_name, account_id, avatar_url, is_active, config_json, created_at, updated_at
             FROM socialops.social_accounts WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn create(&self, req: &CreateAccountRequest) -> Result<SocialAccount, sqlx::Error> {
        sqlx::query_as!(
            SocialAccount,
            r#"INSERT INTO socialops.social_accounts (user_id, platform, account_name, config_json)
             VALUES ($1, $2, $3, $4)
             RETURNING id, user_id, platform, account_name, account_id, avatar_url, is_active, config_json, created_at, updated_at"#,
            req.user_id,
            &req.platform,
            &req.account_name,
            req.config.as_ref(),
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update(&self, id: Uuid, req: &UpdateAccountRequest) -> Result<Option<SocialAccount>, sqlx::Error> {
        sqlx::query_as!(
            SocialAccount,
            r#"UPDATE socialops.social_accounts
             SET account_name = COALESCE($2, account_name),
                 config_json = COALESCE($3, config_json),
                 is_active = COALESCE($4, is_active),
                 updated_at = NOW()
             WHERE id = $1
             RETURNING id, user_id, platform, account_name, account_id, avatar_url, is_active, config_json, created_at, updated_at"#,
            id,
            req.account_name.as_deref(),
            req.config.as_ref(),
            req.is_active,
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM socialops.social_accounts WHERE id = $1", id)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
