//! 批量操作/导入导出/重置密码处理器
use super::*;

// ============ 批量操作请求/响应 ============

/// 批量更新角色请求
#[derive(Debug, Deserialize)]
pub(crate) struct BatchUpdateRoleRequest {
    pub user_ids: Vec<i64>,
    pub role: String,
}

/// 重置密码请求
#[derive(Debug, Deserialize)]
pub(crate) struct ResetPasswordRequest {
    pub new_password: Option<String>,
}

/// 重置用户密码
pub(crate) async fn reset_user_password(
    State(state): State<HttpAppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    // 生成或使用提供的密码
    let password_to_hash = req.new_password.as_deref().unwrap_or(&default_password()).to_string();
    let password_hash = match hash_password(&password_to_hash) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("密码哈希失败: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("密码处理失败"),
            )
                .into_response();
        }
    };

    match state
        .user_repository
        .update_password(user_id, &password_hash)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "new_password": password_to_hash
            }), "密码重置成功"),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("用户不存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("重置密码失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("重置密码失败"),
            )
                .into_response()
        }
    }
}

/// 获取导入模板（返回 CSV 模板结构）
pub(crate) async fn get_import_template() -> impl IntoResponse {
    let template = "username,password,email,nickname,phone,role\nuser1,,user1@example.com,用户1,13800000001,user\nuser2,,user2@example.com,用户2,13800000002,user\n";

    (
        StatusCode::OK,
        json_success(serde_json::json!({
            "template": template,
            "columns": ["username", "password", "email", "nickname", "phone", "role"],
            "description": "用户导入模板，请按此格式填写 CSV 文件"
        })),
    )
        .into_response()
}

/// 下载导入模板（返回 CSV 格式）
pub(crate) async fn download_import_template() -> impl IntoResponse {
    let csv_content = "username,password,email,nickname,phone,role\nuser1,,user1@example.com,用户1,13800000001,user\nuser2,,user2@example.com,用户2,13800000002,user\n";

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/csv; charset=utf-8")
        .header(
            "Content-Disposition",
            "attachment; filename=user_import_template.csv",
        )
        .body(axum::body::Body::from(csv_content))
        .unwrap_or_else(|e| {
            tracing::error!("构建下载模板响应失败: {e}");
            axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from("内部服务器错误"))
                .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
        })
        .into_response()
}

/// 导入用户（解析上传的 CSV 文件）
pub(crate) async fn import_users(
    State(state): State<HttpAppState>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let body = req.into_body();
    let bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("读取上传文件失败: {e}");
            return (
                StatusCode::BAD_REQUEST,
                json_error("无法读取上传文件"),
            )
                .into_response();
        }
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(bytes.as_ref());

    let mut total = 0;
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        total += 1;

        match result {
            Ok(record) => {
                if record.len() < 6 {
                    fail_count += 1;
                    errors.push(format!("行 {}: 列数不足", idx + 2));
                    continue;
                }

                let username = record.get(0).unwrap_or("").trim();
                let password = record.get(1).unwrap_or("").trim();
                let email = record.get(2).unwrap_or("").trim();
                let nickname = record.get(3).unwrap_or("").trim();
                let phone = record.get(4).unwrap_or("").trim();
                let _role = record.get(5).unwrap_or("user").trim();

                // 参数校验
                if let Err(msg) = validate_username(username) {
                    fail_count += 1;
                    errors.push(format!("行 {}: {}", idx + 2, msg));
                    continue;
                }

                // 密码哈希
                let password_to_hash = if password.is_empty() {
                    default_password()
                } else {
                    password.to_string()
                };
                let password_hash = match hash_password(&password_to_hash) {
                    Ok(h) => h,
                    Err(e) => {
                        fail_count += 1;
                        errors.push(format!("行 {}: 密码处理失败 - {}", idx + 2, e));
                        continue;
                    }
                };

                // 创建用户
                match state
                    .user_repository
                    .create(
                        username,
                        &password_hash,
                        if email.is_empty() {
                            None
                        } else {
                            Some(email.to_string())
                        },
                        if nickname.is_empty() {
                            None
                        } else {
                            Some(nickname.to_string())
                        },
                        if phone.is_empty() {
                            None
                        } else {
                            Some(phone.to_string())
                        },
                        Some(1), // gender default
                    )
                    .await
                {
                    Ok(_) => {
                        success_count += 1;
                        tracing::info!("成功导入用户: {username}");
                    }
                    Err(crate::repository::UserRepositoryError::AlreadyExists) => {
                        fail_count += 1;
                        errors.push(format!("行 {}: 用户名 '{}' 已存在", idx + 2, username));
                    }
                    Err(e) => {
                        fail_count += 1;
                        errors.push(format!("行 {}: 创建用户失败 - {:?}", idx + 2, e));
                    }
                }
            }
            Err(e) => {
                fail_count += 1;
                errors.push(format!("行 {}: 解析错误 - {}", idx + 2, e));
            }
        }
    }

    tracing::info!(
        "用户导入完成: 总数={total}, 成功={success_count}, 失败={fail_count}"
    );

    (
        StatusCode::OK,
        json_success_msg(serde_json::json!({
            "total": total,
            "success": success_count,
            "failed": fail_count,
            "errors": if errors.len() > 10 {
                errors[..10].to_vec()
            } else {
                errors
            }
        }), &format!("导入完成: 成功 {}", success_count)),
    )
        .into_response()
}

/// 导出用户（生成 CSV 文件）
pub(crate) async fn export_users(
    State(state): State<HttpAppState>,
    Query(_params): Query<UserQueryParams>,
) -> impl IntoResponse {
    let page_size = 10000; // 一次导出最大数量

    match state.user_repository.list(1, page_size).await {
        Ok(result) => {
            // 预分配 String 容量以提高性能
            let mut csv_content = String::with_capacity(result.users.len() * 200);
            csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");
            csv_content.push('\n');

            for user in result.users {
                use std::fmt::Write as FmtWrite;
                let _ = writeln!(
                    csv_content,
                    "{},{},{},{},{},{},{},{}",
                    user.id,
                    user.username,
                    user.nickname.unwrap_or_default(),
                    user.email.unwrap_or_default(),
                    user.phone.unwrap_or_default(),
                    user.role,
                    user.status,
                    user.created_at.to_rfc3339()
                );
            }

            axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/csv; charset=utf-8")
                .header(
                    "Content-Disposition",
                    "attachment; filename=users_export.csv",
                )
                .body(axum::body::Body::from(csv_content))
                .unwrap_or_else(|e| {
                    tracing::error!("构建导出用户响应失败: {e}");
                    axum::response::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from("内部服务器错误"))
                        .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
                })
                .into_response()
        }
        Err(e) => {
            tracing::error!("导出用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("导出用户失败"),
            )
                .into_response()
        }
    }
}

/// 批量更新状态请求
#[derive(Debug, Deserialize)]
pub(crate) struct BatchUpdateStatusRequest {
    pub user_ids: Vec<i64>,
    pub status: i32,
}

/// 批量删除请求
#[derive(Debug, Deserialize)]
pub(crate) struct BatchDeleteRequest {
    pub user_ids: Vec<i64>,
}

/// 批量更新用户角色（使用事务优化）
pub(crate) async fn batch_update_user_role(
    State(state): State<HttpAppState>,
    Json(req): Json<BatchUpdateRoleRequest>,
) -> impl IntoResponse {
    // 角色值校验
    if let Err(msg) = validate_role(&req.role) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 使用事务批量更新
    match state.user_repository.batch_update_role(&req.user_ids, &req.role).await {
        Ok(result) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            }), &format!("成功更新 {} 个用户的角色", result.success_count)),
        ).into_response(),
        Err(e) => {
            tracing::error!("批量更新用户角色失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("批量更新用户角色失败"),
            ).into_response()
        }
    }
}

/// 批量更新用户状态（使用事务优化）
pub(crate) async fn batch_update_user_status(
    State(state): State<HttpAppState>,
    Json(req): Json<BatchUpdateStatusRequest>,
) -> impl IntoResponse {
    // 状态值校验
    if let Err(msg) = validate_status(req.status) {
        return (
            StatusCode::BAD_REQUEST,
            json_error(&format!("{}", msg)),
        )
            .into_response();
    }

    // 使用事务批量更新
    match state.user_repository.batch_update_status(&req.user_ids, req.status).await {
        Ok(result) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            }), &format!("成功更新 {} 个用户的状态", result.success_count)),
        ).into_response(),
        Err(e) => {
            tracing::error!("批量更新用户状态失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("批量更新用户状态失败"),
            ).into_response()
        }
    }
}

/// 批量删除用户（使用事务优化）
pub(crate) async fn batch_delete_users(
    State(state): State<HttpAppState>,
    Json(req): Json<BatchDeleteRequest>,
) -> impl IntoResponse {
    // 使用事务批量删除
    match state.user_repository.batch_delete(&req.user_ids).await {
        Ok(result) => (
            StatusCode::OK,
            json_success_msg(serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            }), &format!("成功删除 {} 个用户", result.success_count)),
        ).into_response(),
        Err(e) => {
            tracing::error!("批量删除用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("批量删除用户失败"),
            ).into_response()
        }
    }
}

