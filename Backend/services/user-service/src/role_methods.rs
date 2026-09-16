    // ============ 角色管理 ============

    async fn list_roles(
        &self,
        request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let req = request.into_inner();
        let page = req.page.max(1);
        let page_size = req.page_size.clamp(1, 100);
        let keyword = if req.keyword.is_empty() {
            None
        } else {
            Some(req.keyword.as_str())
        };

        let result = self
            .state
            .role_repository
            .list(page, page_size, keyword)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let roles: Vec<RoleInfo> = result
            .roles
            .into_iter()
            .map(|r| RoleInfo {
                id: r.id,
                name: r.name,
                description: r.description.unwrap_or_default(),
                r#type: r.role_type,
                status: r.status,
                created_at: r.created_at.timestamp(),
                updated_at: 0,
                // 修复 (2026-08-11 QA): 填充角色用户数, 之前丢弃导致前端恒显示 0
                user_count: r.user_count,
            })
            .collect();

        Ok(Response::new(ListRolesResponse {
            roles,
            total: result.total,
        }))
    }

    async fn get_role(
        &self,
        request: Request<GetRoleRequest>,
    ) -> Result<Response<GetRoleResponse>, Status> {
        let req = request.into_inner();

        let role = self
            .state
            .role_repository
            .find_by_code(&req.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match role {
            Some(role) => Ok(Response::new(GetRoleResponse {
                role: Some(RoleInfo {
                    id: role.id,
                    name: role.name,
                    description: role.description.unwrap_or_default(),
                    r#type: role.role_type,
                    status: role.status,
                    created_at: role.created_at.timestamp(),
                    updated_at: role.updated_at.timestamp(),
                    user_count: 0,
                }),
            })),
            None => Err(Status::not_found("角色不存在" )),
        }
    }

    async fn create_role(
        &self,
        request: Request<CreateRoleRequest>,
    ) -> Result<Response<CreateRoleResponse>, Status> {
        let req = request.into_inner();

        if req.name.is_empty() {
            return Err(Status::invalid_argument("角色名称不能为空" ));
        }

        // 生成的 code 取自 name，转为小写并替换空白为破折号
        let code = req.name.to_lowercase().replace([' ', '\t'], "-" );

        let role_id = self
            .state
            .role_repository
            .create(
                &req.name,
                &code,
                Some(req.description.clone()),
                &req.r#type,
                None,
            )
            .await
            .map_err(|e| match e {
                crate::repository::RoleRepositoryError::AlreadyExists => {
                    Status::already_exists("角色已存在" )
                }
                other => Status::internal(format!("{:?}" , other)),
            })?;

        Ok(Response::new(CreateRoleResponse {
            role: Some(RoleInfo {
                id: role_id,
                name: req.name,
                description: req.description,
                r#type: req.r#type,
                status: 1,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                user_count: 0,
            }),
        }))
    }

    async fn update_role(
        &self,
        request: Request<UpdateRoleRequest>,
    ) -> Result<Response<UpdateRoleResponse>, Status> {
        let req = request.into_inner();

        // 根据 name（即 code）查找并更新
        let existing = self
            .state
            .role_repository
            .find_by_code(&req.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match existing {
            Some(_r) => {
                self.state
                    .role_repository
                    .update(
                        &req.name,
                        Some(req.description).filter(|s| !s.is_empty()),
                        None,
                        Some(req.status),
                    )
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                Ok(Response::new(UpdateRoleResponse { success: true }))
            }
            None => Err(Status::not_found("角色不存在" )),
        }
    }

    async fn delete_role(
        &self,
        request: Request<DeleteRoleRequest>,
    ) -> Result<Response<DeleteRoleResponse>, Status> {
        let req = request.into_inner();

        let result = self
            .state
            .role_repository
            .delete(&req.name)
            .await
            .map_err(|e| match e {
                crate::repository::RoleRepositoryError::HasAssociatedUsers => {
                    Status::failed_precondition("角色有用户关联，无法删除" )
                }
                crate::repository::RoleRepositoryError::HasChildRoles => {
                    Status::failed_precondition("角色有子角色，无法删除" )
                }
                other => Status::internal(format!("{:?}" , other)),
            })?;

        Ok(Response::new(DeleteRoleResponse { success: result }))
    }

    async fn get_role_permissions(
        &self,
        request: Request<GetRolePermissionsRequest>,
    ) -> Result<Response<GetRolePermissionsResponse>, Status> {
        let req = request.into_inner();

        let role = self
            .state
            .role_repository
            .find_by_code(&req.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("角色不存在" ))?;

        let permissions = self
            .state
            .role_repository
            .get_permissions(role.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let perm_codes: Vec<String> = permissions.into_iter().map(|p| p.code).collect();

        Ok(Response::new(GetRolePermissionsResponse {
            permissions: perm_codes,
        }))
    }

    async fn set_role_permissions(
        &self,
        request: Request<SetRolePermissionsRequest>,
    ) -> Result<Response<SetRolePermissionsResponse>, Status> {
        let req = request.into_inner();

        let role = self
            .state
            .role_repository
            .find_by_code(&req.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("角色不存在" ))?;

        // 将权限 code 转为 ID（role_repository 已有 resolve_permission_ids 方法）
        let permission_ids = self
            .state
            .role_repository
            .resolve_permission_ids(&req.permissions)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        self.state
            .role_repository
            .set_permissions(role.id, &permission_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SetRolePermissionsResponse { success: true }))
    }

    async fn get_role_users(
        &self,
        request: Request<GetRoleUsersRequest>,
    ) -> Result<Response<GetRoleUsersResponse>, Status> {
        let req = request.into_inner();
        let page = req.page.max(1);
        let page_size = req.page_size.clamp(1, 100);

        let role = self
            .state
            .role_repository
            .find_by_code(&req.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("角色不存在" ))?;

        let (user_ids, total) = self
            .state
            .role_repository
            .get_users(role.id, page, page_size)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 根据 user_ids 获取用户详情
        let mut users = Vec::new();
        for uid in user_ids {
            if let Ok(Some(u)) = self.state.user_repository.find_by_id(uid).await {
                users.push(GetUserResponse {
                    id: u.id,
                    username: u.username,
                    nickname: u.nickname.unwrap_or_default(),
                    avatar: u.avatar.unwrap_or_default(),
                    phone: u.phone.unwrap_or_default(),
                    email: u.email.unwrap_or_default(),
                    gender: u.gender.unwrap_or(0),
                    address: u.address.unwrap_or_default(),
                    role: u.role,
                    status: u.status,
                    created_at: u.created_at.timestamp(),
                    updated_at: u.updated_at.timestamp(),
                });
            }
        }

        Ok(Response::new(GetRoleUsersResponse { users, total }))
    }
