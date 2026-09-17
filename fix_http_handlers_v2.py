import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# The issue: nested parens in json_success(serde_json::json!(...)) confuse the parser
# Fix: use a simpler pattern with intermediate variables

# Fix 1: create_user function - replace the match block
old_create_user = '''    match state.user_repository.create(&user).await {
        Ok(user_id) => (
            StatusCode::CREATED,
            json_success(serde_json::json!({
                    "id": user_id,
                    "username": req.username
                }),
        )
            .into_response(),
        Err(crate::repository::UserRepositoryError::AlreadyExists) => (
            StatusCode::CONFLICT,
            json_error("用户名已存在"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("创建用户失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("创建用户失败"),
            )
                .into_response()
        }
    }'''

new_create_user = '''    match state.user_repository.create(&user).await {
        Ok(user_id) => {
            let response = serde_json::json!({
                "id": user_id,
                "username": req.username
            });
            (StatusCode::CREATED, json_success(response)).into_response()
        }
        Err(crate::repository::UserRepositoryError::AlreadyExists) => {
            (StatusCode::CONFLICT, json_error("用户名已存在")).into_response()
        }
        Err(e) => {
            tracing::error!("创建用户失败: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, json_error("创建用户失败")).into_response()
        }
    }'''

content = content.replace(old_create_user, new_create_user)

# Fix 2: list_users function
old_list_users = '''    match state.user_repository.list(page, page_size).await {
        Ok(result) => {
            let users: Vec<UserResponse> = result.users.into_iter().map(std::convert::Into::into).collect();
            (
                StatusCode::OK,
                json_success(serde_json::json!({
                        "list": users,
                        "total": result.total,
                        "page": page,
                        "page_size": page_size
                    }),
            ))
                .into_response()
        }
        Err(e) => {
            tracing::error!("查询用户列表失败: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询用户列表失败"),
            )
                .into_response()
        }
    }'''

new_list_users = '''    match state.user_repository.list(page, page_size).await {
        Ok(result) => {
            let users: Vec<UserResponse> = result.users.into_iter().map(std::convert::Into::into).collect();
            let response = serde_json::json!({
                "list": users,
                "total": result.total,
                "page": page,
                "page_size": page_size
            });
            (StatusCode::OK, json_success(response)).into_response()
        }
        Err(e) => {
            tracing::error!("查询用户列表失败: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, json_error("查询用户列表失败")).into_response()
        }
    }'''

content = content.replace(old_list_users, new_list_users)

# Fix 3: get_import_template function
old_get_import = '''    (
        StatusCode::OK,
        json_success(serde_json::json!({
                "template": template,
                "columns": ["username", "password", "email", "nickname", "phone", "role"],
                "description": "用户导入模板，请按此格式填写 CSV 文件"
            }),
    )
        .into_response()'''

new_get_import = '''    {
        let response = serde_json::json!({
            "template": template,
            "columns": ["username", "password", "email", "nickname", "phone", "role"],
            "description": "用户导入模板，请按此格式填写 CSV 文件"
        });
        (StatusCode::OK, json_success(response)).into_response()
    }'''

content = content.replace(old_get_import, new_get_import)

# Fix 4: import_users function - json_success_msg
old_import = '''    (
        StatusCode::OK,
        json_success_msg({
                "total": total,
                "success": success_count,
                "failed": fail_count,
                "errors": if errors.len() > 10 {
                    errors[..10].to_vec()
                } else {
                    errors
                }
            }, &format!("导入完成: 成功 {}"),
    )
        .into_response()'''

new_import = '''    {
        let response = serde_json::json!({
            "total": total,
            "success": success_count,
            "failed": fail_count,
            "errors": if errors.len() > 10 {
                errors[..10].to_vec()
            } else {
                errors
            }
        });
        (StatusCode::OK, json_success_msg(response, &format!("导入完成: 成功 {}", success_count))).into_response()
    }'''

content = content.replace(old_import, new_import)

# Fix 5: batch_update_user_role
old_batch_role = '''        Ok(result) => (
            StatusCode::OK,
            json_success_msg({
                    "success_count": result.success_count,
                    "fail_count": result.fail_count,
                    "errors": result.errors.iter().map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "message": e.message
                        })
                    }).collect::<Vec<_>>()
                }, &format!("成功更新 {}"),
        ).into_response(),'''

new_batch_role = '''        Ok(result) => {
            let response = serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            });
            (StatusCode::OK, json_success_msg(response, &format!("成功更新 {}", result.success_count))).into_response()
        }'''

content = content.replace(old_batch_role, new_batch_role)

# Fix 6: batch_update_user_status
old_batch_status = '''        Ok(result) => (
            StatusCode::OK,
            json_success_msg({
                    "success_count": result.success_count,
                    "fail_count": result.fail_count,
                    "errors": result.errors.iter().map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "message": e.message
                        })
                    }).collect::<Vec<_>>()
                }, &format!("成功更新 {}"),
        ).into_response(),'''

new_batch_status = '''        Ok(result) => {
            let response = serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            });
            (StatusCode::OK, json_success_msg(response, &format!("成功更新 {}", result.success_count))).into_response()
        }'''

content = content.replace(old_batch_status, new_batch_status)

# Fix 7: batch_delete_users
old_batch_delete = '''        Ok(result) => (
            StatusCode::OK,
            json_success_msg({
                    "success_count": result.success_count,
                    "fail_count": result.fail_count,
                    "errors": result.errors.iter().map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "message": e.message
                        })
                    }).collect::<Vec<_>>()
                }, &format!("成功删除 {}"),
        ).into_response(),'''

new_batch_delete = '''        Ok(result) => {
            let response = serde_json::json!({
                "success_count": result.success_count,
                "fail_count": result.fail_count,
                "errors": result.errors.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "message": e.message
                    })
                }).collect::<Vec<_>>()
            });
            (StatusCode::OK, json_success_msg(response, &format!("成功删除 {}", result.success_count))).into_response()
        }'''

content = content.replace(old_batch_delete, new_batch_delete)

# Fix 8: export_users function
old_export = '''        Ok(result) => {
            // 预分配 String 容量以提高性能
            let mut csv_content = String::with_capacity(result.users.len() * 200);
            csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\\n");

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
        }'''

new_export = '''        Ok(result) => {
            // 预分配 String 容量以提高性能
            let mut csv_content = String::with_capacity(result.users.len() * 200);
            csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\n");

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

            let response = axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/csv; charset=utf-8")
                .header("Content-Disposition", "attachment; filename=users_export.csv")
                .body(axum::body::Body::from(csv_content));
            
            match response {
                Ok(resp) => resp.into_response(),
                Err(e) => {
                    tracing::error!("构建导出用户响应失败: {e}");
                    axum::response::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from("内部服务器错误"))
                        .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
                        .into_response()
                }
            }
        }'''

content = content.replace(old_export, new_export)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
