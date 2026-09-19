//! 订单管理路由
//!
//! 支持订单增删、批量导入和多维度条件查询。

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use auth_core::AuthenticatedUser;
use chrono::NaiveDateTime;
use serde_json::{json, Value};

use crate::model::OrderInfo;
use crate::AppState;

/// 订单管理统一入口（根据 `method` 字段分发）
///
/// 支持的操作：`query`, `batch_add`, `add`
pub async fn order(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    AxumJson(req): AxumJson<Value>,
) -> Response {
    let method = req["method" ].as_str().unwrap_or("" );
    let user_role = user.0.role.as_str();

    match method {
        "query" => {
            let code = req["code" ].as_str().unwrap_or("" );
            // 运营方只能查自己，admin 可查所有
            let provide = if user_role == "admin" {
                req["provide" ].as_str().unwrap_or("" )
            } else {
                user_role
            };
            let status = req["status" ].as_i64().unwrap_or(-1);
            let time_start = req["time" ]["start" ].as_str().unwrap_or("" );
            let time_end = req["time" ]["end" ].as_str().unwrap_or("" );
            let order = req["order" ].as_str().unwrap_or("" );
            let paystatus = req["paystatus" ].as_i64().unwrap_or(-1);
            let paytype = req["paytype" ].as_i64().unwrap_or(-1);
            let paytime = req["paytime" ].as_str().unwrap_or("" );
            // 分页参数：默认每页 50 条，最大 500 条
            let limit = req["limit" ].as_i64().unwrap_or(50).clamp(1, 500);
            let offset = req["offset" ].as_i64().unwrap_or(0).max(0);

            match state
                .order_repo
                .query(OrderQueryParams {
                    code,
                    provide,
                    status,
                    order,
                    paystatus,
                    paytype,
                    paytime,
                    limit,
                    offset,
                })
                .await
            {
                Ok(orders) => common::ok_response(orders).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "batch_add" => {
            const MAX_BATCH: usize = 500;
            if let Some(items) = req["items" ].as_array() {
                if items.len() > MAX_BATCH {
                    return common::bad_request_response(
                        &format!("batch size exceeded: max {MAX_BATCH} items per request" )
                    ).into_response();
                }
                for item in items {
                    let p = item["provide" ].as_str().unwrap_or("" );
                    if user_role != "admin" && user_role != p {
                        return common::unauthorized_response("provide mismatch in batch" ).into_response();
                    }
                }
                let orders: Vec<OrderInfo> = items.iter().map(|item| {
                    let pay_time_str = item["pay_time" ].as_str().unwrap_or("2000-01-01 00:00:00" );
                    let pay_time = NaiveDateTime::parse_from_str(pay_time_str, "%Y-%m-%d %H:%M:%S" )
                        .unwrap_or_default();

                    OrderInfo {
                        code: Some(item["code" ].as_str().unwrap_or("" ).to_string()),
                        status: item["status" ].as_i64().unwrap_or(0),
                        provide: item["provide" ].as_str().unwrap_or("" ).to_string(),
                        speed: item["speed" ].as_f64().unwrap_or(0.0),
                        gps: Some(json!({
                            "lng": item["gps" ]["lng" ].as_str().unwrap_or("" ),
                            "lat": item["gps" ]["lat" ].as_str().unwrap_or("" ),
                        })),
                        r#type: 0,
                        time: Some(json!({
                            "start": item["time" ]["start" ].as_str().unwrap_or("" ).to_string(),
                            "end": item["time" ]["end" ].as_str().unwrap_or("" ).to_string(),
                        })),
                        create_date: Some(chrono::Local::now().naive_local()),
                        update_date: Some(chrono::Local::now().naive_local()),
                        delete: Some(false),
                        alert: Some(item["alert" ].as_str().unwrap_or("" ).to_string()),
                        remark: Some(item["remark" ].as_str().unwrap_or("" ).to_string()),
                        hash: String::new(),
                        payable: item["payable" ].as_i64().unwrap_or(0),
                        pay: item["pay" ].as_i64().unwrap_or(0),
                        refund: item["refund" ].as_f64().unwrap_or(0.0),
                        coupon: item["coupon" ].as_f64().unwrap_or(0.0),
                        order: Some(item["order" ].as_str().unwrap_or("" ).to_string()),
                        pay_type: item["pay_type" ].as_i64().unwrap_or(0),
                        pay_time: Some(pay_time),
                        pay_status: item["pay_status" ].as_i64().unwrap_or(0),
                        paytype: item["paytype" ].as_i64().unwrap_or(0),
                        paytime: Some(pay_time),
                        gps_type: Some(item["gps_type" ].as_i64().unwrap_or(0)),
                    }
                }).collect();

                match state.order_repo.add_batch(&orders).await {
                    Ok(_) => common::success_with_message_response("success" ).into_response(),
                    Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
                }
            } else {
                common::bad_request_response("invalid items array" ).into_response()
            }
        }
        "add" => {
            let provide = req["provide" ].as_str().unwrap_or("" );
            if user_role != "admin" && user_role != provide {
                return common::unauthorized_response("provide mismatch" ).into_response();
            }
            let code = req["code" ].as_str().unwrap_or("" );
            let status = req["status" ].as_i64().unwrap_or(0);
            let _speed = req["speed" ].as_f64().unwrap_or(0.0);
            let speed = req["speed" ].as_f64().unwrap_or(0.0);
            let gps_type = req["gps_type" ].as_i64().unwrap_or(0);
            let payable = req["payable" ].as_i64().unwrap_or(0);
            let pay = req["pay" ].as_i64().unwrap_or(0);
            let refund = req["refund" ].as_f64().unwrap_or(0.0);
            let coupon = req["coupon" ].as_f64().unwrap_or(0.0);
            let order = req["order" ].as_str().unwrap_or("" );
            let pay_type = req["pay_type" ].as_i64().unwrap_or(0);
            let pay_status = req["pay_status" ].as_i64().unwrap_or(0);
            let alert = req["alert" ].as_str().unwrap_or("" );
            let remark = req["remark" ].as_str().unwrap_or("" );

            let pay_time_str = req["pay_time" ].as_str().unwrap_or("2000-01-01 00:00:00" );
            let pay_time = NaiveDateTime::parse_from_str(pay_time_str, "%Y-%m-%d %H:%M:%S" )
                .unwrap_or_default();

            let info = OrderInfo {
                code: Some(code.to_string()),
                status,
                provide: provide.to_string(),
                speed,
                gps: Some(json!({
                    "lng": req["gps" ]["lng" ].as_str().unwrap_or("" ),
                    "lat": req["gps" ]["lat" ].as_str().unwrap_or("" ),
                })),
                r#type: 0,
                time: Some(json!({
                    "start": req["time" ]["start" ].as_str().unwrap_or("" ).to_string(),
                    "end": req["time" ]["end" ].as_str().unwrap_or("" ).to_string(),
                })),
                create_date: Some(chrono::Local::now().naive_local()),
                update_date: Some(chrono::Local::now().naive_local()),
                delete: Some(false),
                alert: Some(alert.to_string()),
                remark: Some(remark.to_string()),
                hash: String::new(),
                payable,
                pay,
                refund,
                coupon,
                order: Some(order.to_string()),
                pay_type,
                pay_time: Some(pay_time),
                pay_status,
                paytype: pay_type,
                paytime: Some(pay_time),
                gps_type: Some(gps_type),
            };

            match state.order_repo.add(&info).await {
                Ok(hash) => common::ok_response(json!({"hash": hash})).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        _ => common::bad_request_response("unknown method" ).into_response(),
    }
}
