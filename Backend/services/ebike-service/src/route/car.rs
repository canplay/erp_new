//! 车辆管理路由
//!
//! 支持车辆增删、批量导入、条件查询、历史轨迹和告警管理。

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use auth_core::AuthenticatedUser;
use serde_json::{json, Value};

use crate::model::CarInfo;
use crate::db::GenericRepository;
use crate::AppState;

/// 检查运营商车辆配额是否已满
/// 使用单次 LEFT JOIN 查询合并 COUNT 和配额获取
async fn check_quota(state: &AppState, provide: &str) -> Result<(), String> {
    // admin 不受配额限制
    // 单次查询：同时获取车辆数和配额
    let row = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) as cnt, COALESCE(o.vehicle_quota, 999999) as quota FROM car LEFT JOIN operators o ON car.provide = o.provide WHERE car.provide = $1 AND car.delete = false").bind(provide)
    .fetch_one(&*state.car_repo.pool())
    .await
    .map_err(|e| format!("运营商查询失败: {e}" ))?;

    let count: i64 = row;
    let quota: i32 = row as i32;

    if count >= i64::from(quota) {
        Err(format!(
            "配额已满: {provide} 当前 {count}/{quota} 辆, 无法新增 "
        ))
    } else {
        Ok(())
    }
}

/// 从缓存或数据库获取停放区列表（带缓存）
async fn get_storages_cached(state: &AppState) -> Result<Vec<crate::model::StorageInfo>, String> {
    let mut cache = state.storage_cache.write().await;
    // 缓存未初始化或过期时重新查询
    if cache.is_empty() {
        let storages = state.storage_repo.query(" ", "" , -1)
            .await
            .map_err(|e| format!("停放区查询失败: {e}" ))?;
        *cache = storages;
    }
    Ok(cache.clone())
}

/// 检查车辆 GPS 是否在任意停放区电子围栏内
/// 返回违停信息,None 表示合规
fn check_violation(
    lng: &str,
    lat: &str,
    storages: &[crate::model::StorageInfo],
) -> Option<String> {
    if lng.is_empty() || lat.is_empty() {
        return None;
    }
    let pt_lng: f64 = lng.parse().unwrap_or(0.0);
    let pt_lat: f64 = lat.parse().unwrap_or(0.0);
    for s in storages {
        if let Some(ref pts_json) = s.points {
            if let Ok(pts) = serde_json::from_str::<Vec<Vec<f64>>>(pts_json) {
                if pts.len() < 3 { continue; }
                // 射线法判断点是否在多边形内
                let mut inside = false;
                let mut j = pts.len() - 1;
                for i in 0..pts.len() {
                    let (xi, yi) = (pts[i][0], pts[i][1]);
                    let (xj, yj) = (pts[j][0], pts[j][1]);
                    if ((yi > pt_lat) != (yj > pt_lat))
                        && (pt_lng < (xj - xi) * (pt_lat - yi) / (yj - yi) + xi)
                    {
                        inside = !inside;
                    }
                    j = i;
                }
                if inside {
                    return None; // 在某个停放区内,合法
                }
            }
        }
    }
    Some("车辆不在任何已登记的停放区电子围栏内".to_string())
}

/// 车辆管理统一入口（根据 `method` 字段分发）
///
/// 支持的操作：`add`, `batch_add`, `delete`, `query`, `history`, `alert`, `violations`, `resolve_violation`
///
/// # HTTP 状态码语义
/// - `200`: 成功。`ok_response` 返回 `{"success": true, "data": [...]}`（query/history/alert/violations）；
///   `success_with_message_response` 返回 `{"success": true, "message": "..." }`（add/batch_add/delete/resolve_violation）。
/// - `400`: 业务/参数错误 `{"success": false, "error": "..." }`：配额已满、batch 超过 500 条、`items` 非数组、未知 `method`。
/// - `401`: 鉴权/数据隔离失败 `{"success": false, "error": "..." }`：运营方 provide 与账号不匹配、非 admin 处理违停。
/// - `500`: 数据库或内部错误 `{"success": false, "error": "..." }`。
///
/// # 各 method 的 JSON body 字段
/// - `add` (`*` 必填): `code*`, `provide*`, `status`, `speed`, `gps{lng,lat}`, `gps_type`, `time{start,end}`, `alert`, `remark`；
///   先做配额校验（配额满返回 400），再对 GPS 做电子围栏违停检测（违停仅记录不阻断）。
/// - `batch_add`: `items[]`（≤500 条，每项字段同 `add`），逐项做 provide 归属校验、配额校验与违停检测。
/// - `delete`: `code*`, `provide*`（provide 为空时仅按 code 软删）；非 admin 只能删自己的车。
/// - `query`: `code`, `provide`, `status`(-1 表示全部), `time{start,end}`, `limit`(默认 50，最大 500), `offset`(默认 0)；
///   返回车辆数组 `data`。非 admin 的 `provide` 被强制为当前账号。
/// - `history`: `code*`；返回当前月份历史表（car_history_yyyy_mm）中该车辆的轨迹数组。
/// - `alert`: `code`, `provide`, `status`(-1 全部), `time`, `alert`, `remark`；返回含告警的车辆数组。
/// - `violations`: `provide`, `status`(-1 全部)；返回违停记录数组（最多 200 条）。
/// - `resolve_violation`: `id*`, `remark`；仅 admin 可调用，成功返回 200 `{"success": true, "message": "违停已处理" }`。
pub async fn car(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    AxumJson(req): AxumJson<Value>,
) -> Response {
    let method = req["method" ].as_str().unwrap_or("" );
    let user_role = user.0.role.as_str();

    match method {
        "add" => {
            let provide = req["provide" ].as_str().unwrap_or("" );
            // 运营方只能操作自己的数据,admin 可操作所有
            if user_role != "admin" && user_role != provide {
                return common::unauthorized_response("provide mismatch: 运营商标识与账号不匹配 " ).into_response();
            }
            let code = req["code" ].as_str().unwrap_or("" );
            let status = req["status" ].as_i64();
            let provide = req["provide" ].as_str().unwrap_or("" );
            let speed = req["speed" ].as_f64().unwrap_or(0.0);
            let gps_type = req["gps_type" ].as_i64();
            let alert = req["alert" ].as_str().unwrap_or("" );
            let remark = req["remark" ].as_str().unwrap_or("" );

            let time_start = req["time" ]["start" ].as_str().unwrap_or("" );
            let time_end = req["time" ]["end" ].as_str().unwrap_or("" );

            let info = CarInfo {
                code: code.to_string(),
                status: status.unwrap_or(0),
                provide: provide.to_string(),
                speed,
                gps: Some(json!({
                    "lng": req["gps" ]["lng" ].as_str().unwrap_or("" ),
                    "lat": req["gps" ]["lat" ].as_str().unwrap_or("" ),
                })),
                gps_type: gps_type.unwrap_or(0),
                time: Some(json!({
                    "start": time_start.to_string(),
                    "end": time_end.to_string(),
                })),
                create_date: Some(chrono::Local::now().naive_local()),
                update_date: Some(chrono::Local::now().naive_local()),
                delete: Some(false),
                alert: Some(alert.to_string()),
                remark: Some(remark.to_string()),
                r#type: 0,
            };

            // 配额校验：查询该运营商当前车辆数是否已超配额
            let quota_ok = check_quota(&state, provide).await;
            if let Err(msg) = quota_ok {
                return common::bad_request_response(&msg).into_response();
            }

            // 违停检测：使用缓存的停放区列表
            let lng = req["gps" ]["lng" ].as_str().unwrap_or("" );
            let lat = req["gps" ]["lat" ].as_str().unwrap_or("" );
            if let Ok(storages) = get_storages_cached(&state).await {
                if let Some(violation_msg) = check_violation(lng, lat, &storages) {
                    let _ = state.car_repo.save_violation(
                        code, provide, lng, lat, &violation_msg
                    ).await;
                    tracing::warn!("[违停检测] 车辆 {} ({}) 违停: {}", code, provide, violation_msg);
                }
            }

            match state.car_repo.add(&info).await {
                Ok(_) => common::success_with_message_response("success" ).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "batch_add" => {
            const MAX_BATCH: usize = 500;
            if let Some(items) = req["items" ].as_array() {
                if items.len() > MAX_BATCH {
                    return common::bad_request_response(
                        &format!("batch size exceeded: max {MAX_BATCH} items per request " )
                    ).into_response();
                }
                // 校验批量下每辆车是否属于当前运营商
                for item in items {
                    let p = item["provide" ].as_str().unwrap_or("" );
                    if user_role != "admin" && user_role != p {
                        return common::unauthorized_response(
                            &format!("provide mismatch in batch: 车辆 {} 不属于当前运营方 ", item["code" ].as_str().unwrap_or("?" ))
                        ).into_response();
                    }
                }
                // 每辆车执行配额和违停检测
                let mut cars: Vec<CarInfo> = Vec::new();
                for item in items {
                    let code = item["code" ].as_str().unwrap_or("" );
                    let provide = item["provide" ].as_str().unwrap_or("" );
                    let lng = item["gps" ]["lng" ].as_str().unwrap_or("" );
                    let lat = item["gps" ]["lat" ].as_str().unwrap_or("" );

                    // 配额检查
                    if user_role != "admin" {
                        if let Err(msg) = check_quota(&state, provide).await {
                            return common::bad_request_response(&msg).into_response();
                        }
                    }

                    // 违停检测（使用缓存的停放区列表）
                    if let Ok(storages) = get_storages_cached(&state).await {
                        if let Some(violation_msg) = check_violation(lng, lat, &storages) {
                            let _ = state.car_repo.save_violation(
                                code, provide, lng, lat, &violation_msg
                            ).await;
                            tracing::warn!(
                                "[违停检测] 车辆 {} ({}) 违停: {}",
                                code, provide, violation_msg
                            );
                        }
                    }

                    cars.push(CarInfo {
                        code: code.to_string(),
                        status: item["status" ].as_i64().unwrap_or(0),
                        provide: provide.to_string(),
                        speed: item["speed" ].as_f64().unwrap_or(0.0),
                        gps: Some(json!({
                            "lng": lng,
                            "lat": lat,
                        })),
                        gps_type: item["gps_type" ].as_i64().unwrap_or(0),
                        time: Some(json!({
                            "start": item["time" ]["start" ].as_str().unwrap_or("" ).to_string(),
                            "end": item["time" ]["end" ].as_str().unwrap_or("" ).to_string(),
                        })),
                        create_date: Some(chrono::Local::now().naive_local()),
                        update_date: Some(chrono::Local::now().naive_local()),
                        delete: Some(false),
                        alert: Some(item["alert" ].as_str().unwrap_or("" ).to_string()),
                        remark: Some(item["remark" ].as_str().unwrap_or("" ).to_string()),
                        r#type: 0,
                    });
                }

                match state.car_repo.add_batch(&cars).await {
                    Ok(_) => common::success_with_message_response("success" ).into_response(),
                    Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
                }
            } else {
                common::bad_request_response("invalid items array " ).into_response()
            }
        }
        "delete" => {
            let code = req["code" ].as_str().unwrap_or("" );
            let provide = req["provide" ].as_str().unwrap_or("" );
            if user_role != "admin" && user_role != provide {
                return common::unauthorized_response("provide mismatch " ).into_response();
            }
            match state.car_repo.del(code, provide).await {
                Ok(_) => common::success_with_message_response("success" ).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "query" => {
            let code = req["code" ].as_str().unwrap_or("" );
            // 运营方只能查自己,admin 可查所有
            let provide = if user_role == "admin" {
                req["provide" ].as_str().unwrap_or("" )
            } else {
                user_role
            };
            let status = req["status" ].as_i64().unwrap_or(-1);
            let time_start = req["time" ]["start" ].as_str().unwrap_or("" );
            let time_end = req["time" ]["end" ].as_str().unwrap_or("" );
            // 分页参数：默认每页 50 条，最大 500 条
            let limit = req["limit" ].as_i64().unwrap_or(50).clamp(1, 500);
            let offset = req["offset" ].as_i64().unwrap_or(0).max(0);

            match state
                .car_repo
                .query(code, provide, status, time_start, time_end, limit, offset)
                .await
            {
                Ok(cars) => common::ok_response(cars).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "history" => {
            let code = req["code" ].as_str().unwrap_or("" );
            match state.car_repo.history(code).await {
                Ok(cars) => common::ok_response(cars).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "alert" => {
            let code = req["code" ].as_str().unwrap_or("" );
            let provide = if user_role == "admin" {
                req["provide" ].as_str().unwrap_or("" )
            } else {
                user_role
            };
            let status = req["status" ].as_i64().unwrap_or(-1);
            let time = req["time" ].as_str().unwrap_or("" );
            let alert = req["alert" ].as_str().unwrap_or("" );
            let remark = req["remark" ].as_str().unwrap_or("" );

            match state
                .car_repo
                .alert(code, provide, status, time, alert, remark)
                .await
            {
                // car_alert 返回 CarListResponse（告警后的车辆列表），与 car_query 语义一致
                Ok(cars) => common::ok_response(cars).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        // 违停管理
        "violations" => {
            let provide = if user_role == "admin" {
                req["provide" ].as_str().unwrap_or("" )
            } else {
                user_role
            };
            let status = req["status" ].as_i64().unwrap_or(-1);
            match state.car_repo.list_violations(provide, status).await {
                Ok(list) => common::ok_response(list).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "resolve_violation" => {
            if user_role != "admin" {
                return common::unauthorized_response("仅管理员可处理违停" ).into_response();
            }
            let id = req["id" ].as_i64();
            let remark = req["remark" ].as_str().unwrap_or("" );
            match state.car_repo.resolve_violation(id.unwrap_or(0), remark).await {
                Ok(_) => common::success_with_message_response("违停已处理" ).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        _ => common::bad_request_response("unknown method " ).into_response(),
    }
}
