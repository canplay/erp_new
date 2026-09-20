//! 部门仓储层

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

/// 部门仓储错误类型
#[derive(Error, Debug)]
pub(crate) enum DepartmentRepositoryError {
    #[error("数据库错误: {0}" )]
    Database(#[from] sqlx::Error),

    #[error("部门已存在" )]
    AlreadyExists,

    #[error("部门有用户关联，无法删除" )]
    HasAssociatedUsers,

    #[error("部门有子部门，无法删除" )]
    HasChildDepartments,

    #[error("不能将自己设置为父部门" )]
    CircularReference,

    #[error("部门层级不能超过5级" )]
    MaxLevelExceeded,
}

/// 更新部门参数
#[derive(Debug, Clone)]
pub(crate) struct UpdateDepartmentParams {
    pub dept_id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub leader_id: Option<i64>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
}

/// 部门信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Department {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub leader_id: Option<i64>,
    pub leader_name: Option<String>,
    pub description: Option<String>,
    pub status: i32,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// 部门列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DepartmentListItem {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub leader_id: Option<i64>,
    pub leader_name: Option<String>,
    pub user_count: i64,
    pub status: i32,
    pub created_at: chrono::DateTime<Utc>,
}

/// 部门树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DepartmentTreeNode {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub leader_id: Option<i64>,
    pub leader_name: Option<String>,
    pub user_count: i64,
    pub children: Vec<Self>,
}

/// 分页结果
pub(crate) struct PaginatedDepartments {
    pub departments: Vec<DepartmentListItem>,
    pub total: i64,
}

/// 部门仓储
#[derive(Clone)]
pub(crate) struct DepartmentRepository {
    pool: PgPool,
}

impl DepartmentRepository {
    /// 创建新的部门仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建部门
    pub(crate) async fn create(
        &self,
        name: &str,
        code: Option<&str>,
        parent_id: Option<i64>,
        leader_id: Option<i64>,
        description: Option<&str>,
        sort_order: Option<i32>,
    ) -> Result<i64, DepartmentRepositoryError> {
        if let Some(c) = code {
            let exists = sqlx::query!(
                r##"SELECT EXISTS(SELECT 1 FROM departments WHERE code = $1) AS "exists!" "##,
                c
            )
            .fetch_one(&self.pool)
            .await?
            .exists;

            if exists {
                return Err(DepartmentRepositoryError::AlreadyExists);
            }
        }

        let level = if let Some(pid) = parent_id {
            let row = sqlx::query!(
                "SELECT level FROM departments WHERE id = $1" ,
                pid
            )
            .fetch_optional(&self.pool)
            .await?;

            match row {
                Some(r) => r.level.unwrap_or(0) + 1,
                None => 0,
            }
        } else {
            0
        };

        if level > 5 {
            return Err(DepartmentRepositoryError::MaxLevelExceeded);
        }

        let row = sqlx::query!(
            r"INSERT INTO departments (name, code, parent_id, level, leader_id, description, sort_order, status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 1)
               RETURNING id" ,
            name,
            code,
            parent_id,
            level,
            leader_id,
            description,
            sort_order.unwrap_or(0),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 根据ID查找部门
    pub(crate) async fn find_by_id(
        &self,
        dept_id: i64,
    ) -> Result<Option<Department>, DepartmentRepositoryError> {
        let row = sqlx::query_as!(
            Department,
            r##"SELECT d.id, d.name, d.code, d.parent_id,
                      COALESCE(d.level, 0) AS "level!" ,
                      COALESCE(d.sort_order, 0) AS "sort_order!" ,
                      d.leader_id, d.description,
                      COALESCE(d.status, 1) AS "status!" ,
                      COALESCE(d.created_at, NOW()) AS "created_at!" ,
                      COALESCE(d.updated_at, NOW()) AS "updated_at!" ,
                      u.nickname as leader_name
               FROM departments d
               LEFT JOIN users u ON d.leader_id = u.id
               WHERE d.id = $1"##,
            dept_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Department {
            id: r.id,
            name: r.name,
            code: r.code,
            parent_id: r.parent_id,
            level: r.level,
            sort_order: r.sort_order,
            leader_id: r.leader_id,
            leader_name: r.leader_name,
            description: r.description,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 根据代码查找部门
    
    /// 更新部门
    pub(crate) async fn update(
        &self,
        params: UpdateDepartmentParams,
    ) -> Result<Option<Department>, DepartmentRepositoryError> {
        if let Some(new_parent_id) = params.parent_id {
            if new_parent_id == params.dept_id {
                return Err(DepartmentRepositoryError::CircularReference);
            }

            if self.is_descendant(new_parent_id, params.dept_id).await? {
                return Err(DepartmentRepositoryError::CircularReference);
            }
        }

        let level = if let Some(pid) = params.parent_id {
            let row = sqlx::query!(
                "SELECT level FROM departments WHERE id = $1" ,
                pid
            )
            .fetch_optional(&self.pool)
            .await?;

            match row {
                Some(r) => r.level.unwrap_or(0) + 1,
                None => 0,
            }
        } else {
            0
        };

        if level > 5 {
            return Err(DepartmentRepositoryError::MaxLevelExceeded);
        }

        let row = sqlx::query!(
            r##"UPDATE departments
               SET name = COALESCE($1, name),
                   code = COALESCE($2, code),
                   parent_id = $3,
                   level = $4,
                   leader_id = $5,
                   description = COALESCE($6, description),
                   sort_order = COALESCE($7, sort_order),
                   status = COALESCE($8, status),
                   updated_at = NOW()
               WHERE id = $9
               RETURNING id, name, code, parent_id,
                          COALESCE(level, 0) AS "level!" ,
                          COALESCE(sort_order, 0) AS "sort_order!" ,
                          leader_id, description,
                          COALESCE(status, 1) AS "status!" ,
                          COALESCE(created_at, NOW()) AS "created_at!" ,
                          COALESCE(updated_at, NOW()) AS "updated_at!"
"##,
            params.name.as_deref(),
            params.code.as_deref(),
            params.parent_id,
            level,
            params.leader_id,
            params.description.as_deref(),
            params.sort_order,
            params.status,
            params.dept_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Department {
            id: r.id,
            name: r.name,
            code: r.code,
            parent_id: r.parent_id,
            level: r.level,
            sort_order: r.sort_order,
            leader_id: r.leader_id,
            leader_name: None,
            description: r.description,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 检查是否是自己的后代
    async fn is_descendant(
        &self,
        potential_descendant: i64,
        ancestor: i64,
    ) -> Result<bool, DepartmentRepositoryError> {
        let mut current_id = Some(potential_descendant);

        while let Some(id) = current_id {
            let row = sqlx::query!(
                "SELECT parent_id FROM departments WHERE id = $1" ,
                id
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(r) = row {
                let parent_id: Option<i64> = r.parent_id;
                if parent_id == Some(ancestor) {
                    return Ok(true);
                }
                current_id = parent_id;
            } else {
                break;
            }
        }

        Ok(false)
    }

    /// 删除部门
    pub(crate) async fn delete(&self, dept_id: i64) -> Result<bool, DepartmentRepositoryError> {
        let child_count =
            sqlx::query!("SELECT COUNT(*) as count FROM departments WHERE parent_id = $1" , dept_id)
                .fetch_one(&self.pool)
                .await?
                .count
                .unwrap_or(0);

        if child_count > 0 {
            return Err(DepartmentRepositoryError::HasChildDepartments);
        }

        let user_count =
            sqlx::query!("SELECT COUNT(*) as count FROM user_departments WHERE department_id = $1" , dept_id)
                .fetch_one(&self.pool)
                .await?
                .count
                .unwrap_or(0);

        if user_count > 0 {
            return Err(DepartmentRepositoryError::HasAssociatedUsers);
        }

        let result = sqlx::query!("DELETE FROM departments WHERE id = $1" , dept_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页查询部门列表
    pub(crate) async fn list(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
    ) -> Result<PaginatedDepartments, DepartmentRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            DepartmentListItem,
            r##"SELECT d.id, d.name, d.code, d.parent_id,
                      COALESCE(d.level, 0) AS "level!" ,
                      COALESCE(d.sort_order, 0) AS "sort_order!" ,
                      d.leader_id,
                      COALESCE(d.status, 1) AS "status!" ,
                      COALESCE(d.created_at, NOW()) AS "created_at!" ,
                      u.nickname as leader_name,
                      COUNT(ud.id) AS "user_count!"
               FROM departments d
               LEFT JOIN users u ON d.leader_id = u.id
               LEFT JOIN user_departments ud ON d.id = ud.department_id
               WHERE ($1::text IS NULL OR d.name ILIKE '%' || $1 || '%' OR d.code ILIKE '%' || $1 || '%')
               GROUP BY d.id, d.name, d.code, d.parent_id, d.level, d.sort_order,
                        d.leader_id, d.status, d.created_at, u.nickname
               ORDER BY d.sort_order, d.created_at DESC
               LIMIT $2 OFFSET $3"##,
            keyword,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!(
            r"SELECT COUNT(*) as count FROM departments
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')" ,
            keyword,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let departments: Vec<DepartmentListItem> = rows
            .into_iter()
            .map(|row| DepartmentListItem {
                id: row.id,
                name: row.name,
                code: row.code,
                parent_id: row.parent_id,
                level: row.level,
                sort_order: row.sort_order,
                leader_id: row.leader_id,
                leader_name: row.leader_name,
                user_count: row.user_count,
                status: row.status,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedDepartments { departments, total })
    }

    /// 获取部门树
    pub(crate) async fn get_tree(&self) -> Result<Vec<DepartmentTreeNode>, DepartmentRepositoryError> {
        let rows = sqlx::query!(
            r##"SELECT d.id, d.name, d.code, d.parent_id,
                      COALESCE(d.level, 0) AS "level!" ,
                      COALESCE(d.sort_order, 0) AS "sort_order!" ,
                      d.leader_id,
                      u.nickname as leader_name,
                      COUNT(ud.id) AS "user_count!"
               FROM departments d
               LEFT JOIN users u ON d.leader_id = u.id
               LEFT JOIN user_departments ud ON d.id = ud.department_id
               WHERE d.status = 1
               GROUP BY d.id, d.name, d.code, d.parent_id, d.level, d.sort_order,
                        d.leader_id, u.nickname
               ORDER BY d.level, d.sort_order"##,
        )
        .fetch_all(&self.pool)
        .await?;

        let all_nodes: Vec<DepartmentTreeNode> = rows
            .into_iter()
            .map(|row| DepartmentTreeNode {
                id: row.id,
                name: row.name,
                code: row.code,
                parent_id: row.parent_id,
                level: row.level,
                sort_order: row.sort_order,
                leader_id: row.leader_id,
                leader_name: row.leader_name,
                user_count: row.user_count,
                children: Vec::new(),
            })
            .collect();

        let mut node_map: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
        for (idx, node) in all_nodes.iter().enumerate() {
            node_map.insert(node.id, idx);
        }

        // 收集根节点
        let root_ids: Vec<i64> = all_nodes
            .iter()
            .filter(|n| n.parent_id.is_none())
            .map(|n| n.id)
            .collect();

        // 构建树
        self.build_tree_recursive(&root_ids, &node_map, &all_nodes)
            .await
    }

    /// 递归构建部门树
    async fn build_tree_recursive(
        &self,
        parent_ids: &[i64],
        node_map: &std::collections::HashMap<i64, usize>,
        all_nodes: &[DepartmentTreeNode],
    ) -> Result<Vec<DepartmentTreeNode>, DepartmentRepositoryError> {
        let mut result = Vec::new();

        for parent_id in parent_ids {
            if let Some(&idx) = node_map.get(parent_id) {
                let mut node = all_nodes[idx].clone();

                let child_rows = sqlx::query!(
                    "SELECT id FROM departments WHERE parent_id = $1 AND status = 1 ORDER BY sort_order" ,
                    parent_id
                )
                .fetch_all(&self.pool)
                .await?;

                let child_ids: Vec<i64> = child_rows.into_iter().map(|row| row.id).collect();

                if !child_ids.is_empty() {
                    node.children =
                        Box::pin(self.build_tree_recursive(&child_ids, node_map, all_nodes))
                            .await?;
                }

                result.push(node);
            }
        }

        Ok(result)
    }

    /// 获取部门下的用户列表
    pub(crate) async fn get_users(
        &self,
        dept_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<i64>, i64), DepartmentRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query!(
            "SELECT user_id FROM user_departments WHERE department_id = $1\n               ORDER BY is_primary DESC, created_at DESC\n               LIMIT $2 OFFSET $3" ,
            dept_id,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row =
            sqlx::query!("SELECT COUNT(*) as count FROM user_departments WHERE department_id = $1" , dept_id)
                .fetch_one(&self.pool)
                .await?;

        let user_ids: Vec<i64> = rows.into_iter().map(|row| row.user_id).collect();
        let total = total_row.count.unwrap_or(0);

        Ok((user_ids, total))
    }

    }
