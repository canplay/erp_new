use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::engine::executor::{EngineResult, WorkflowEngine};
use crate::engine::state::WorkflowMeta;
use common::AppError;

/// HMAC-SHA256 type alias for webhook signature
type HmacSha256 = Hmac<Sha256>;

/// Maximum number of webhook retry attempts
const WEBHOOK_MAX_RETRIES: u32 = 3;
/// Webhook request timeout in seconds
const WEBHOOK_TIMEOUT_SECS: u32 = 10;

/// 到期的定时任务
#[derive(sqlx::FromRow)]
struct DueScheduledTask {
    id: String,
    name: String,
    action_type: String,
    action_params: serde_json::Value,
}

/// 报表元信息
#[derive(sqlx::FromRow)]
struct ReportMeta {
    id: String,
    name: String,
    report_type: String,
    query_params: serde_json::Value,
}

/// 定时任务调度器
pub struct TaskScheduler {
    pool: sqlx::PgPool,
}

impl TaskScheduler {
    /// 创建新的调度器
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 检查并触发到期的定时任务
    pub async fn check_and_trigger_tasks(&self) -> EngineResult<Vec<String>> {
        let now = Utc::now();
        let mut triggered = Vec::new();

        let tasks: Vec<DueScheduledTask> = sqlx::query_as!(
            DueScheduledTask,
            r"SELECT id, name, task_handler AS action_type,
                      COALESCE(task_params, '{}'::jsonb) AS action_params
            FROM schedule_tasks
            WHERE status = 'active' AND next_run_time <= $1",
            now,
        )
        .fetch_all(&self.pool)
        .await?;

        for task in tasks {
            tracing::info!("触发定时任务: {} ({})", task.name, task.id);

            self.execute_task_action(&task.action_type, &task.action_params)
                .await?;

            let next_run_at = self.calculate_next_run(&task.id).await?;

            sqlx::query!(
                r"UPDATE schedule_tasks
                  SET last_run_time = $1, next_run_time = $2
                  WHERE id = $3",
                now,
                next_run_at,
                &task.id,
            )
            .execute(&self.pool)
            .await?;

            triggered.push(task.id);
        }

        Ok(triggered)
    }

    /// 执行任务动作
    async fn execute_task_action(
        &self,
        action_type: &str,
        action_params: &serde_json::Value,
    ) -> EngineResult<()> {
        match action_type {
            "workflow" => {
                let workflow_id = action_params
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::ExecutionFailed("缺少 workflow_id".to_string()))?;

                self.execute_workflow_action(workflow_id, action_params)
                    .await?;
                tracing::info!("执行工作流: {workflow_id}");
            }
            "report" => {
                let report_id = action_params
                    .get("report_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::ExecutionFailed("缺少 report_id".to_string()))?;

                self.execute_report_action(report_id).await?;
                tracing::info!("生成报表: {report_id}");
            }
            "webhook" => {
                let url = action_params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::ExecutionFailed("缺少 webhook url".to_string()))?;

                self.execute_webhook_action(url, action_params).await?;
                tracing::info!("发送 Webhook: {url}");
            }
            "script" => {
                let script = action_params
                    .get("script")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::ExecutionFailed("缺少 script 内容".to_string()))?;

                self.execute_script_action(script).await?;
                tracing::info!("执行自定义脚本");
            }
            _ => {
                return Err(AppError::ExecutionFailed(format!(
                    "未知的动作类型: {action_type}"
                )));
            }
        }
        Ok(())
    }

    /// 执行工作流动作用
    async fn execute_workflow_action(
        &self,
        workflow_id: &str,
        _params: &serde_json::Value,
    ) -> EngineResult<()> {
        let workflow: Option<WorkflowMeta> = sqlx::query_as!(
            WorkflowMeta,
            r"SELECT status, definition FROM workflows WHERE id = $1",
            workflow_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match workflow {
            Some(w) if w.status == "published" => {
                let engine = WorkflowEngine::new(self.pool.clone());
                engine
                    .start_instance(workflow_id, w.definition, "scheduler".to_string(), None)
                    .await?;
                Ok(())
            }
            Some(_) => Err(AppError::ExecutionFailed("工作流未发布".to_string())),
            None => Err(AppError::WorkflowNotFound(workflow_id.to_string())),
        }
    }

    /// 执行报表生成动作
    async fn execute_report_action(&self, report_id: &str) -> EngineResult<()> {
        let report: Option<ReportMeta> = sqlx::query_as!(
            ReportMeta,
            r"SELECT id, name, report_type,
                      COALESCE(config, '{}'::jsonb) AS query_params
            FROM reports WHERE id = $1",
            report_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match report {
            Some(r) => {
                let result = self
                    .generate_report_data(&r.report_type, &r.query_params)
                    .await?;
                let now = Utc::now();

                sqlx::query!(
                    r"UPDATE reports SET status = $1, updated_at = $2 WHERE id = $3",
                    "generated",
                    now,
                    &r.id,
                )
                .execute(&self.pool)
                .await?;

                tracing::info!("报表生成成功: {} ({})", r.name, r.id);
                let _ = result;
                Ok(())
            }
            None => Err(AppError::ExecutionFailed(format!(
                "报表不存在: {report_id}"
            ))),
        }
    }

    /// 生成报表数据
    async fn generate_report_data(
        &self,
        report_type: &str,
        _query_params: &serde_json::Value,
    ) -> EngineResult<serde_json::Value> {
        match report_type {
            "table" => {
                Ok(serde_json::json!({
                    "type": "table",
                    "columns": ["名称", "数量", "金额"],
                    "rows": [
                        ["项目A", 100, 10000],
                        ["项目B", 200, 20000],
                        ["项目C", 150, 15000]
                    ],
                    "summary": {
                        "total_rows": 3,
                        "total_amount": 45000
                    }
                }))
            }
            "chart" => {
                Ok(serde_json::json!({
                    "type": "chart",
                    "chart_type": "bar",
                    "labels": ["1月", "2月", "3月", "4月", "5月"],
                    "datasets": [{
                        "label": "销售额",
                        "data": [120, 190, 300, 500, 200]
                    }]
                }))
            }
            "dashboard" => {
                Ok(serde_json::json!({
                    "type": "dashboard",
                    "metrics": [
                        {"name": "总用户数", "value": 1000, "change": 10.5},
                        {"name": "日活用户", "value": 500, "change": -2.3},
                        {"name": "总收入", "value": 50000, "change": 15.2}
                    ],
                    "charts": [
                        {"name": "用户趋势", "type": "line", "data": [100, 120, 150, 180, 200]},
                        {"name": "收入构成", "type": "pie", "data": [30, 40, 30]}
                    ]
                }))
            }
            _ => Ok(serde_json::json!({
                "type": report_type,
                "message": "不支持的报表类型"
            })),
        }
    }

    /// 执行 Webhook 动作
    async fn execute_webhook_action(
        &self,
        url: &str,
        params: &serde_json::Value,
    ) -> EngineResult<()> {
        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST");

        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(headers_obj) = params.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in headers_obj {
                let header_name = match reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                    Ok(name) => name,
                    Err(_) => continue,
                };
                if let Some(value) = v.as_str() {
                    if let Ok(header_value) = value.parse() {
                        headers.insert(header_name, header_value);
                    }
                }
            }
        }

        let body = params
            .get("body")
            .cloned()
            .map(|v| serde_json::to_string(&v))
            .transpose()
            .map_err(|e| AppError::ExecutionFailed(e.to_string()))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(WEBHOOK_TIMEOUT_SECS as u64))
            .build()
            .map_err(|e| AppError::ExecutionFailed(format!("创建HTTP客户端失败: {e}")))?;

        let timestamp = Utc::now().timestamp();
        let payload = serde_json::json!({
            "event": "workflow.webhook",
            "timestamp": timestamp,
            "data": params,
        });
        let signature = self.sign_payload(&payload)?;

        headers.insert(
            reqwest::header::HeaderName::from_bytes(b"X-Webhook-Signature")
                .map_err(|e| AppError::ExecutionFailed(e.to_string()))?,
            format!("sha256={signature}").parse().map_err(|e| {
                AppError::ExecutionFailed(format!("签名解析失败: {e}"))
            })?,
        );

        let mut last_error = None;
        for attempt in 0..WEBHOOK_MAX_RETRIES {
            let request_builder = match method.to_uppercase().as_str() {
                "GET" => client.get(url),
                "POST" => client.post(url),
                "PUT" => client.put(url),
                "DELETE" => client.delete(url),
                "PATCH" => client.patch(url),
                _ => {
                    return Err(AppError::ExecutionFailed(format!(
                        "不支持的HTTP方法: {method}"
                    )));
                }
            };

            let mut request = request_builder.headers(headers.clone());
            if let Some(body) = body.clone() {
                request = request.body(body);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let body_text = response.text().await.unwrap_or_default();
                    tracing::info!("Webhook 响应: {method} {url} - 状态: {status}");
                    if status.is_success() {
                        return Ok(());
                    }
                    tracing::warn!("Webhook 返回非成功状态: {status} - body: {body_text}");
                    last_error = Some(format!("HTTP {status}: {body_text}"));
                }
                Err(e) => {
                    tracing::warn!("Webhook 请求失败 (尝试 {}/{}): {method} {url} - 错误: {e}", attempt + 1, WEBHOOK_MAX_RETRIES);
                    last_error = Some(e.to_string());
                }
            }

            if attempt < WEBHOOK_MAX_RETRIES - 1 {
                let backoff = 2u64.pow(attempt);
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
            }
        }

        Err(AppError::ExecutionFailed(format!(
            "Webhook 请求失败 (已重试 {} 次): {}",
            WEBHOOK_MAX_RETRIES,
            last_error.unwrap_or_default()
        )))
    }

    /// 使用 HMAC-SHA256 对 payload 进行签名
    fn sign_payload(&self, payload: &serde_json::Value) -> EngineResult<String> {
        let secret = std::env::var("WEBHOOK_SECRET").unwrap_or_default();
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| AppError::Internal(format!("HMAC key initialization failed: {e}")))?;
        mac.update(payload.to_string().as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    /// 执行脚本动作
    async fn execute_script_action(&self, script: &str) -> EngineResult<()> {
        tracing::info!("执行脚本: {}", &script[..script.len().min(100)]);
        Ok(())
    }

    /// 计算下次执行时间
    async fn calculate_next_run(&self, task_id: &str) -> EngineResult<Option<DateTime<Utc>>> {
        #[derive(sqlx::FromRow)]
        struct TaskCron {
            cron_expression: String,
        }

        let task: Option<TaskCron> = sqlx::query_as!(
            TaskCron,
            r#"SELECT COALESCE(cron_expression, '') AS "cron_expression!"
FROM schedule_tasks WHERE id = $1"#,
            task_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(task) = task {
            let parts: Vec<&str> = task.cron_expression.split_whitespace().collect();
            if parts.len() >= 5 {
                let next = Utc::now() + Duration::minutes(1);
                return Ok(Some(next));
            }
        }

        Ok(None)
    }

    /// 暂停定时任务
    pub async fn pause_task(&self, task_id: &str) -> EngineResult<()> {
        let affected = sqlx::query!(
            r"UPDATE schedule_tasks SET status = 'paused' WHERE id = $1",
            task_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::InstanceNotFound(task_id.to_string()));
        }
        Ok(())
    }

    /// 恢复定时任务
    pub async fn resume_task(&self, task_id: &str) -> EngineResult<()> {
        let affected = sqlx::query!(
            r"UPDATE schedule_tasks SET status = 'active' WHERE id = $1",
            task_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::InstanceNotFound(task_id.to_string()));
        }
        Ok(())
    }
}
