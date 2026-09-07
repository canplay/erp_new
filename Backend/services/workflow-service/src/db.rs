//!
//! 数据库初始化
//! 
//! 创建工作流相关表

use sqlx::PgPool;
use tracing::info;

pub async fn init_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("初始化工作流数据库表...");

    // 工作流定义表
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS workflows (
            id VARCHAR(64) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            definition JSONB NOT NULL DEFAULT '{}',
            status VARCHAR(32) NOT NULL DEFAULT 'draft',
            version INTEGER NOT NULL DEFAULT 1,
            created_by VARCHAR(64) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 工作流节点表
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS workflow_nodes (
            id VARCHAR(64) PRIMARY KEY,
            workflow_id VARCHAR(64) NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            node_type VARCHAR(32) NOT NULL DEFAULT 'task',
            position_x DOUBLE PRECISION NOT NULL DEFAULT 0,
            position_y DOUBLE PRECISION NOT NULL DEFAULT 0,
            config JSONB NOT NULL DEFAULT '{}',
            timeout INTEGER,
            auto_complete BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 工作流边表
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS workflow_edges (
            id VARCHAR(64) PRIMARY KEY,
            workflow_id VARCHAR(64) NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
            source_node_id VARCHAR(64) NOT NULL,
            target_node_id VARCHAR(64) NOT NULL,
            edge_type VARCHAR(32) NOT NULL DEFAULT 'normal',
            condition TEXT,
            label VARCHAR(255),
            priority INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 工作流实例表
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS workflow_instances (
            id VARCHAR(64) PRIMARY KEY,
            workflow_id VARCHAR(64) NOT NULL REFERENCES workflows(id),
            workflow_version INTEGER NOT NULL,
            status VARCHAR(32) NOT NULL DEFAULT 'pending',
            current_node_id VARCHAR(64),
            variables JSONB NOT NULL DEFAULT '{}',
            started_by VARCHAR(64) NOT NULL,
            started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 任务记录表
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS task_records (
            id VARCHAR(64) PRIMARY KEY,
            instance_id VARCHAR(64) NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
            node_id VARCHAR(64) NOT NULL,
            node_name VARCHAR(255) NOT NULL,
            assignee VARCHAR(64) NOT NULL,
            status VARCHAR(32) NOT NULL DEFAULT 'pending',
            comment TEXT,
            form_data JSONB,
            started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ,
            timeout_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 创建索引
    sqlx::raw_sql("CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status)").execute(pool)
        .await?;
    
    sqlx::raw_sql("CREATE INDEX IF NOT EXISTS idx_workflow_nodes_workflow_id ON workflow_nodes(workflow_id)").execute(pool)
        .await?;
    
    sqlx::raw_sql("CREATE INDEX IF NOT EXISTS idx_workflow_edges_workflow_id ON workflow_edges(workflow_id)").execute(pool)
        .await?;
    
    sqlx::raw_sql("CREATE INDEX IF NOT EXISTS idx_workflow_instances_workflow_id ON workflow_instances(workflow_id)").execute(pool)
        .await?;
    
    sqlx::raw_sql("CREATE INDEX IF NOT EXISTS idx_task_records_instance_id ON task_records(instance_id)").execute(pool)
        .await?;
    
    sqlx::raw_sql("CREATE INDEX IF NOT EXISTS idx_task_records_assignee ON task_records(assignee)").execute(pool)
        .await?;

    info!("工作流数据库表初始化完成");
    Ok(())
}
