//! 系统配置管理路由

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use serde_json::{json, Value};

use crate::model::OptionsInfo;
use crate::AppState;

/// 系统配置管理入口（根据 `method` 字段分发）
///
/// 支持的操作：`query`, `update`
pub async fn options(
    State(state): State<AppState>,
    AxumJson(req): AxumJson<Value>,
) -> Response {
    let method = req["method" ].as_str().unwrap_or("" );

    match method {
        "query" => {
            match state.options_repo.query().await {
                Ok(options) => common::ok_response(options).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        "update" => {
            let name = req["name" ].as_str().unwrap_or("" );
            let system = req["options" ]["system" ].as_i64().unwrap_or(0);
            let alert = req["options" ]["alert" ].as_i64().unwrap_or(0);
            let level = req["level" ].as_i64().unwrap_or(5);

            let info = OptionsInfo {
                name: name.to_string(),
                options: json!({ "system": system, "alert": alert }),
                level,
                create_date: Some(chrono::Local::now().naive_local()),
                update_date: Some(chrono::Local::now().naive_local()),
                delete: Some(false),
            };

            match state.options_repo.update(&info).await {
                Ok(_) => common::success_with_message_response("success" ).into_response(),
                Err(e) => common::internal_error_response(&format!("{e:?}" )).into_response(),
            }
        }
        _ => common::bad_request_response("unknown method" ).into_response(),
    }
}
