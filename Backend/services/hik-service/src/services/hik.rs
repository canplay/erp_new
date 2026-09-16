// 海康API服务
// Hikvision API service

use redis::AsyncCommands;
use serde_json::json;

use common::AppError;
use common::AppResult;
use crate::models::HikRequest;

/// 海康服务
pub struct HikService {
    /// API基础URL
    base_url: String,
    /// Client ID
    client_id: String,
    /// Client Secret
    client_secret: String,
    /// Redis连接
    redis_url: String,
}

impl HikService {
    /// 创建海康服务实例
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("HIK_URL" ).unwrap_or_default(),
            client_id: std::env::var("HIK_CLIENT_ID" ).unwrap_or_default(),
            client_secret: std::env::var("HIK_CLIENT_SECRET" ).unwrap_or_default(),
            redis_url: std::env::var("REDIS_URL" ).unwrap_or_default(),
        }
    }

    /// 获取访问令牌
    pub async fn get_token(&self) -> AppResult<String> {
        let client = redis::Client::open(self.redis_url.as_str()).map_err(AppError::Redis)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(AppError::Redis)?;

        // 尝试从Redis获取缓存的token
        let cached_token: Option<String> =
            conn.get("hik_token" ).await.map_err(AppError::Redis)?;

        if let Some(token) = cached_token
            && !token.is_empty() {
                return Ok(token);
            }

        // 获取新token
        let params = [
            ("client_id" , self.client_id.as_str()),
            ("client_secret" , self.client_secret.as_str()),
        ];

        let client = reqwest::Client::new();
        let url = format!("{}/artemis/oauth/token" , self.base_url);

        let resp = client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(AppError::HttpError)?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.map_err(AppError::HttpError)?;

            if let Some(access_token) = body["access_token" ].as_str() {
                let expires_in = body["expires_in" ].as_i64().unwrap_or(3600);

                // 缓存token
                let _: () = conn
                    .set_ex("hik_token" , access_token, expires_in as u64)
                    .await
                    .map_err(AppError::Redis)?;

                return Ok(access_token.to_string());
            }
        }

        Err(AppError::HikTokenError)
    }

    /// 执行海康API调用
    pub async fn exec(&self, request: &HikRequest) -> AppResult<serde_json::Value> {
        let token = self.get_token().await?;
        let client = reqwest::Client::new();

        let (path, body) = match request.method.as_str() {
            // 司机相关
            "driver" => (
                "/artemis/api/v1/driver/driverInfo".to_string(),
                json!({ "phone": request.phone }),
            ),
            "plates" => (
                "/artemis/api/v1/driver/plates".to_string(),
                json!({ "driverId": request.driver_id }),
            ),
            "register" => (
                "/artemis/api/v1/driver/register".to_string(),
                json!({ "phone": request.phone }),
            ),
            "addPlate" => (
                "/artemis/api/v1/driver/addPlate".to_string(),
                json!({
                    "driverId": request.driver_id,
                    "plateNo": request.plate_no,
                    "plateColor": request.plate_color
                }),
            ),
            "delPlate" => (
                "/artemis/api/v1/driver/delPlate".to_string(),
                json!({
                    "driverId": request.driver_id,
                    "plateNo": request.plate_no,
                    "plateColor": request.plate_color
                }),
            ),
            // 停车场订单相关
            "parkOrders" => (
                "/artemis/api/v1/charge/parkOrders".to_string(),
                json!({
                    "plateNo": request.plate_no,
                    "phone": request.phone,
                    "pageNo": request.page_no,
                    "pageSize": request.page_size,
                    "requestType": request.request_type
                }),
            ),
            "parkParking" => (
                "/artemis/api/v1/charge/order/parking".to_string(),
                json!({ "plateNo": request.plate_no }),
            ),
            "parkArrears" => (
                "/artemis/api/v1/charge/getArrearsRecords".to_string(),
                json!({ "plateNo": request.plate_no }),
            ),
            "parkOrdersDetail" => (
                "/artemis/api/v1/charge/parkOrders/detail".to_string(),
                json!({
                    "phone": request.phone,
                    "uniqueId": request.unique_id,
                    "parkCode": request.park_code,
                    "isUseCoupon": request.is_use_coupon
                }),
            ),
            "report" => (
                "/artemis/api/v1/charge/parkOrders/appeal".to_string(),
                json!({
                    "phone": request.phone,
                    "uniqueId": request.unique_id,
                    "parkCode": request.park_code,
                    "appealType": request.appeal_type,
                    "appealRemark": request.appeal_remark,
                    "appealInTime": request.appeal_in_time,
                    "appealOutTime": request.appeal_out_time,
                    "appealSource": request.appeal_source
                }),
            ),
            "payBatch" => (
                "/artemis/api/v1/charge/payment/batch".to_string(),
                json!({
                    "arrearsIds": request.arrears_ids,
                    "payType": 5,
                    "payChannel": 4,
                    "phone": request.phone
                }),
            ),
            // 停车场信息
            "parkinfos" => (
                "/artemis/api/v1/dealer/parkinfos".to_string(),
                json!({
                    "pageNo": request.page_no.unwrap_or(1),
                    "pageSize": request.page_size.unwrap_or(50),
                    "parkName": request.park_name
                }),
            ),
            "parkinfo" => (
                format!(
                    "/artemis/api/v1/dealer/parkinfo/{}" ,
                    request.park_code.as_ref().unwrap_or(&String::new())
                ),
                json!({}),
            ),
            _ => return Err(AppError::HikInvalidMethod),
        };

        let url = format!("{}{}" , self.base_url, path);

        let resp = client
            .post(&url)
            .header("access_token" , &token)
            .header("Content-Type" , "application/json" )
            .json(&body)
            .send()
            .await
            .map_err(AppError::HttpError)?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.map_err(AppError::HttpError)?;
            Ok(body)
        } else {
            Err(AppError::HikApiError(format!("HTTP {}" , resp.status())))
        }
    }

    /// 发送优惠券
    pub async fn send_coupon(
        &self,
        phone: &str,
        amount: i32,
        start: i64,
        end: &str,
    ) -> AppResult<serde_json::Value> {
        let token = self.get_token().await?;
        let client = reqwest::Client::new();

        // 先获取停车场列表
        let url = format!("{}/artemis/api/v1/dealer/parkinfos" , self.base_url);
        let body = json!({
            "pageNo": 1,
            "pageSize": 50
        });

        let resp = client
            .post(&url)
            .header("access_token" , &token)
            .header("Content-Type" , "application/json" )
            .json(&body)
            .send()
            .await
            .map_err(AppError::HttpError)?;

        if !resp.status().is_success() {
            return Err(AppError::HikApiError("Failed to get park infos".to_string()));
        }

        let park_data: serde_json::Value = resp.json().await.map_err(AppError::HttpError)?;

        let results = park_data["data" ]["results" ]
            .as_array()
            .ok_or_else(|| AppError::HikApiError("Invalid park data".to_string()))?;

        // 构建停车场编码列表
        let park_codes: Vec<String> = results
            .iter()
            .filter_map(|p| p["parkCode" ].as_str().map(String::from))
            .collect();
        let park_codes_str = park_codes.join("," );

        // 发送优惠券
        let url = format!("{}/artemis/api/v1/sendCoupon" , self.base_url);
        let coupon_body = json!({
            "parkCodes": park_codes_str,
            "generateObj": 2,
            "telephone": phone,
            "couponType": 1,
            "deductContent": amount,
            "startTime": start,
            "endTime": end,
            "couponSource": "1001"
        });

        let resp = client
            .post(&url)
            .header("access_token" , &token)
            .header("Content-Type" , "application/json" )
            .json(&coupon_body)
            .send()
            .await
            .map_err(AppError::HttpError)?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.map_err(AppError::HttpError)?;
            Ok(body)
        } else {
            Err(AppError::HikApiError("Failed to send coupon".to_string()))
        }
    }
}

impl Default for HikService {
    fn default() -> Self {
        Self::new()
    }
}
