//! 车牌识别通行业务逻辑服务
//!
//! 完整处理流程：记录通行→查车辆授权→入场开闸/出场计费→更新状态

use redis::aio::ConnectionManager;
use reqwest::Client as HttpClient;
use sqlx::PgPool;

use common::AppError;
use common::AppResult;
use crate::models::{
    CreatePassRecord, GateControlResult, PassProcessResult, PassRecord, VehicleAuthorization,
};

/// 车牌通行业务服务
pub struct PassService {
    db_pool: PgPool,
    redis_conn: Option<ConnectionManager>,
    http_client: HttpClient,
    hik_service_url: String,
}

impl PassService {
    #[must_use]
    pub fn new(db_pool: PgPool, redis_conn: Option<ConnectionManager>) -> Self {
        Self {
            db_pool,
            redis_conn,
            http_client: HttpClient::new(),
            hik_service_url: std::env::var("HIK_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8092".to_string()),
        }
    }

    // ==================== 核心入口 ====================

    /// 处理车牌识别回调（完整业务流）
    pub async fn process_callback(&self, req: &CreatePassRecord) -> AppResult<PassProcessResult> {
        let record = self.record_pass(req).await?;
        let auth = self.check_vehicle_authorization(&req.plate_no, &req.park_code).await?;

        let gate_opened = match req.direction.as_str() {
            "entry" => self.handle_entry(&record, &auth).await?,
            "exit" => self.handle_exit(&record, &auth).await?,
            _ => false,
        };

        self.update_record_status(record.id, "processed", "").await?;

        Ok(PassProcessResult {
            record_id: record.id,
            plate_no: req.plate_no.clone(),
            direction: req.direction.clone(),
            is_authorized: auth.is_authorized,
            gate_opened,
            auth_type: auth.auth_type.clone(),
            driver_name: auth.driver_name.clone(),
        })
    }

    // ==================== 数据库操作 ====================

    pub async fn record_pass(&self, req: &CreatePassRecord) -> AppResult<PassRecord> {
        let row = sqlx::query_as!(
            PassRecord,
            r#"
            INSERT INTO public.lpr_pass_records
                (plate_no, plate_color, plate_type, vehicle_type, device_id, device_name,
                 park_code, lane_code, direction, pass_time, image_url, confidence)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, plate_no, plate_color, plate_type, vehicle_type, device_id, device_name,
                      park_code, lane_code, direction, pass_time, image_url, confidence,
                      status, related_order_id, remark, created_at, updated_at
            "#,
            &req.plate_no,
            &req.plate_color,
            &req.plate_type,
            &req.vehicle_type,
            &req.device_id,
            &req.device_name,
            &req.park_code,
            &req.lane_code,
            &req.direction,
            req.pass_time,
            &req.image_url,
            req.confidence,
        )
        .fetch_one(&self.db_pool)
        .await?;

        tracing::info!("通行记录已保存 id={}, 车牌={}, 方向={}", row.id, row.plate_no, row.direction);
        Ok(row)
    }

    pub async fn update_record_status(&self, record_id: i64, status: &str, remark: &str) -> AppResult<()> {
        sqlx::query!(
            r#"UPDATE public.lpr_pass_records SET status = $1, remark = $2, updated_at = NOW() WHERE id = $3"#,
            status,
            remark,
            record_id,
        )
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }

    pub async fn find_latest_entry(&self, plate_no: &str, park_code: &str) -> AppResult<Option<PassRecord>> {
        let row = sqlx::query_as!(
            PassRecord,
            r#"
            SELECT id, plate_no, plate_color, plate_type, vehicle_type, device_id, device_name,
                   park_code, lane_code, direction, pass_time, image_url, confidence,
                   status, related_order_id, remark, created_at, updated_at
            FROM public.lpr_pass_records
            WHERE plate_no = $1 AND park_code = $2 AND direction = 'entry'
            ORDER BY pass_time DESC LIMIT 1
            "#,
            plate_no,
            park_code,
        )
        .fetch_optional(&self.db_pool)
        .await?;
        Ok(row)
    }

    // ==================== hik-service 集成 ====================

    async fn check_vehicle_authorization(&self, plate_no: &str, _park_code: &str) -> AppResult<VehicleAuthorization> {
        // Redis 缓存
        if let Some(ref conn) = self.redis_conn {
            let cache_key = format!("lpr:vehicle:auth:{plate_no}");
            let mut conn = conn.clone();
            if let Ok(Some(cached)) = redis::cmd("GET")
                .arg(&cache_key)
                .query_async::<Option<String>>(&mut conn)
                .await
                && let Ok(auth) = serde_json::from_str::<VehicleAuthorization>(&cached) {
                    tracing::debug!("车辆授权缓存命中: {} => {:?}", plate_no, auth.auth_type);
                    return Ok(auth);
                }
        }

        // 调用 hik-service
        let payload = serde_json::json!({
            "method": "parkParking",
            "plate_no": plate_no,
            "phone": "", "pageNo": 1, "pageSize": 1, "requestType": ""
        });

        let auth = match self.http_client
            .post(format!("{}/api/hik/exec", self.hik_service_url))
            .json(&payload).send().await
        {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                Self::parse_hik_response(&body, plate_no)
            }
            Ok(resp) => {
                tracing::warn!("hik-service 查询失败: HTTP {}, 车牌={}", resp.status(), plate_no);
                VehicleAuthorization { is_authorized: false, auth_type: "temp".to_string(), driver_name: String::new(), driver_phone: String::new(), valid_until: None }
            }
            Err(e) => {
                tracing::error!("hik-service 请求失败: {e}, 车牌={plate_no}");
                VehicleAuthorization { is_authorized: false, auth_type: "temp".to_string(), driver_name: String::new(), driver_phone: String::new(), valid_until: None }
            }
        };

        // 写 Redis 缓存
        if let Some(ref conn) = self.redis_conn {
            let cache_key = format!("lpr:vehicle:auth:{plate_no}");
            if let Ok(json) = serde_json::to_string(&auth) {
                let mut conn = conn.clone();
                let _: std::result::Result<(), redis::RedisError> = redis::cmd("SETEX")
                    .arg(&cache_key).arg(300usize).arg(&json)
                    .query_async(&mut conn).await;
            }
        }
        Ok(auth)
    }

    fn parse_hik_response(body: &serde_json::Value, _plate_no: &str) -> VehicleAuthorization {
        if let Some(data) = body.get("data")
            && let Some(result) = data.get("result")
                && let Some(orders) = result.as_array()
                    && let Some(order) = orders.first() {
                        let is_monthly = order.get("orderType")
                            .and_then(|v| v.as_str())
                            .is_some_and(|t| t == "1" || t == "3");
                        if is_monthly {
                            return VehicleAuthorization {
                                is_authorized: true,
                                auth_type: "monthly_vip".to_string(),
                                driver_name: order.get("driverName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                driver_phone: order.get("phone").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                valid_until: order.get("validDate").and_then(|v| v.as_str()).map(std::string::ToString::to_string),
                            };
                        }
                    }
        VehicleAuthorization { is_authorized: false, auth_type: "temp".to_string(), driver_name: String::new(), driver_phone: String::new(), valid_until: None }
    }

    // ==================== 方向处理 ====================

    async fn handle_entry(&self, record: &PassRecord, auth: &VehicleAuthorization) -> AppResult<bool> {
        if auth.is_authorized {
            tracing::info!("授权车辆入场: 车牌={}, 类型={}", record.plate_no, auth.auth_type);
            match self.open_gate(&record.park_code, &record.device_id).await {
                Ok(r) => Ok(r.success),
                Err(e) => { tracing::error!("开闸失败: {}, 车牌={}", e, record.plate_no); Ok(false) }
            }
        } else {
            tracing::info!("临时车辆入场: 车牌={}, 记录通行不开闸", record.plate_no);
            Ok(false)
        }
    }

    async fn handle_exit(&self, record: &PassRecord, auth: &VehicleAuthorization) -> AppResult<bool> {
        let entry = self.find_latest_entry(&record.plate_no, &record.park_code).await?;
        if let Some(entry_record) = entry {
            let minutes = record.pass_time.signed_duration_since(entry_record.pass_time).num_minutes();
            tracing::info!("车辆出场: 车牌={}, 停车{}分钟", record.plate_no, minutes);
            if auth.is_authorized || minutes <= 15 {
                return self.open_gate(&record.park_code, &record.device_id).await.map(|r| r.success)
                    .map_err(|e| AppError::LprInternal(e.to_string()));
            }
            tracing::info!("临时车辆需缴费: 车牌={}, 停车{}分钟", record.plate_no, minutes);
            Ok(false)
        } else {
            tracing::warn!("出场未找到入场记录: 车牌={}, 直接开闸放行", record.plate_no);
            self.open_gate(&record.park_code, &record.device_id).await.map(|r| r.success)
                .map_err(|e| AppError::LprInternal(e.to_string()))
        }
    }

    // ==================== Signo 开闸 ====================

    async fn open_gate(&self, park_code: &str, device_id: &str) -> AppResult<GateControlResult> {
        let payload = serde_json::json!({ "park_code": park_code, "device_id": device_id });
        match self.http_client.post(format!("{}/api/hik/signo/open", self.hik_service_url))
            .json(&payload).send().await
        {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let success = body.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);
                let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                Ok(GateControlResult { success, message })
            }
            Ok(resp) => {
                tracing::error!("Signo 开闸 HTTP 失败: {}, park_code={}", resp.status(), park_code);
                Ok(GateControlResult { success: false, message: format!("HTTP {}", resp.status()) })
            }
            Err(e) => {
                tracing::error!("Signo 开闸请求失败: {e}, park_code={park_code}");
                Ok(GateControlResult { success: false, message: e.to_string() })
            }
        }
    }

    // ==================== 查询方法 ====================

    /// 查询通行记录列表
    pub async fn list_records(&self, plate_no: &str, park_code: &str, direction: &str, status: &str, page: i32, page_size: i32) -> AppResult<Vec<PassRecord>> {
        let rows = sqlx::query_as!(
            PassRecord,
            r#"SELECT id, plate_no, plate_color, plate_type, vehicle_type, device_id, device_name,
                      park_code, lane_code, direction, pass_time, image_url, confidence,
                      status, related_order_id, remark, created_at, updated_at
               FROM public.lpr_pass_records
               WHERE ($1 = '' OR plate_no ILIKE '%' || $1 || '%')
                 AND ($2 = '' OR park_code = $2)
                 AND ($3 = '' OR direction = $3)
                 AND ($4 = '' OR status = $4)
               ORDER BY pass_time DESC
               LIMIT $5 OFFSET $6"#,
            plate_no,
            park_code,
            direction,
            status,
            i64::from(page_size),
            i64::from((page - 1) * page_size),
        )
        .fetch_all(&self.db_pool).await?;
        Ok(rows)
    }

    /// 获取单个通行记录
    pub async fn get_record(&self, id: i64) -> AppResult<PassRecord> {
        let row = sqlx::query_as!(
            PassRecord,
            r#"SELECT id, plate_no, plate_color, plate_type, vehicle_type, device_id, device_name,
                      park_code, lane_code, direction, pass_time, image_url, confidence,
                      status, related_order_id, remark, created_at, updated_at
               FROM public.lpr_pass_records WHERE id = $1"#,
            id,
        )
        .fetch_one(&self.db_pool).await?;
        Ok(row)
    }

    /// 获取通行统计
    pub async fn get_stats(&self) -> AppResult<crate::models::PassStats> {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "total!",
                      COALESCE(SUM(CASE WHEN direction = 'entry' THEN 1 ELSE 0 END), 0) AS "entry!",
                      COALESCE(SUM(CASE WHEN direction = 'exit' THEN 1 ELSE 0 END), 0) AS "exit!",
                      COALESCE(SUM(CASE WHEN confidence >= 0.9 THEN 1 ELSE 0 END), 0) AS "high_conf!"
               FROM public.lpr_pass_records"#,
        )
        .fetch_one(&self.db_pool).await?;
        Ok(crate::models::PassStats {
            total_pass: row.total,
            total_entry: row.entry,
            total_exit: row.exit,
            high_confidence: row.high_conf,
            park_code: String::new(),
        })
    }

    /// 获取车辆授权信息（复用授权查询逻辑）
    pub async fn get_vehicle_auth(&self, plate_no: &str, park_code: &str) -> AppResult<VehicleAuthorization> {
        self.check_vehicle_authorization(plate_no, park_code).await
    }
}
