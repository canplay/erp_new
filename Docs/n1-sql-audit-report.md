# user-service N+1 SQL 排查报告

**生成时间:** 2026-09-19
**审查文件数:** 3
**SQL 语句总数:** 67
**N+1 反模式发现数:** 10

## 文件概览

| 文件 | SQL 语句数 | N+1 数 | 风险等级 |
|------|-----------|--------|----------|
| announcement_repository.rs | 29 | 1 | 🔴 高 |
| role_repository.rs | 38 | 7 | 🔴 高 |
| handlers.rs | 0 | 2 | 🔴 高 |

## N+1 反模式详情

### #1 [HIGH] announcement_repository.rs:982-990

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for (key, value) in configs {`

**循环体代码:**
```rust
            sqlx::query!(
                "UPDATE system_configs SET config_value = $1, updated_at = NOW() WHERE config_key = $2" ,
                value,
                key,
            )
            .execute(&mut *tx)
            .await?;
        }
```

**修复建议:** Replace loop queries with single batch query using = ANY($1) or IN clause

**推荐修复代码:**
```rust
```

### #2 [HIGH] role_repository.rs:433-441

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for perm_id in permission_ids {`

**循环体代码:**
```rust
            sqlx::query!(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)" ,
                role_id,
                perm_id,
            )
            .execute(&mut *tx)
            .await?;
        }
```

**修复建议:** Batch INSERT: Use UNNEST or multi-row INSERT VALUES; Example: INSERT INTO table (col1, col2) SELECT * FROM UNNEST($1::type[], $2::type[])

**推荐修复代码:**
```rust
```

### #3 [HIGH] role_repository.rs:534-553

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for target_id in target_role_ids {`

**循环体代码:**
```rust
            // 删除现有权限
            sqlx::query!(
                "DELETE FROM role_permissions WHERE role_id = $1" ,
                target_id,
            )
            .execute(&mut *tx)
            .await?;

            // 复制权限
            for perm_id in &permission_ids {
                sqlx::query!(
                    "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)" ,
                    target_id,
                    perm_id,
                )
... (4 more lines)
```

**修复建议:** Batch INSERT: Use UNNEST or multi-row INSERT VALUES; Example: INSERT INTO table (col1, col2) SELECT * FROM UNNEST($1::type[], $2::type[])

**推荐修复代码:**
```rust
```

### #4 [HIGH] role_repository.rs:544-552

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for perm_id in &permission_ids {`

**循环体代码:**
```rust
                sqlx::query!(
                    "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)" ,
                    target_id,
                    perm_id,
                )
                .execute(&mut *tx)
                .await?;
            }
```

**修复建议:** Batch INSERT: Use UNNEST or multi-row INSERT VALUES; Example: INSERT INTO table (col1, col2) SELECT * FROM UNNEST($1::type[], $2::type[])

**推荐修复代码:**
```rust
```

### #5 [HIGH] role_repository.rs:706-740

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for perm in permissions {`

**循环体代码:**
```rust
            let resource_type = perm
                .get("resource_type" )
                .and_then(|v| v.as_str())
                .unwrap_or("" );
            let data_scope = perm
                .get("data_scope" )
                .and_then(|v| v.as_str())
                .unwrap_or("all" );
            let filter_expression = perm.get("filter_expression" ).and_then(|v| v.as_str());
            let allowed_department_ids = perm.get("allowed_department_ids" );
            let allowed_user_ids = perm.get("allowed_user_ids" );
            let priority = perm.get("priority" ).and_then(serde_json::Value::as_i64).unwrap_or(0) as i32;
            let enabled = perm
                .get("enabled" )
                .and_then(serde_json::Value::as_bool)
... (19 more lines)
```

**修复建议:** Batch INSERT: Use UNNEST or multi-row INSERT VALUES; Example: INSERT INTO table (col1, col2) SELECT * FROM UNNEST($1::type[], $2::type[])

**推荐修复代码:**
```rust
```

### #6 [HIGH] role_repository.rs:794-821

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for perm in permissions {`

**循环体代码:**
```rust
            let resource_type = perm
                .get("resource_type" )
                .and_then(|v| v.as_str())
                .unwrap_or("" );
            let field_name = perm
                .get("field_name" )
                .and_then(|v| v.as_str())
                .unwrap_or("" );
            let permission = perm
                .get("permission" )
                .and_then(|v| v.as_str())
                .unwrap_or("read_write" );
            let mask_pattern = perm.get("mask_pattern" ).and_then(|v| v.as_str());

            sqlx::query!(
... (12 more lines)
```

**修复建议:** Batch INSERT: Use UNNEST or multi-row INSERT VALUES; Example: INSERT INTO table (col1, col2) SELECT * FROM UNNEST($1::type[], $2::type[])

**推荐修复代码:**
```rust
```

### #7 [HIGH] role_repository.rs:882-896

- **类型:** repo_call_in_loop
- **描述:** Repository method call + raw SQL inside for loop (N+1)
- **循环签名:** `for (idx, parent_code) in inherit_from.iter().enumerate() {`

**循环体代码:**
```rust
            if let Some(parent) = self.find_by_code(parent_code).await? {
                sqlx::query!(
                    r"INSERT INTO permission_inheritances
                       (id, parent_role_id, child_role_id, inherit_data_permissions,
                        inherit_field_permissions, override_child_permissions, priority)
                       VALUES (gen_random_uuid()::text, $1, $2, TRUE, TRUE, FALSE, $3)" ,
                    parent.id.to_string(),
                    role_id.to_string(),
                    idx as i32,
                )
                .execute(&self.pool)
                .await?;
            }
        }
```

**修复建议:** Batch repository call: Replace individual find_by_id with batch fetch using WHERE IN or = ANY(ARRAY); Collect IDs: let ids: Vec<i64> = collection.iter().map(|x| x.id).collect();; Single query: repo.find_all_by_ids(&ids) -> Result<Vec<T>, Error>

**推荐修复代码:**
```rust
// Before (N+1):
for code in codes {
    let role = repo.find_by_code(code).await?;
}

// After (批量查询):
let roles = repo.find_all_by_codes(&codes).await?;
```

### #8 [HIGH] role_repository.rs:938-949

- **类型:** select_loop
- **描述:** Raw SQL query! macro inside for loop (N+1)
- **循环签名:** `for code in perms {`

**循环体代码:**
```rust
                let perm_row = sqlx::query!(
                    "SELECT id FROM permissions WHERE code = $1" ,
                    code,
                )
                .fetch_optional(&self.pool)
                .await?;

                if let Some(pr) = perm_row {
                    permission_ids.push(pr.id);
                }
            }
```

**修复建议:** Replace loop queries with single batch query using = ANY($1) or IN clause

**推荐修复代码:**
```rust
```

### #9 [HIGH] handlers.rs:658-675

- **类型:** repo_call_in_loop
- **描述:** Repository method call inside for loop (N+1)
- **循环签名:** `for uid in user_ids {`

**循环体代码:**
```rust
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
... (2 more lines)
```

**修复建议:** Use batch query: Resolve all needed IDs first, then fetch in single query; IDs: let ids: Vec<i64> = user_ids {.iter().map(|x| *x).collect();; Single query: repo.find_all_by_ids(&ids) -> HashMap<i64, T>

**推荐修复代码:**
```rust
// Before (N+1):
for uid in user_ids {
    let user = repo.find_by_id(uid).await?;
    // process user
}

// After (批量查询):
let users = repo.find_all_by_ids(&user_ids).await?;
let user_map: HashMap<i64, User> = users.into_iter().map(|u| (u.id, u)).collect();
for uid in user_ids {
    if let Some(user) = user_map.get(&uid) {
        // process user
    }
}
```

### #10 [HIGH] handlers.rs:917-934

- **类型:** repo_call_in_loop
- **描述:** Repository method call inside for loop (N+1)
- **循环签名:** `for uid in user_ids {`

**循环体代码:**
```rust
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
... (2 more lines)
```

**修复建议:** Use batch query: Resolve all needed IDs first, then fetch in single query; IDs: let ids: Vec<i64> = user_ids {.iter().map(|x| *x).collect();; Single query: repo.find_all_by_ids(&ids) -> HashMap<i64, T>

**推荐修复代码:**
```rust
// Before (N+1):
for uid in user_ids {
    let user = repo.find_by_id(uid).await?;
    // process user
}

// After (批量查询):
let users = repo.find_all_by_ids(&user_ids).await?;
let user_map: HashMap<i64, User> = users.into_iter().map(|u| (u.id, u)).collect();
for uid in user_ids {
    if let Some(user) = user_map.get(&uid) {
        // process user
    }
}
```

## 修复优先级矩阵

| 优先级 | 数量 | 影响 | 修复建议 |
|--------|------|------|----------|
| P0 (立即修复) | 10 | 高并发下 DB 连接耗尽，响应延迟飙升 | 批量查询 + 事务包裹 |
| P1 (本周修复) | 0 | 批量操作时性能下降 | 批量 INSERT/UPDATE |
| P2 (后续优化) | 0 | 代码可维护性 | 重构为批量接口 |

## 整体优化建议

### 1. 批量查询方法补充
- `RoleRepository::find_all_by_codes(&[String]) -> HashMap<String, Role>`
- `RoleRepository::find_all_by_ids(&[i64]) -> HashMap<i64, Role>`
- `UserRepository::find_all_by_ids(&[i64]) -> HashMap<i64, User>`

### 2. 批量 INSERT 模式
- 使用 PostgreSQL UNNEST 语法实现批量插入
- 使用 sqlx::Arguments 动态构建批量参数

### 3. 事务包裹
- 所有批量操作应在事务中执行
- 设置合理的 statement_timeout 和 lock_timeout
