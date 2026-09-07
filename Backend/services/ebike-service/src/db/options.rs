//! 系统配置数据仓库

use chrono::Local;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Error;

use crate::model::OptionsInfo;

pub struct OptionsRepository {
    pool: PgPool,
}

impl OptionsRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn query(&self) -> Result<Vec<OptionsInfo>, Error> {
        let rows = sqlx::query_as!(
            OptionsInfo,
            r#"SELECT COALESCE(name, '') AS "name!",
                      COALESCE(options, '{}'::json) AS "options!",
                      COALESCE(level, 0) AS "level!",
                      create_date, update_date, delete
               FROM public.options WHERE delete = false"#,
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            Ok(vec![OptionsInfo {
                name: String::new(),
                options: json!({ "system": 60, "alert": 30 }),
                level: 0,
                create_date: Some(Local::now().naive_local()),
                update_date: Some(Local::now().naive_local()),
                delete: Some(false),
            }])
        } else {
            Ok(rows)
        }
    }

    pub async fn update(&self, info: &OptionsInfo) -> Result<bool, Error> {
        let rows = sqlx::query!("SELECT name FROM public.options WHERE name = $1", &info.name)
            .fetch_all(&self.pool)
            .await?;

        if rows.is_empty() {
            sqlx::query!(
                "INSERT INTO public.options VALUES ($1, $2, $3, $4, $5, $6)",
                &info.name,
                &info.options,
                Local::now().naive_local(),
                Local::now().naive_local(),
                false,
                info.level,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE public.options SET name = $1, options = $2, update_date = $3, delete = $4, level = $5 WHERE name = $6",
                &info.name,
                &info.options,
                Local::now().naive_local(),
                false,
                info.level,
                &info.name,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(true)
    }
}
