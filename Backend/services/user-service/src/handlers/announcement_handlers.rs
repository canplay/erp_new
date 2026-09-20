use grpc_proto::user::*;
use tonic::{Request, Response, Status};
use super::UserServiceImpl;
use crate::repository::{CreateAnnouncementParams, UpdateAnnouncementParams};

pub(crate) async fn list_announcements(s: &UserServiceImpl, request: Request<ListAnnouncementsRequest>) -> Result<Response<ListAnnouncementsResponse>, Status> {
    let req = request.into_inner();
    let is_active = if req.is_active == 0 { None } else { Some(req.is_active == 1) };
    let result = s.state.announcement_repository.list(req.page.max(1), req.page_size.clamp(1, 100), is_active)
        .await.map_err(|e| Status::internal(e.to_string()))?;

    let announcements: Vec<AnnouncementInfo> = result.announcements.into_iter().map(|a| AnnouncementInfo {
        id: a.id, title: a.title, content: String::new(), r#type: a.announcement_type,
        priority: a.priority, is_pinned: a.is_pinned, is_active: a.is_active,
        created_by: a.created_by_name.unwrap_or_default(),
        start_time: a.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
        end_time: a.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
        created_at: a.created_at.timestamp(), updated_at: 0,
    }).collect();

    Ok(Response::new(ListAnnouncementsResponse { announcements, total: result.total }))
}

pub(crate) async fn get_announcement(s: &UserServiceImpl, request: Request<GetAnnouncementRequest>) -> Result<Response<GetAnnouncementResponse>, Status> {
    let req = request.into_inner();
    let a = s.state.announcement_repository.find_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    match a {
        Some(a) => Ok(Response::new(GetAnnouncementResponse {
            announcement: Some(AnnouncementInfo {
                id: a.id, title: a.title, content: a.content, r#type: a.announcement_type,
                priority: a.priority, is_pinned: a.is_pinned, is_active: a.is_active,
                created_by: a.created_by_name.unwrap_or_default(),
                start_time: a.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                end_time: a.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                created_at: a.created_at.timestamp(), updated_at: a.updated_at.timestamp(),
            }),
        })),
        None => Err(Status::not_found("公告不存在")),
    }
}

pub(crate) async fn create_announcement(s: &UserServiceImpl, request: Request<CreateAnnouncementRequest>) -> Result<Response<CreateAnnouncementResponse>, Status> {
    let req = request.into_inner();
    let created_by: Option<i64> = req.created_by.parse().ok();
    let id = s.state.announcement_repository.create(
        CreateAnnouncementParams {
            title: &req.title,
            content: &req.content,
            announcement_type: &req.r#type,
            priority: req.priority,
            is_pinned: req.is_pinned,
            is_active: req.is_active,
            start_time: None,
            end_time: None,
            created_by,
        }
    ).await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(CreateAnnouncementResponse {
        announcement: Some(AnnouncementInfo {
            id, title: req.title, content: req.content, r#type: req.r#type,
            priority: req.priority, is_pinned: req.is_pinned, is_active: req.is_active,
            created_by: req.created_by, start_time: String::new(), end_time: String::new(),
            created_at: chrono::Utc::now().timestamp(), updated_at: chrono::Utc::now().timestamp(),
        }),
    }))
}

pub(crate) async fn update_announcement(s: &UserServiceImpl, request: Request<UpdateAnnouncementRequest>) -> Result<Response<UpdateAnnouncementResponse>, Status> {
    let req = request.into_inner();
    let exists = s.state.announcement_repository.find_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if exists.is_none() { return Err(Status::not_found("公告不存在")); }

    let title = if req.title.is_empty() { None } else { Some(req.title) };
    let content = if req.content.is_empty() { None } else { Some(req.content) };
    let announcement_type = if req.r#type.is_empty() { None } else { Some(req.r#type) };
    let priority = if req.priority == 0 { None } else { Some(req.priority) };

    s.state.announcement_repository.update(
        UpdateAnnouncementParams {
            id: req.id,
            title,
            content,
            announcement_type,
            priority,
            is_pinned: Some(req.is_pinned),
            is_active: Some(req.is_active),
            start_time: None,
            end_time: None,
        }
    ).await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UpdateAnnouncementResponse { success: true }))
}

pub(crate) async fn delete_announcement(s: &UserServiceImpl, request: Request<DeleteAnnouncementRequest>) -> Result<Response<DeleteAnnouncementResponse>, Status> {
    let req = request.into_inner();
    let success = s.state.announcement_repository.delete(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !success { return Err(Status::not_found("公告不存在")); }
    Ok(Response::new(DeleteAnnouncementResponse { success: true }))
}

pub(crate) async fn list_system_configs(s: &UserServiceImpl, request: Request<ListSystemConfigsRequest>) -> Result<Response<ListSystemConfigsResponse>, Status> {
    let req = request.into_inner();
    let category = if req.category.is_empty() { None } else { Some(req.category.as_str()) };
    let configs = s.state.announcement_repository.list_system_configs(category).await
        .map_err(|e| Status::internal(e.to_string()))?;

    let config_infos: Vec<SystemConfigInfo> = configs.into_iter().map(|c| SystemConfigInfo {
        id: c.id, category: c.category, key: c.config_key, value: c.config_value.unwrap_or_default(),
        r#type: c.value_type, label: c.label, description: c.description.unwrap_or_default(),
        sort: c.sort_order, status: c.status,
        created_at: c.created_at.timestamp(), updated_at: c.updated_at.timestamp(),
    }).collect();

    Ok(Response::new(ListSystemConfigsResponse { configs: config_infos }))
}

pub(crate) async fn update_system_config(s: &UserServiceImpl, request: Request<UpdateSystemConfigRequest>) -> Result<Response<UpdateSystemConfigResponse>, Status> {
    let req = request.into_inner();
    let success = s.state.announcement_repository.update_config(&req.key, &req.value).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !success { return Err(Status::not_found("配置不存在")); }
    Ok(Response::new(UpdateSystemConfigResponse { success: true }))
}

pub(crate) async fn batch_update_system_configs(s: &UserServiceImpl, request: Request<BatchUpdateSystemConfigsRequest>) -> Result<Response<BatchUpdateSystemConfigsResponse>, Status> {
    let req = request.into_inner();
    let pairs: Vec<(String, String)> = req.configs.into_iter().map(|c| (c.key, c.value)).collect();
    s.state.announcement_repository.batch_update_system_configs(&pairs).await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(Response::new(BatchUpdateSystemConfigsResponse { success: true }))
}

pub(crate) async fn get_role_permission_config(s: &UserServiceImpl, request: Request<GetRolePermissionConfigRequest>) -> Result<Response<GetRolePermissionConfigResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let data_rows = s.state.role_repository.get_data_permissions(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    let field_rows = s.state.role_repository.get_field_permissions(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;

    let data_permissions = data_rows.into_iter().map(|row| DataPermissionInfo {
        permission: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        name: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        entity_type: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        scope: row.get("data_scope").and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0),
        custom_scope: vec![],
        filter_group_json: row.get("filter_expression").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
    }).collect();
    let field_permissions = field_rows.into_iter().map(|row| FieldPermissionInfo {
        permission: row.get("permission").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        name: row.get("field_name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        entity_type: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        field: row.get("field_name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        can_view: row.get("permission").and_then(|v| v.as_str()).map(|p| p != "hidden").unwrap_or(true),
        can_edit: row.get("permission").and_then(|v| v.as_str()).map(|p| p == "read_write").unwrap_or(false),
        allowed_fields: vec![],
        denied_fields: vec![],
        read_only: row.get("permission").and_then(|v| v.as_str()).map(|p| p == "read_only").unwrap_or(false),
        description: String::new(),
        sensitive: false,
    }).collect();

    Ok(Response::new(GetRolePermissionConfigResponse { data_permissions, field_permissions }))
}

pub(crate) async fn update_role_permission_config(s: &UserServiceImpl, request: Request<UpdateRolePermissionConfigRequest>) -> Result<Response<UpdateRolePermissionConfigResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let data_json: Vec<serde_json::Value> = req.data_permissions.iter().map(|d| serde_json::json!({
        "resource_type": d.entity_type,
        "data_scope": d.scope.to_string(),
        "filter_expression": if d.filter_group_json.is_empty() { None } else { Some(d.filter_group_json.clone()) },
        "priority": 0,
        "enabled": true,
    })).collect();
    let field_json: Vec<serde_json::Value> = req.field_permissions.iter().map(|f| serde_json::json!({
        "resource_type": f.entity_type,
        "field_name": f.field,
        "permission": if f.can_edit { "read_write" } else if f.can_view { "read_only" } else { "hidden" },
        "mask_pattern": None::<String>,
    })).collect();

    s.state.role_repository.set_data_permissions(role.id, &data_json).await
        .map_err(|e| Status::internal(e.to_string()))?;
    s.state.role_repository.set_field_permissions(role.id, &field_json).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UpdateRolePermissionConfigResponse { success: true }))
}

pub(crate) async fn get_role_data_permissions(s: &UserServiceImpl, request: Request<GetRoleDataPermissionsRequest>) -> Result<Response<GetRoleDataPermissionsResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let data_rows = s.state.role_repository.get_data_permissions(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    let data_permissions = data_rows.into_iter().map(|row| DataPermissionInfo {
        permission: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        name: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        entity_type: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        scope: row.get("data_scope").and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0),
        custom_scope: vec![],
        filter_group_json: row.get("filter_expression").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
    }).collect();

    Ok(Response::new(GetRoleDataPermissionsResponse { data_permissions }))
}

pub(crate) async fn set_role_data_permissions(s: &UserServiceImpl, request: Request<SetRoleDataPermissionsRequest>) -> Result<Response<SetRoleDataPermissionsResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let data_json: Vec<serde_json::Value> = req.data_permissions.iter().map(|d| serde_json::json!({
        "resource_type": d.entity_type,
        "data_scope": d.scope.to_string(),
        "filter_expression": if d.filter_group_json.is_empty() { None } else { Some(d.filter_group_json.clone()) },
        "priority": 0,
        "enabled": true,
    })).collect();
    s.state.role_repository.set_data_permissions(role.id, &data_json).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(SetRoleDataPermissionsResponse { success: true }))
}

pub(crate) async fn get_role_field_permissions(s: &UserServiceImpl, request: Request<GetRoleFieldPermissionsRequest>) -> Result<Response<GetRoleFieldPermissionsResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let field_rows = s.state.role_repository.get_field_permissions(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    let field_permissions = field_rows.into_iter().map(|row| FieldPermissionInfo {
        permission: row.get("permission").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        name: row.get("field_name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        entity_type: row.get("resource_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        field: row.get("field_name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        can_view: row.get("permission").and_then(|v| v.as_str()).map(|p| p != "hidden").unwrap_or(true),
        can_edit: row.get("permission").and_then(|v| v.as_str()).map(|p| p == "read_write").unwrap_or(false),
        allowed_fields: vec![],
        denied_fields: vec![],
        read_only: row.get("permission").and_then(|v| v.as_str()).map(|p| p == "read_only").unwrap_or(false),
        description: String::new(),
        sensitive: false,
    }).collect();

    Ok(Response::new(GetRoleFieldPermissionsResponse { field_permissions }))
}

pub(crate) async fn set_role_field_permissions(s: &UserServiceImpl, request: Request<SetRoleFieldPermissionsRequest>) -> Result<Response<SetRoleFieldPermissionsResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let field_json: Vec<serde_json::Value> = req.field_permissions.iter().map(|f| serde_json::json!({
        "resource_type": f.entity_type,
        "field_name": f.field,
        "permission": if f.can_edit { "read_write" } else if f.can_view { "read_only" } else { "hidden" },
        "mask_pattern": None::<String>,
    })).collect();
    s.state.role_repository.set_field_permissions(role.id, &field_json).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(SetRoleFieldPermissionsResponse { success: true }))
}

pub(crate) async fn get_role_inherit_chain(s: &UserServiceImpl, request: Request<GetRoleInheritChainRequest>) -> Result<Response<GetRoleInheritChainResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    let chain = s.state.role_repository.get_inherit_chain(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    let inherit_from: Vec<String> = chain.iter()
        .filter_map(|row| row.get("parent_role_code").and_then(|v| v.as_str()).map(String::from))
        .collect();
    let permissions = s.state.role_repository.get_permissions(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    let effective_permissions: Vec<String> = permissions.into_iter().map(|p| p.code).collect();

    Ok(Response::new(GetRoleInheritChainResponse {
        info: Some(InheritInfo { role_name: role.name, inherit_from, effective_permissions }),
    }))
}

pub(crate) async fn set_role_inherit(s: &UserServiceImpl, request: Request<SetRoleInheritRequest>) -> Result<Response<SetRoleInheritResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    s.state.role_repository.set_inherit(role.id, &req.inherit_from).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(SetRoleInheritResponse { success: true }))
}

pub(crate) async fn remove_role_inherit(s: &UserServiceImpl, request: Request<RemoveRoleInheritRequest>) -> Result<Response<RemoveRoleInheritResponse>, Status> {
    let req = request.into_inner();
    let role = s.state.role_repository.find_by_code(&req.role_name).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("角色不存在"))?;

    s.state.role_repository.remove_inherit(role.id).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(RemoveRoleInheritResponse { success: true }))
}

pub(crate) async fn get_accessible_departments(_s: &UserServiceImpl, _request: Request<GetAccessibleDepartmentsRequest>) -> Result<Response<GetAccessibleDepartmentsResponse>, Status> {
    Ok(Response::new(GetAccessibleDepartmentsResponse { departments: vec![], total: 0 }))
}

pub(crate) async fn get_accessible_tenants(_s: &UserServiceImpl, _request: Request<GetAccessibleTenantsRequest>) -> Result<Response<GetAccessibleTenantsResponse>, Status> {
    Ok(Response::new(GetAccessibleTenantsResponse { tenants: vec![], total: 0 }))
}

pub(crate) async fn list_permission_definitions(_s: &UserServiceImpl, _request: Request<ListPermissionDefinitionsRequest>) -> Result<Response<ListPermissionDefinitionsResponse>, Status> {
    Ok(Response::new(ListPermissionDefinitionsResponse { permissions: vec![], total: 0 }))
}

pub(crate) async fn create_permission_definition(_s: &UserServiceImpl, _request: Request<CreatePermissionDefinitionRequest>) -> Result<Response<CreatePermissionDefinitionResponse>, Status> {
    Ok(Response::new(CreatePermissionDefinitionResponse { permission: None }))
}

pub(crate) async fn update_permission_definition(_s: &UserServiceImpl, _request: Request<UpdatePermissionDefinitionRequest>) -> Result<Response<UpdatePermissionDefinitionResponse>, Status> {
    Ok(Response::new(UpdatePermissionDefinitionResponse { success: true }))
}

pub(crate) async fn delete_permission_definition(_s: &UserServiceImpl, _request: Request<DeletePermissionDefinitionRequest>) -> Result<Response<DeletePermissionDefinitionResponse>, Status> {
    Ok(Response::new(DeletePermissionDefinitionResponse { success: true }))
}

pub(crate) async fn batch_create_permission_definitions(_s: &UserServiceImpl, _request: Request<BatchCreatePermissionDefinitionsRequest>) -> Result<Response<BatchCreatePermissionDefinitionsResponse>, Status> {
    Ok(Response::new(BatchCreatePermissionDefinitionsResponse { count: 0 }))
}

pub(crate) async fn batch_assign_permissions(_s: &UserServiceImpl, _request: Request<BatchAssignPermissionsRequest>) -> Result<Response<BatchAssignPermissionsResponse>, Status> {
    Ok(Response::new(BatchAssignPermissionsResponse { success: true, affected: 0 }))
}

pub(crate) async fn copy_role_permissions(s: &UserServiceImpl, request: Request<CopyRolePermissionsRequest>) -> Result<Response<CopyRolePermissionsResponse>, Status> {
    let req = request.into_inner();
    let from_role = s.state.role_repository.find_by_code(&req.from_role).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("源角色不存在"))?;
    let to_role = s.state.role_repository.find_by_code(&req.to_role).await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("目标角色不存在"))?;

    s.state.role_repository.copy_permissions(from_role.id, &[to_role.id]).await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(CopyRolePermissionsResponse { success: true }))
}

pub(crate) async fn validate_data_permission(_s: &UserServiceImpl, _request: Request<ValidateDataPermissionRequest>) -> Result<Response<ValidateDataPermissionResponse>, Status> {
    Ok(Response::new(ValidateDataPermissionResponse { allowed: true }))
}

pub(crate) async fn check_sensitive_permission(_s: &UserServiceImpl, _request: Request<CheckSensitivePermissionRequest>) -> Result<Response<CheckSensitivePermissionResponse>, Status> {
    Ok(Response::new(CheckSensitivePermissionResponse { sensitive: false }))
}

pub(crate) async fn list_dictionary_types(s: &UserServiceImpl, request: Request<ListDictionaryTypesRequest>) -> Result<Response<ListDictionaryTypesResponse>, Status> {
    let req = request.into_inner();
    let page = req.page.max(1);
    let page_size = req.page_size.clamp(1, 100);
    let keyword = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
    let status = if req.status == 0 { None } else { Some(req.status) };

    let result = s.state.announcement_repository.list_dictionary_types(page, page_size, keyword, status)
        .await.map_err(|e| Status::internal(e.to_string()))?;

    let types: Vec<DictionaryTypeInfo> = result.types.into_iter().map(|t| DictionaryTypeInfo {
        id: t.id, code: t.code, name: t.name,
        description: t.description.unwrap_or_default(),
        sort: t.sort, status: t.status,
        created_at: t.created_at.timestamp(), updated_at: t.updated_at.timestamp(),
    }).collect();

    Ok(Response::new(ListDictionaryTypesResponse { types, total: result.total }))
}

pub(crate) async fn create_dictionary_type(s: &UserServiceImpl, request: Request<CreateDictionaryTypeRequest>) -> Result<Response<CreateDictionaryTypeResponse>, Status> {
    let req = request.into_inner();
    if req.code.is_empty() || req.name.is_empty() { return Err(Status::invalid_argument("字典类型编码和名称不能为空")); }

    let description = if req.description.is_empty() { None } else { Some(req.description.as_str()) };
    let sort = Some(req.sort);

    let type_id = s.state.announcement_repository.create_dictionary_type(&req.code, &req.name, description, sort)
        .await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(CreateDictionaryTypeResponse {
        r#type: Some(DictionaryTypeInfo {
            id: type_id, code: req.code, name: req.name,
            description: req.description,
            sort: sort.unwrap_or(0), status: 1,
            created_at: chrono::Utc::now().timestamp(), updated_at: chrono::Utc::now().timestamp(),
        }),
    }))
}

pub(crate) async fn update_dictionary_type(s: &UserServiceImpl, request: Request<UpdateDictionaryTypeRequest>) -> Result<Response<UpdateDictionaryTypeResponse>, Status> {
    let req = request.into_inner();
    let existing = s.state.announcement_repository.find_dictionary_type_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if existing.is_none() { return Err(Status::not_found("字典类型不存在")); }

    s.state.announcement_repository.update_dictionary_type(
        req.id,
        Some(req.name).filter(|x| !x.is_empty()),
        Some(req.description).filter(|x| !x.is_empty()),
        Some(req.sort).filter(|&x| x != 0),
        Some(req.status),
    ).await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UpdateDictionaryTypeResponse { success: true }))
}

pub(crate) async fn delete_dictionary_type(s: &UserServiceImpl, request: Request<DeleteDictionaryTypeRequest>) -> Result<Response<DeleteDictionaryTypeResponse>, Status> {
    let req = request.into_inner();
    let success = s.state.announcement_repository.delete_dictionary_type(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !success { return Err(Status::not_found("字典类型不存在")); }
    Ok(Response::new(DeleteDictionaryTypeResponse { success: true }))
}

pub(crate) async fn list_dictionary_items(s: &UserServiceImpl, request: Request<ListDictionaryItemsRequest>) -> Result<Response<ListDictionaryItemsResponse>, Status> {
    let req = request.into_inner();
    let type_id = if req.type_id == 0 { None } else { Some(req.type_id) };
    let type_code = if req.type_code.is_empty() { None } else { Some(req.type_code.as_str()) };
    let keyword = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };
    let status = if req.status == 0 { None } else { Some(req.status) };

    let result = s.state.announcement_repository.list_dictionary_items(type_id, type_code, keyword, status)
        .await.map_err(|e| Status::internal(e.to_string()))?;

    let items: Vec<DictionaryItemInfo> = result.items.into_iter().map(|item| DictionaryItemInfo {
        id: item.id, type_id: item.type_id, label: item.label, value: item.value,
        sort: item.sort, status: item.status, is_default: item.is_default,
        remark: item.remark.unwrap_or_default(),
        created_at: item.created_at.timestamp(), updated_at: item.updated_at.timestamp(),
    }).collect();

    Ok(Response::new(ListDictionaryItemsResponse { items, total: result.total }))
}

pub(crate) async fn create_dictionary_item(s: &UserServiceImpl, request: Request<CreateDictionaryItemRequest>) -> Result<Response<CreateDictionaryItemResponse>, Status> {
    let req = request.into_inner();
    let sort = Some(req.sort);
    let status = Some(req.status);
    let is_default = Some(req.is_default);
    let remark = if req.remark.is_empty() { None } else { Some(req.remark.as_str()) };

    let params = crate::repository::CreateDictionaryItemParams {
        type_id: req.type_id, label: &req.label, value: &req.value,
        sort, status, is_default, remark,
    };

    let item_id = s.state.announcement_repository.create_dictionary_item(params)
        .await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(CreateDictionaryItemResponse {
        item: Some(DictionaryItemInfo {
            id: item_id, type_id: req.type_id, label: req.label, value: req.value,
            sort: sort.unwrap_or(0), status: status.unwrap_or(1), is_default: is_default.unwrap_or(false),
            remark: remark.unwrap_or("").to_string(),
            created_at: chrono::Utc::now().timestamp(), updated_at: chrono::Utc::now().timestamp(),
        }),
    }))
}

pub(crate) async fn update_dictionary_item(s: &UserServiceImpl, request: Request<UpdateDictionaryItemRequest>) -> Result<Response<UpdateDictionaryItemResponse>, Status> {
    let req = request.into_inner();
    let exists = s.state.announcement_repository.find_dictionary_item_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if exists.is_none() { return Err(Status::not_found("字典项不存在")); }

    let label = if req.label.is_empty() { None } else { Some(req.label) };
    let value = if req.value.is_empty() { None } else { Some(req.value) };
    let remark = if req.remark.is_empty() { None } else { Some(req.remark) };
    let sort_val = if req.sort == 0 { None } else { Some(req.sort) };
    let status_val = if req.status == 0 { None } else { Some(req.status) };
    let is_default_val = if req.is_default { Some(true) } else { None };

    let update_params = crate::repository::UpdateDictionaryItemParams {
        id: req.id,
        label: label.as_deref().unwrap_or(""),
        value: value.as_deref().unwrap_or(""),
        sort: sort_val, status: status_val,
        is_default: is_default_val,
        remark: remark.as_deref(),
    };

    s.state.announcement_repository.update_dictionary_item(update_params)
        .await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UpdateDictionaryItemResponse { success: true }))
}

pub(crate) async fn delete_dictionary_item(s: &UserServiceImpl, request: Request<DeleteDictionaryItemRequest>) -> Result<Response<DeleteDictionaryItemResponse>, Status> {
    let req = request.into_inner();
    let success = s.state.announcement_repository.delete_dictionary_item(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    if !success { return Err(Status::not_found("字典项不存在")); }
    Ok(Response::new(DeleteDictionaryItemResponse { success: true }))
}

pub(crate) async fn get_dictionary_type(s: &UserServiceImpl, request: Request<GetDictionaryTypeRequest>) -> Result<Response<GetDictionaryTypeResponse>, Status> {
    let req = request.into_inner();
    let found = s.state.announcement_repository.find_dictionary_type_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    match found {
        Some(t) => Ok(Response::new(GetDictionaryTypeResponse {
            r#type: Some(DictionaryTypeInfo {
                id: t.id, code: t.code, name: t.name,
                description: t.description.unwrap_or_default(),
                sort: t.sort, status: t.status,
                created_at: t.created_at.timestamp(), updated_at: t.updated_at.timestamp(),
            }),
        })),
        None => Err(Status::not_found("字典类型不存在")),
    }
}

pub(crate) async fn get_dictionary_item(s: &UserServiceImpl, request: Request<GetDictionaryItemRequest>) -> Result<Response<GetDictionaryItemResponse>, Status> {
    let req = request.into_inner();
    let found = s.state.announcement_repository.find_dictionary_item_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;
    match found {
        Some(item) => Ok(Response::new(GetDictionaryItemResponse {
            item: Some(DictionaryItemInfo {
                id: item.id, type_id: item.type_id,
                label: item.label, value: item.value,
                sort: item.sort, status: item.status,
                is_default: item.is_default,
                remark: item.remark.unwrap_or_default(),
                created_at: item.created_at.timestamp(),
                updated_at: item.updated_at.timestamp(),
            }),
        })),
        None => Err(Status::not_found("字典项不存在")),
    }
}
