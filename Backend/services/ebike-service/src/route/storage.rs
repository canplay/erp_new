//! 仓储管理路由
//!
//! 支持仓储增删、批量导入和多维度查询。

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use auth_core::AuthenticatedUser;
use serde_json::{json, Value};

use crate::model::StorageInfo;
use crate::AppState;

/// 仓储管理统一入口（根据 `method` 字段分发）
///
/// 支持的操作：`add`, `batch_add`, `delete`, `query`, `history`
pub async fn storage(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    AxumJson(req): AxumJson<Value>,
) -> Response {
    let method = req["method" ].as_str().unwrap_or("" );
    let user_role = user.0.role.as_str();

    match method {
        "add" => {
            let provide = req["provide" ].as_str().unwrap_or("" );
            if user_role != "admin" && user_role != provide {
                return common::unauthorized_response("provide mismatch" ).into_response();
            }
            let code = req["code" ].as_str().unwrap_or("" );
            let status = req["status" ].as_i64().unwrap_or(0);
            let sum = req["sum" ].as_i64().unwrap_or(0);
            let cur = req["cur" ].as_i64().unwrap_or(0);
            let gps_type = req["gps_type" ].as_i64().unwrap_or(0);
            let alert = req["alert" ].as_str().unwrap_or("" );
            let remark = req["remark" ].as_str().unwrap_or("" );
            let points = req["points" ].as_str().unwrap_or("" );

            let info = StorageInfo {
                code: code.to_string(),
                status,
                provide: provide.to_string(),
                gps: Some(json!({
                    "lng": req["gps" ]["lng" ].as_str().unwrap_or("" ),
                    "lat": req["gps" ]["lat" ].as_str().unwrap_or("" ),
                })),
                gps_type,
                sum,
                cur,
                create_date: Some(chrono::Local::now().naive_local()),
                update_date: Some(chrono::Local::now().naive_local()),
                delete: Some(false),
                alert: Some(alert.to_string()),
                remark: Some(remark.to_string()),
                points: Some(points.to_string()),
                r#type: 0,
            };

            match state.storage_repo.add(&info).await {
                Ok(_) => common::success_with_message_response("success" ).into_response(),
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
                let storages: Vec<StorageInfo> = items.iter().map(|item| StorageInfo {
                    code: item["code" ].as_str().unwrap_or("" ).to_string(),
                    status: item["status" ].as_i64().unwrap_or(0),
                    provide: item["provide" ].as_str().unwrap_or("" ).to_string(),
                    gps: Some(json!({
                        "lng": item["gps" ]["lng" ].as_str().unwrap_or("" ),
                        "lat": item["gps" ]["lat" ].as_str().unwrap_or("" ),
                    })),
                    gps_type: item["gps_type" ].as_i64().unwrap_or(0),
                    sum: item["sum" ].as_i64().unwrap_or(0),
                    cur: item["cur" ].as_i64().unwrap_or(0),
                    create_date: Some(chrono::Local::now().naive_local()),
                    update_date: Some(chrono::Local::now().naive_local()),
                    delete: Some(false),
                    alert: Some(item["alert" ].as_str().unwrap_or("" ).to_string()),
                    remark: Some(item["remark" ].as_str().unwrap_or("" ).to_string()),
                    points: Some(item["points" ].as_str().unwrap_or("" ).to_string()),
                    r#type: 0,
                }).collect();

                match state.storage_repo.add_batch(&storages).await {
                    Ok(_) => common::success_with_message_response("success" ).into_response(),
                    Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
                }
            } else {
                common::bad_request_response("invalid items array" ).into_response()
            }
        }
        "delete" => {
            let code = req["code" ].as_str().unwrap_or("" );
            let provide = req["provide" ].as_str().unwrap_or("" );
            // 运营方只能删除自己的数据，admin 可删除所有
            if user_role != "admin" && user_role != provide {
                return common::unauthorized_response("provide mismatch" ).into_response();
            }
            match state.storage_repo.del(code).await {
                Ok(_) => common::success_with_message_response("success" ).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "query" => {
            let code = req["code" ].as_str().unwrap_or("" );
            // 运营方只能查自己，admin 可查所有
            let provide = if user_role == "admin" {
                req["provide" ].as_str().unwrap_or("" )
            } else {
                user_role
            };
            let status = req["status" ].as_i64().unwrap_or(-1);

            match state.storage_repo.query(code, provide, status).await {
                Ok(storages) => common::ok_response(storages).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "history" => {
            let code = req["code" ].as_str().unwrap_or("" );
            match state.storage_repo.history(code).await {
                Ok(storages) => common::ok_response(storages).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        _ => common::bad_request_response("unknown method" ).into_response(),
    }
}
