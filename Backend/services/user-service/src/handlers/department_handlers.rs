use grpc_proto::user::*;
use tonic::{Request, Response, Status};
use super::UserServiceImpl;

pub(crate) async fn list_departments(s: &UserServiceImpl, request: Request<ListDepartmentsRequest>) -> Result<Response<ListDepartmentsResponse>, Status> {
    let req = request.into_inner();
    let page = req.page.max(1);
    let page_size = req.page_size.clamp(1, 100);
    let keyword = if req.keyword.is_empty() { None } else { Some(req.keyword.as_str()) };

    let result = s.state.department_repository.list(page, page_size, keyword).await
        .map_err(|e| Status::internal(e.to_string()))?;

    let departments: Vec<DepartmentInfo> = result.departments.into_iter().map(|d| DepartmentInfo {
        id: d.id, name: d.name,
        parent_id: d.parent_id.unwrap_or(0),
        description: String::new(),
        sort_order: d.sort_order, status: d.status,
        created_at: d.created_at.timestamp(),
        updated_at: 0,
    }).collect();

    Ok(Response::new(ListDepartmentsResponse { departments, total: result.total }))
}

pub(crate) async fn get_department(s: &UserServiceImpl, request: Request<GetDepartmentRequest>) -> Result<Response<GetDepartmentResponse>, Status> {
    let req = request.into_inner();
    let dept = s.state.department_repository.find_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;

    match dept {
        Some(d) => Ok(Response::new(GetDepartmentResponse {
            department: Some(DepartmentInfo {
                id: d.id, name: d.name,
                parent_id: d.parent_id.unwrap_or(0),
                description: d.description.unwrap_or_default(),
                sort_order: d.sort_order, status: d.status,
                created_at: d.created_at.timestamp(),
                updated_at: d.updated_at.timestamp(),
            }),
        })),
        None => Err(Status::not_found("部门不存在")),
    }
}

pub(crate) async fn create_department(s: &UserServiceImpl, request: Request<CreateDepartmentRequest>) -> Result<Response<CreateDepartmentResponse>, Status> {
    let req = request.into_inner();
    if req.name.is_empty() { return Err(Status::invalid_argument("部门名称不能为空")); }

    let parent_id = if req.parent_id == 0 { None } else { Some(req.parent_id) };
    let description = if req.description.is_empty() { None } else { Some(req.description.as_str()) };
    let sort_order = if req.sort_order == 0 { None } else { Some(req.sort_order) };

    let dept_id = s.state.department_repository.create(&req.name, None, parent_id, None, description, sort_order).await
        .map_err(|e| match e {
            crate::repository::DepartmentRepositoryError::AlreadyExists => Status::already_exists("部门代码已存在"),
            other => Status::internal(format!("{:?}", other)),
        })?;

    Ok(Response::new(CreateDepartmentResponse {
        department: Some(DepartmentInfo {
            id: dept_id, name: req.name,
            parent_id: req.parent_id,
            description: req.description,
            sort_order: sort_order.unwrap_or(0),
            status: 1,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }),
    }))
}

pub(crate) async fn update_department(s: &UserServiceImpl, request: Request<UpdateDepartmentRequest>) -> Result<Response<UpdateDepartmentResponse>, Status> {
    let req = request.into_inner();
    let exists = s.state.department_repository.find_by_id(req.id).await
        .map_err(|e| Status::internal(e.to_string()))?;

    if exists.is_none() { return Err(Status::not_found("部门不存在")); }

    s.state.department_repository.update(
        req.id,
        Some(req.name).filter(|x| !x.is_empty()),
        None,
        Some(req.parent_id).filter(|&id| id != 0),
        None,
        Some(req.description).filter(|x| !x.is_empty()),
        Some(req.sort_order).filter(|&x| x != 0),
        Some(req.status),
    ).await.map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UpdateDepartmentResponse { success: true }))
}

pub(crate) async fn delete_department(s: &UserServiceImpl, request: Request<DeleteDepartmentRequest>) -> Result<Response<DeleteDepartmentResponse>, Status> {
    let req = request.into_inner();
    let result = s.state.department_repository.delete(req.id).await
        .map_err(|e| match e {
            crate::repository::DepartmentRepositoryError::HasChildDepartments => Status::failed_precondition("部门有子部门，无法删除"),
            crate::repository::DepartmentRepositoryError::HasAssociatedUsers => Status::failed_precondition("部门有用户关联，无法删除"),
            other => Status::internal(format!("{:?}", other)),
        })?;

    Ok(Response::new(DeleteDepartmentResponse { success: result }))
}

pub(crate) async fn get_department_tree(s: &UserServiceImpl, _request: Request<GetDepartmentTreeRequest>) -> Result<Response<GetDepartmentTreeResponse>, Status> {
    let tree_nodes = s.state.department_repository.get_tree().await
        .map_err(|e| Status::internal(e.to_string()))?;

    fn flatten_tree(nodes: Vec<crate::repository::DepartmentTreeNode>) -> Vec<DepartmentInfo> {
        let mut result = Vec::new();
        for node in nodes {
            let children = flatten_tree(node.children);
            result.push(DepartmentInfo {
                id: node.id, name: node.name,
                parent_id: node.parent_id.unwrap_or(0),
                description: String::new(),
                sort_order: node.sort_order,
                status: 1, created_at: 0, updated_at: 0,
            });
            result.extend(children);
        }
        result
    }

    let tree = flatten_tree(tree_nodes);
    Ok(Response::new(GetDepartmentTreeResponse { tree }))
}

pub(crate) async fn get_department_users(s: &UserServiceImpl, request: Request<GetDepartmentUsersRequest>) -> Result<Response<GetDepartmentUsersResponse>, Status> {
    let req = request.into_inner();
    let page = req.page.max(1);
    let page_size = req.page_size.clamp(1, 100);

    let (user_ids, total) = s.state.department_repository.get_users(req.department_id, page, page_size).await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut users = Vec::new();
    for uid in user_ids {
        if let Ok(Some(u)) = s.state.user_repository.find_by_id(uid).await {
            users.push(grpc_proto::user::GetUserResponse {
                id: u.id, username: u.username,
                nickname: u.nickname.unwrap_or_default(),
                avatar: u.avatar.unwrap_or_default(),
                phone: u.phone.unwrap_or_default(),
                email: u.email.unwrap_or_default(),
                gender: u.gender.unwrap_or(0),
                address: u.address.unwrap_or_default(),
                role: u.role, status: u.status,
                created_at: u.created_at.timestamp(),
                updated_at: u.updated_at.timestamp(),
            });
        }
    }

    Ok(Response::new(GetDepartmentUsersResponse { users, total }))
}
