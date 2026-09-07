//! 车辆管理 Repository
//!
//! 提供拖车车辆的数据访问功能

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

// ============ 错误类型 ============

#[derive(Error, Debug)]
pub enum CarRepositoryError {
    #[error("数据库错误: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("记录不存在")]
    NotFound,

    #[error("记录已存在")]
    AlreadyExists,
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedCars {
    pub cars: Vec<CarListItem>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ============ 数据模型 ============

/// 车辆详情（添加 Serialize 以支持 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Car {
    pub id: i64,
    pub license: String,
    pub vehicle: Option<serde_json::Value>,
    pub engine: String,
    pub car_type: String,
    pub dc_type: String,
    pub dc_causes: String,
    pub car_color: String,
    pub dc_date: String,
    pub dc_address: String,
    pub dc_key: String,
    pub dc_party_name: String,
    pub dc_party_cardid: String,
    pub dc_party_tel: String,
    pub p_name: String,
    pub p_id: String,
    pub dc_acc: String,
    pub dc_name: String,
    pub dc_into_date: String,
    pub car_remark: String,
    pub driver: String,
    pub operator: String,
    pub drag_km: String,
    pub drag_unit: String,
    pub drag_money: String,
    pub cmd_unit: String,
    pub cmd_user: String,
    pub cv: String,
    pub cv_acc: String,
    pub cv_name: String,
    pub cv_date: String,
    pub cv_opinion: String,
    pub tv: String,
    pub tv_acc: String,
    pub tv_name: String,
    pub tv_date: String,
    pub tv_opinion: String,
    pub rc_name: String,
    pub rc_idcard: String,
    pub rc_tel: String,
    pub parking_date: String,
    pub parking_unit: String,
    pub parking_money: String,
    pub parking_payable: String,
    pub parking_paidin: String,
    pub remark: String,
    pub rs_name: String,
    pub rs_acc: String,
    pub rs_date: String,
    pub attachment: Option<serde_json::Value>,
    pub create_date: NaiveDateTime,
    pub create_user: String,
    pub update_date: NaiveDateTime,
    pub update_user: String,
    pub delete: bool,
}

/// 车辆列表项
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarListItem {
    pub id: i64,
    pub license: String,
    pub vehicle: Option<serde_json::Value>,
    pub engine: String,
    pub car_type: String,
    pub dc_type: String,
    pub dc_causes: String,
    pub car_color: String,
    pub dc_date: String,
    pub dc_address: String,
    pub dc_key: String,
    pub dc_party_name: String,
    pub dc_party_cardid: String,
    pub dc_party_tel: String,
    pub dc_name: String,
    pub cmd_unit: String,
    pub cv: String,
    pub tv: String,
    pub parking_unit: String,
    pub remark: String,
    pub rs_name: String,
    pub rs_date: String,
    pub delete: bool,
}

/// 车辆查询参数
#[derive(Debug, Default, Deserialize)]
pub struct CarQuery {
    pub status: Option<String>,
    pub in_date: Option<String>,
    pub out_date: Option<String>,
    pub content: Option<String>,
    pub model: Option<String>,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub key: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub sort_by: Option<String>,
    pub descending: Option<bool>,
}

/// 车辆 Repository
pub struct CarRepository {
    pool: PgPool,
}

impl CarRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 查询车辆列表（分页）
    pub async fn list(&self, query: &CarQuery) -> Result<PaginatedCars, CarRepositoryError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        // 状态过滤: "" 不限制, "N" 未放行, "Y" 已放行
        let status = match query.status.as_deref() {
            Some("未放行") => "N",
            Some("已放行") => "Y",
            _ => "",
        };

        // 日期范围过滤
        // mode: "" 不限制 / "both" (dc_date LIKE $3 OR rs_date LIKE $4) / "in" dc_date BETWEEN / "out" rs_date BETWEEN
        let (date_mode, d1, d2) = match (&query.in_date, &query.out_date) {
            (Some(in_date), Some(out_date))
                if in_date.contains(" - ") && out_date.contains(" - ") =>
            {
                let in_part = in_date.split(" - ").next().unwrap_or("");
                let out_part = out_date.split(" - ").next().unwrap_or("");
                ("both", format!("%{in_part}%"), format!("%{out_part}%"))
            }
            (Some(in_date), Some(_)) if in_date.contains(" - ") => {
                let dates: Vec<&str> = in_date.split(" - ").collect();
                if dates.len() == 2 {
                    ("in", dates[0].to_string(), dates[1].to_string())
                } else {
                    ("", String::new(), String::new())
                }
            }
            (Some(_), Some(out_date)) if out_date.contains(" - ") => {
                let dates: Vec<&str> = out_date.split(" - ").collect();
                if dates.len() == 2 {
                    ("out", dates[0].to_string(), dates[1].to_string())
                } else {
                    ("", String::new(), String::new())
                }
            }
            _ => ("", String::new(), String::new()),
        };

        // 内容搜索 (content 与 model 同时提供才生效; 列名白名单)
        let model_col = match query.model.as_deref() {
            Some("车牌") => "license",
            Some("车辆类型") => "car_type",
            Some("车身颜色") => "car_color",
            Some("车架号") => "vehicle",
            Some("发动机号") => "engine",
            Some("记录人") => "dc_name",
            Some("车辆备注") => "car_remark",
            Some("拖移备注") => "remark",
            _ => "license",
        };
        let content = if query.content.is_some() && query.model.is_some() {
            format!("%{}%", query.content.as_deref().unwrap_or(""))
        } else {
            String::new()
        };

        // 记录人过滤
        let name = query
            .name
            .as_deref()
            .map(|n| format!("%{n}%"))
            .unwrap_or_default();

        // 部门过滤: "城管部门" 特殊分支, "全部"/None 不限制
        let (unit, unit_like) = match query.unit.as_deref() {
            Some("城管部门") => ("城管部门", String::new()),
            Some(u) if u != "全部" => (u, format!("%{u}%")),
            _ => ("", String::new()),
        };

        // 编号过滤 (原逻辑: 精确匹配, 不加 %)
        let key = query.key.clone().unwrap_or_default();

        // 排序: 列名白名单 + 方向 (无 sort_by 时默认 dc_date DESC)
        let sort_key = match query.sort_by.as_deref() {
            Some("license") => "license",
            Some("car_type") => "car_type",
            Some("dc_date") => "dc_date",
            Some("dc_name") => "dc_name",
            Some("cmd_unit") => "cmd_unit",
            _ => "",
        };
        let descending = query.sort_by.is_none() || query.descending.unwrap_or(false);

        // 查询总数 (修复: 原 count SQL 拼接了 ORDER BY, 聚合查询会报 GroupingError, 已去掉)
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(id) FROM tow_car WHERE "delete" = false
                AND ($1 = '' OR ($1 = 'N' AND (rs_date IS NULL OR rs_date = '1000-01-01 00:00:00')) OR ($1 = 'Y' AND rs_date IS NOT NULL AND rs_date != '1000-01-01 00:00:00'))
                AND ($2 = '' OR ($2 = 'both' AND (dc_date LIKE $3 OR rs_date LIKE $4)) OR ($2 = 'in' AND dc_date BETWEEN $3 AND $4) OR ($2 = 'out' AND rs_date BETWEEN $3 AND $4))
                AND ($5 = '' OR ($6 = 'license' AND license LIKE $5) OR ($6 = 'car_type' AND car_type LIKE $5) OR ($6 = 'car_color' AND car_color LIKE $5) OR ($6 = 'vehicle' AND vehicle::text LIKE $5) OR ($6 = 'engine' AND engine LIKE $5) OR ($6 = 'dc_name' AND dc_name LIKE $5) OR ($6 = 'car_remark' AND car_remark LIKE $5) OR ($6 = 'remark' AND remark LIKE $5))
                AND ($7 = '' OR dc_name LIKE $7)
                AND ($8 = '' OR ($8 = '城管部门' AND cmd_unit IN ('城管部门','卧龙执法点','开化执法点','新平执法点')) OR ($9 <> '' AND cmd_unit LIKE $9))
                AND ($10 = '' OR dc_key LIKE $10)
                LIMIT 1"#,
            status,
            date_mode,
            &d1,
            &d2,
            &content,
            model_col,
            &name,
            unit,
            &unit_like,
            &key,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // 查询列表 (排序方向静态, 两个 query! 变体)
        let cars: Vec<CarListItem> = if descending {
            sqlx::query_as!(
                CarListItem,
                r#"SELECT id,
                    COALESCE(license, '') AS "license!",
                    vehicle,
                    COALESCE(engine, '') AS "engine!",
                    COALESCE(car_type, '') AS "car_type!",
                    COALESCE(dc_type, '') AS "dc_type!",
                    COALESCE(dc_causes, '') AS "dc_causes!",
                    COALESCE(car_color, '') AS "car_color!",
                    COALESCE(dc_date, '') AS "dc_date!",
                    COALESCE(dc_address, '') AS "dc_address!",
                    COALESCE(dc_key, '') AS "dc_key!",
                    COALESCE(dc_party_name, '') AS "dc_party_name!",
                    COALESCE(dc_party_cardid, '') AS "dc_party_cardid!",
                    COALESCE(dc_party_tel, '') AS "dc_party_tel!",
                    COALESCE(dc_name, '') AS "dc_name!",
                    COALESCE(cmd_unit, '') AS "cmd_unit!",
                    COALESCE(cv, '') AS "cv!",
                    COALESCE(tv, '') AS "tv!",
                    COALESCE(parking_unit, '') AS "parking_unit!",
                    COALESCE(remark, '') AS "remark!",
                    COALESCE(rs_name, '') AS "rs_name!",
                    COALESCE(rs_date, '') AS "rs_date!",
                    COALESCE("delete", false) AS "delete!"
                FROM tow_car
                WHERE "delete" = false
                    AND ($1 = '' OR ($1 = 'N' AND (rs_date IS NULL OR rs_date = '1000-01-01 00:00:00')) OR ($1 = 'Y' AND rs_date IS NOT NULL AND rs_date != '1000-01-01 00:00:00'))
                    AND ($2 = '' OR ($2 = 'both' AND (dc_date LIKE $3 OR rs_date LIKE $4)) OR ($2 = 'in' AND dc_date BETWEEN $3 AND $4) OR ($2 = 'out' AND rs_date BETWEEN $3 AND $4))
                    AND ($5 = '' OR ($6 = 'license' AND license LIKE $5) OR ($6 = 'car_type' AND car_type LIKE $5) OR ($6 = 'car_color' AND car_color LIKE $5) OR ($6 = 'vehicle' AND vehicle::text LIKE $5) OR ($6 = 'engine' AND engine LIKE $5) OR ($6 = 'dc_name' AND dc_name LIKE $5) OR ($6 = 'car_remark' AND car_remark LIKE $5) OR ($6 = 'remark' AND remark LIKE $5))
                    AND ($7 = '' OR dc_name LIKE $7)
                    AND ($8 = '' OR ($8 = '城管部门' AND cmd_unit IN ('城管部门','卧龙执法点','开化执法点','新平执法点')) OR ($9 <> '' AND cmd_unit LIKE $9))
                    AND ($10 = '' OR dc_key LIKE $10)
                ORDER BY CASE WHEN $11 = 'license' THEN license
                              WHEN $11 = 'car_type' THEN car_type
                              WHEN $11 = 'dc_name' THEN dc_name
                              WHEN $11 = 'cmd_unit' THEN cmd_unit
                              ELSE dc_date END DESC
                LIMIT $12 OFFSET $13"#,
                status,
                date_mode,
                &d1,
                &d2,
                &content,
                model_col,
                &name,
                unit,
                &unit_like,
                &key,
                sort_key,
                page_size as i64,
                offset as i64,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                CarListItem,
                r#"SELECT id,
                    COALESCE(license, '') AS "license!",
                    vehicle,
                    COALESCE(engine, '') AS "engine!",
                    COALESCE(car_type, '') AS "car_type!",
                    COALESCE(dc_type, '') AS "dc_type!",
                    COALESCE(dc_causes, '') AS "dc_causes!",
                    COALESCE(car_color, '') AS "car_color!",
                    COALESCE(dc_date, '') AS "dc_date!",
                    COALESCE(dc_address, '') AS "dc_address!",
                    COALESCE(dc_key, '') AS "dc_key!",
                    COALESCE(dc_party_name, '') AS "dc_party_name!",
                    COALESCE(dc_party_cardid, '') AS "dc_party_cardid!",
                    COALESCE(dc_party_tel, '') AS "dc_party_tel!",
                    COALESCE(dc_name, '') AS "dc_name!",
                    COALESCE(cmd_unit, '') AS "cmd_unit!",
                    COALESCE(cv, '') AS "cv!",
                    COALESCE(tv, '') AS "tv!",
                    COALESCE(parking_unit, '') AS "parking_unit!",
                    COALESCE(remark, '') AS "remark!",
                    COALESCE(rs_name, '') AS "rs_name!",
                    COALESCE(rs_date, '') AS "rs_date!",
                    COALESCE("delete", false) AS "delete!"
                FROM tow_car
                WHERE "delete" = false
                    AND ($1 = '' OR ($1 = 'N' AND (rs_date IS NULL OR rs_date = '1000-01-01 00:00:00')) OR ($1 = 'Y' AND rs_date IS NOT NULL AND rs_date != '1000-01-01 00:00:00'))
                    AND ($2 = '' OR ($2 = 'both' AND (dc_date LIKE $3 OR rs_date LIKE $4)) OR ($2 = 'in' AND dc_date BETWEEN $3 AND $4) OR ($2 = 'out' AND rs_date BETWEEN $3 AND $4))
                    AND ($5 = '' OR ($6 = 'license' AND license LIKE $5) OR ($6 = 'car_type' AND car_type LIKE $5) OR ($6 = 'car_color' AND car_color LIKE $5) OR ($6 = 'vehicle' AND vehicle::text LIKE $5) OR ($6 = 'engine' AND engine LIKE $5) OR ($6 = 'dc_name' AND dc_name LIKE $5) OR ($6 = 'car_remark' AND car_remark LIKE $5) OR ($6 = 'remark' AND remark LIKE $5))
                    AND ($7 = '' OR dc_name LIKE $7)
                    AND ($8 = '' OR ($8 = '城管部门' AND cmd_unit IN ('城管部门','卧龙执法点','开化执法点','新平执法点')) OR ($9 <> '' AND cmd_unit LIKE $9))
                    AND ($10 = '' OR dc_key LIKE $10)
                ORDER BY CASE WHEN $11 = 'license' THEN license
                              WHEN $11 = 'car_type' THEN car_type
                              WHEN $11 = 'dc_name' THEN dc_name
                              WHEN $11 = 'cmd_unit' THEN cmd_unit
                              ELSE dc_date END ASC
                LIMIT $12 OFFSET $13"#,
                status,
                date_mode,
                &d1,
                &d2,
                &content,
                model_col,
                &name,
                unit,
                &unit_like,
                &key,
                sort_key,
                page_size as i64,
                offset as i64,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(PaginatedCars {
            cars,
            total,
            page,
            page_size,
        })
    }

    /// 统计车辆数量
    pub async fn count(&self, query: &CarQuery) -> Result<i64, CarRepositoryError> {
        // 状态过滤: "" 不限制, "N" 未放行, "Y" 已放行
        let status = match query.status.as_deref() {
            Some("未放行") => "N",
            Some("已放行") => "Y",
            _ => "",
        };

        // 日期范围过滤 (仅原逻辑的 "both" 分支)
        let (date_mode, d1, d2) = match (&query.in_date, &query.out_date) {
            (Some(in_date), Some(out_date))
                if in_date.contains(" - ") && out_date.contains(" - ") =>
            {
                let in_part = in_date.split(" - ").next().unwrap_or("");
                let out_part = out_date.split(" - ").next().unwrap_or("");
                ("both", format!("%{in_part}%"), format!("%{out_part}%"))
            }
            _ => ("", String::new(), String::new()),
        };

        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(id) FROM tow_car WHERE "delete" = false
                AND ($1 = '' OR ($1 = 'N' AND (rs_date IS NULL OR rs_date = '1000-01-01 00:00:00')) OR ($1 = 'Y' AND rs_date IS NOT NULL AND rs_date != '1000-01-01 00:00:00'))
                AND ($2 = '' OR (dc_date LIKE $3 OR rs_date LIKE $4))
                LIMIT 1"#,
            status,
            date_mode,
            &d1,
            &d2,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    /// 根据车牌号查询车辆
    pub async fn find_by_license(&self, license: &str) -> Result<Option<Car>, CarRepositoryError> {
        let car: Option<Car> = sqlx::query_as!(
            Car,
            r#"SELECT id,
                COALESCE(license, '') AS "license!",
                vehicle,
                COALESCE(engine, '') AS "engine!",
                COALESCE(car_type, '') AS "car_type!",
                COALESCE(dc_type, '') AS "dc_type!",
                COALESCE(dc_causes, '') AS "dc_causes!",
                COALESCE(car_color, '') AS "car_color!",
                COALESCE(dc_date, '') AS "dc_date!",
                COALESCE(dc_address, '') AS "dc_address!",
                COALESCE(dc_key, '') AS "dc_key!",
                COALESCE(dc_party_name, '') AS "dc_party_name!",
                COALESCE(dc_party_cardid, '') AS "dc_party_cardid!",
                COALESCE(dc_party_tel, '') AS "dc_party_tel!",
                COALESCE(p_name, '') AS "p_name!",
                COALESCE(p_id, '') AS "p_id!",
                COALESCE(dc_acc, '') AS "dc_acc!",
                COALESCE(dc_name, '') AS "dc_name!",
                COALESCE(dc_into_date, '') AS "dc_into_date!",
                COALESCE(car_remark, '') AS "car_remark!",
                COALESCE(driver, '') AS "driver!",
                COALESCE(operator, '') AS "operator!",
                COALESCE(drag_km, '') AS "drag_km!",
                COALESCE(drag_unit, '') AS "drag_unit!",
                COALESCE(drag_money, '') AS "drag_money!",
                COALESCE(cmd_unit, '') AS "cmd_unit!",
                COALESCE(cmd_user, '') AS "cmd_user!",
                COALESCE(cv, '') AS "cv!",
                COALESCE(cv_acc, '') AS "cv_acc!",
                COALESCE(cv_name, '') AS "cv_name!",
                COALESCE(cv_date, '') AS "cv_date!",
                COALESCE(cv_opinion, '') AS "cv_opinion!",
                COALESCE(tv, '') AS "tv!",
                COALESCE(tv_acc, '') AS "tv_acc!",
                COALESCE(tv_name, '') AS "tv_name!",
                COALESCE(tv_date, '') AS "tv_date!",
                COALESCE(tv_opinion, '') AS "tv_opinion!",
                COALESCE(rc_name, '') AS "rc_name!",
                COALESCE(rc_idcard, '') AS "rc_idcard!",
                COALESCE(rc_tel, '') AS "rc_tel!",
                COALESCE(parking_date, '') AS "parking_date!",
                COALESCE(parking_unit, '') AS "parking_unit!",
                COALESCE(parking_money, '') AS "parking_money!",
                COALESCE(parking_payable, '') AS "parking_payable!",
                COALESCE(parking_paidin, '') AS "parking_paidin!",
                COALESCE(remark, '') AS "remark!",
                COALESCE(rs_name, '') AS "rs_name!",
                COALESCE(rs_acc, '') AS "rs_acc!",
                COALESCE(rs_date, '') AS "rs_date!",
                attachment,
                COALESCE(create_date::timestamp, '1970-01-01 00:00:00') AS "create_date!",
                COALESCE(create_user, '') AS "create_user!",
                COALESCE(update_date::timestamp, '1970-01-01 00:00:00') AS "update_date!",
                COALESCE(update_user, '') AS "update_user!",
                COALESCE("delete", false) AS "delete!"
            FROM tow_car
            WHERE license LIKE $1 AND "delete" = false"#,
            format!("%{license}%"),
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(car)
    }

    /// 根据 ID 查询车辆
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Car>, CarRepositoryError> {
        let car: Option<Car> = sqlx::query_as!(
            Car,
            r#"SELECT id,
                COALESCE(license, '') AS "license!",
                vehicle,
                COALESCE(engine, '') AS "engine!",
                COALESCE(car_type, '') AS "car_type!",
                COALESCE(dc_type, '') AS "dc_type!",
                COALESCE(dc_causes, '') AS "dc_causes!",
                COALESCE(car_color, '') AS "car_color!",
                COALESCE(dc_date, '') AS "dc_date!",
                COALESCE(dc_address, '') AS "dc_address!",
                COALESCE(dc_key, '') AS "dc_key!",
                COALESCE(dc_party_name, '') AS "dc_party_name!",
                COALESCE(dc_party_cardid, '') AS "dc_party_cardid!",
                COALESCE(dc_party_tel, '') AS "dc_party_tel!",
                COALESCE(p_name, '') AS "p_name!",
                COALESCE(p_id, '') AS "p_id!",
                COALESCE(dc_acc, '') AS "dc_acc!",
                COALESCE(dc_name, '') AS "dc_name!",
                COALESCE(dc_into_date, '') AS "dc_into_date!",
                COALESCE(car_remark, '') AS "car_remark!",
                COALESCE(driver, '') AS "driver!",
                COALESCE(operator, '') AS "operator!",
                COALESCE(drag_km, '') AS "drag_km!",
                COALESCE(drag_unit, '') AS "drag_unit!",
                COALESCE(drag_money, '') AS "drag_money!",
                COALESCE(cmd_unit, '') AS "cmd_unit!",
                COALESCE(cmd_user, '') AS "cmd_user!",
                COALESCE(cv, '') AS "cv!",
                COALESCE(cv_acc, '') AS "cv_acc!",
                COALESCE(cv_name, '') AS "cv_name!",
                COALESCE(cv_date, '') AS "cv_date!",
                COALESCE(cv_opinion, '') AS "cv_opinion!",
                COALESCE(tv, '') AS "tv!",
                COALESCE(tv_acc, '') AS "tv_acc!",
                COALESCE(tv_name, '') AS "tv_name!",
                COALESCE(tv_date, '') AS "tv_date!",
                COALESCE(tv_opinion, '') AS "tv_opinion!",
                COALESCE(rc_name, '') AS "rc_name!",
                COALESCE(rc_idcard, '') AS "rc_idcard!",
                COALESCE(rc_tel, '') AS "rc_tel!",
                COALESCE(parking_date, '') AS "parking_date!",
                COALESCE(parking_unit, '') AS "parking_unit!",
                COALESCE(parking_money, '') AS "parking_money!",
                COALESCE(parking_payable, '') AS "parking_payable!",
                COALESCE(parking_paidin, '') AS "parking_paidin!",
                COALESCE(remark, '') AS "remark!",
                COALESCE(rs_name, '') AS "rs_name!",
                COALESCE(rs_acc, '') AS "rs_acc!",
                COALESCE(rs_date, '') AS "rs_date!",
                attachment,
                COALESCE(create_date::timestamp, '1970-01-01 00:00:00') AS "create_date!",
                COALESCE(create_user, '') AS "create_user!",
                COALESCE(update_date::timestamp, '1970-01-01 00:00:00') AS "update_date!",
                COALESCE(update_user, '') AS "update_user!",
                COALESCE("delete", false) AS "delete!"
            FROM tow_car
            WHERE id = $1 AND "delete" = false"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(car)
    }
}
