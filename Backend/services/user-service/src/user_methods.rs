    // ============ 用户 CRUD ============

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .state
            .user_repository
            .find_by_id(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match user {
            Some(user) => {
                let response = GetUserResponse {
                    id: user.id,
                    username: user.username,
                    nickname: user.nickname.unwrap_or_default(),
                    avatar: user.avatar.unwrap_or_default(),
                    phone: user.phone.unwrap_or_default(),
                    email: user.email.unwrap_or_default(),
                    gender: user.gender.unwrap_or(0),
                    address: user.address.unwrap_or_default(),
                    role: user.role,
                    status: user.status,
                    created_at: user.created_at.timestamp(),
                    updated_at: user.updated_at.timestamp(),
                };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("User not found")),
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();

        if req.username.is_empty() {
            return Err(Status::invalid_argument("用户名不能为空"));
        }
        if req.username.len() > 50 {
            return Err(Status::invalid_argument("用户名长度不能超过50个字符"));
        }
        if req.password.len() < 8 {
            return Err(Status::invalid_argument("密码长度至少8位"));
        }

        let username = req.username.clone();
        let nickname = if req.nickname.is_empty() {
            None
        } else {
            Some(req.nickname.clone())
        };
        let email = if req.email.is_empty() {
            None
        } else {
            Some(req.email)
        };
        let phone = if req.phone.is_empty() {
            None
        } else {
            Some(req.phone)
        };
        let gender = if req.gender == 0 {
            None
        } else {
            Some(req.gender)
        };

        let user_id = self
            .state
            .user_repository
            .create(&req.username, &req.password, email, nickname, phone, gender)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = CreateUserResponse {
            id: user_id,
            username,
            nickname: if req.nickname.is_empty() {
                String::new()
            } else {
                req.nickname
            },
        };
        Ok(Response::new(response))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .state
            .user_repository
            .update(
                req.user_id,
                Some(req.nickname).filter(|s| !s.is_empty()),
                Some(req.gender).filter(|&g| g != 0),
                Some(req.address).filter(|s| !s.is_empty()),
                Some(req.avatar).filter(|s| !s.is_empty()),
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match user {
            Some(user) => {
                let response = UpdateUserResponse {
                    id: user.id,
                    username: user.username,
                    nickname: user.nickname.unwrap_or_default(),
                };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("User not found")),
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();

        let success = self
            .state
            .user_repository
            .delete(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteUserResponse { success }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();

        let page = req.page.max(1);
        let page_size = req.page_size.clamp(1, 100);

        let result = self
            .state
            .user_repository
            .list(page, page_size)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let users: Vec<GetUserResponse> = result
            .users
            .into_iter()
            .map(|user| GetUserResponse {
                id: user.id,
                username: user.username,
                nickname: user.nickname.unwrap_or_default(),
                avatar: user.avatar.unwrap_or_default(),
                phone: user.phone.unwrap_or_default(),
                email: user.email.unwrap_or_default(),
                gender: user.gender.unwrap_or(0),
                address: user.address.unwrap_or_default(),
                role: user.role,
                status: user.status,
                created_at: user.created_at.timestamp(),
                updated_at: user.updated_at.timestamp(),
            })
            .collect();

        Ok(Response::new(ListUsersResponse {
            users,
            total: result.total,
        }))
    }

    // ============ 用户扩展操作 ============

    async fn update_user_status(
        &self,
        request: Request<UpdateUserStatusRequest>,
    ) -> Result<Response<UpdateUserStatusResponse>, Status> {
        let req = request.into_inner();

        let found = self
            .state
            .user_repository
            .update_status(req.user_id, req.status)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !found {
            return Err(Status::not_found("用户不存在"));
        }

        Ok(Response::new(UpdateUserStatusResponse { success: true }))
    }

    async fn update_user_role(
        &self,
        request: Request<UpdateUserRoleRequest>,
    ) -> Result<Response<UpdateUserRoleResponse>, Status> {
        let req = request.into_inner();

        if req.role.is_empty() {
            return Err(Status::invalid_argument("角色不能为空"));
        }

        let found = self
            .state
            .user_repository
            .update_role(req.user_id, &req.role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !found {
            return Err(Status::not_found("用户不存在"));
        }

        Ok(Response::new(UpdateUserRoleResponse { success: true }))
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();

        // Security fix: 不允许默认密码，必须提供 new_password
        if req.new_password.is_empty() {
            return Err(Status::invalid_argument("new_password is required"));
        }
        if req.new_password.len() < 8 {
            return Err(Status::invalid_argument("new_password must be at least 8 characters"));
        }

        use argon2::password_hash::{SaltString, rand_core::OsRng};
        use argon2::PasswordHasher;
        let password_to_hash = &req.new_password;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = argon2::Argon2::default();
        let password_hash = argon2
            .hash_password(password_to_hash.as_bytes(), &salt)
            .map_err(|e| Status::internal(format!("密码哈希失败: {e}")))?
            .to_string();

        let found = self
            .state
            .user_repository
            .update_password(req.user_id, &password_hash)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !found {
            return Err(Status::not_found("用户不存在"));
        }

        Ok(Response::new(ResetPasswordResponse { success: true }))
    }

    async fn batch_update_user_role(
        &self,
        request: Request<BatchUpdateUserRoleRequest>,
    ) -> Result<Response<BatchUpdateUserRoleResponse>, Status> {
        let req = request.into_inner();

        if req.user_ids.is_empty() {
            return Ok(Response::new(BatchUpdateUserRoleResponse {
                success: true,
                affected: 0,
            }));
        }

        let result = self
            .state
            .user_repository
            .batch_update_role(&req.user_ids, &req.role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(BatchUpdateUserRoleResponse {
            success: result.fail_count == 0,
            affected: result.success_count as i32,
        }))
    }

    async fn batch_update_user_status(
        &self,
        request: Request<BatchUpdateUserStatusRequest>,
    ) -> Result<Response<BatchUpdateUserStatusResponse>, Status> {
        let req = request.into_inner();

        if req.user_ids.is_empty() {
            return Ok(Response::new(BatchUpdateUserStatusResponse {
                success: true,
                affected: 0,
            }));
        }

        let result = self
            .state
            .user_repository
            .batch_update_status(&req.user_ids, req.status)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(BatchUpdateUserStatusResponse {
            success: result.fail_count == 0,
            affected: result.success_count as i32,
        }))
    }

    async fn batch_delete_users(
        &self,
        request: Request<BatchDeleteUsersRequest>,
    ) -> Result<Response<BatchDeleteUsersResponse>, Status> {
        let req = request.into_inner();

        if req.user_ids.is_empty() {
            return Ok(Response::new(BatchDeleteUsersResponse {
                success: true,
                deleted: 0,
            }));
        }

        let result = self
            .state
            .user_repository
            .batch_delete(&req.user_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(BatchDeleteUsersResponse {
            success: result.fail_count == 0,
            deleted: result.success_count as i32,
        }))
    }
