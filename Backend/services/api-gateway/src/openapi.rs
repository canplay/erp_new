//! `OpenAPI` 文档模块
//!
//! 提供 `OpenAPI` 3.0 规范的 JSON 文档以及 Swagger UI 界面。
//! 文档自动包含所有已注册的 API 路由信息。

use std::sync::Arc;
use axum::{Router, routing::get, response::Html};

use crate::AppState;

/// `OpenAPI` 3.0 规范定义（JSON）
fn openapi_spec() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "MyAI API Gateway",
            "description": "API 网关 - 所有微服务的统一 HTTP 入口\n\n所有 API 路径均以 `/api/*` 为前缀。认证通过 JWT Bearer Token 实现。\n\n## 认证方式\n在请求头中添加 `Authorization: Bearer <token>`。\n\n## 响应格式\n所有 API 返回统一格式：\n```json\n{\n  \"code\": 0,\n  \"message\": \"success\",\n  \"data\": { ... }\n}\n```",
            "version": env!("CARGO_PKG_VERSION"),
            "contact": {
                "name": "MyAI Team"
            }
        },
        "servers": [
            { "url": "http://localhost:8090", "description": "本地开发" },
            { "url": "/", "description": "当前服务" }
        ],
        "paths": {
            // ===== 系统管理 =====
            "/health": {
                "get": { "summary": "健康检查", "description": "简单健康检查，返回 OK", "tags": ["系统"], "responses": { "200": { "description": "OK" } } }
            },
            "/health/detailed": {
                "get": { "summary": "详细健康检查", "description": "返回所有上游服务的连接状态", "tags": ["系统"], "responses": { "200": { "description": "详细健康状态" } } }
            },
            "/health/discovery": {
                "get": { "summary": "服务发现状态", "description": "查看服务发现注册和连接状态", "tags": ["系统"], "responses": { "200": { "description": "服务发现状态" } } }
            },
            "/api/admin/stats": {
                "get": { "summary": "仪表盘统计", "description": "获取系统仪表盘统计数据", "tags": ["系统"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "统计数据" } } }
            },
            "/rate-limit": {
                "get": { "summary": "限流配置", "description": "查看当前限流配置", "tags": ["系统"], "responses": { "200": { "description": "限流配置" } } }
            },
            "/metrics": {
                "get": { "summary": "Prometheus 指标", "description": "以 Prometheus 格式暴露系统指标", "tags": ["系统"], "responses": { "200": { "description": "指标数据" } } }
            },
            "/circuit-breaker/status": {
                "get": { "summary": "熔断器状态", "description": "查看所有服务的熔断器状态", "tags": ["系统"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "熔断器状态列表" } } }
            },

            // ===== 认证服务 =====
            "/api/auth/login": {
                "post": {
                    "summary": "用户登录", "tags": ["认证"],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": {
                            "schema": {
                                "type": "object",
                                "required": ["username", "password"],
                                "properties": {
                                    "username": { "type": "string", "description": "用户名" },
                                    "password": { "type": "string", "description": "密码" }
                                }
                            }
                        }}
                    },
                    "responses": {
                        "200": { "description": "登录成功，返回 token" },
                        "401": { "description": "用户名或密码错误" }
                    }
                }
            },
            "/api/auth/register": {
                "post": {
                    "summary": "用户注册", "tags": ["认证"],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": {
                            "schema": {
                                "type": "object",
                                "required": ["username", "password", "email"],
                                "properties": {
                                    "username": { "type": "string" },
                                    "password": { "type": "string" },
                                    "email": { "type": "string", "format": "email" },
                                    "phone": { "type": "string" },
                                    "nickname": { "type": "string" }
                                }
                            }
                        }}
                    },
                    "responses": { "200": { "description": "注册成功" } }
                }
            },
            "/api/auth/verify": {
                "post": { "summary": "验证 Token", "tags": ["认证"], "responses": { "200": { "description": "Token 验证结果" } } }
            },
            "/api/auth/refresh": {
                "post": { "summary": "刷新 Token", "tags": ["认证"], "responses": { "200": { "description": "新 Token" } } }
            },

            // ===== 用户服务 =====
            "/api/users": {
                "get": {
                    "summary": "用户列表", "tags": ["用户"], "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer", "default": 1 } },
                        { "name": "page_size", "in": "query", "schema": { "type": "integer", "default": 20 } },
                        { "name": "keyword", "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "用户列表" } }
                },
                "post": {
                    "summary": "创建用户", "tags": ["用户"], "security": [{ "bearerAuth": [] }],
                    "requestBody": {
                        "content": { "application/json": {
                            "schema": {
                                "type": "object",
                                "required": ["username", "password"],
                                "properties": {
                                    "username": { "type": "string" },
                                    "password": { "type": "string" },
                                    "email": { "type": "string" },
                                    "nickname": { "type": "string" },
                                    "phone": { "type": "string" },
                                    "gender": { "type": "integer" }
                                }
                            }
                        }}
                    },
                    "responses": { "200": { "description": "创建成功" } }
                }
            },
            "/api/users/{id}": {
                "get": { "summary": "用户详情", "tags": ["用户"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }], "responses": { "200": { "description": "用户信息" } } },
                "put": { "summary": "更新用户", "tags": ["用户"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除用户", "tags": ["用户"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }], "responses": { "200": { "description": "删除成功" } } }
            },

            // ===== CMS 内容管理 =====
            "/api/cms/articles": {
                "get": { "summary": "文章列表", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "parameters": [
                    { "name": "page", "in": "query", "schema": { "type": "integer" } },
                    { "name": "page_size", "in": "query", "schema": { "type": "integer" } },
                    { "name": "category_id", "in": "query", "schema": { "type": "integer" } },
                    { "name": "keyword", "in": "query", "schema": { "type": "string" } },
                    { "name": "status", "in": "query", "schema": { "type": "string" } }
                ], "responses": { "200": { "description": "文章列表" } } },
                "post": { "summary": "创建文章", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },
            "/api/cms/articles/{id}": {
                "get": { "summary": "文章详情", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }], "responses": { "200": { "description": "文章详情" } } },
                "put": { "summary": "更新文章", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除文章", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "删除成功" } } }
            },
            "/api/cms/categories": {
                "get": { "summary": "分类列表", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "分类列表" } } },
                "post": { "summary": "创建分类", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },
            "/api/cms/categories/{id}": {
                "get": { "summary": "分类详情", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }], "responses": { "200": { "description": "分类信息" } } },
                "put": { "summary": "更新分类", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除分类", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "删除成功" } } }
            },
            "/api/cms/tags": {
                "get": { "summary": "标签列表", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "标签列表" } } },
                "post": { "summary": "创建标签", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },
            "/api/cms/tags/{id}": {
                "delete": { "summary": "删除标签", "tags": ["CMS"], "security": [{ "bearerAuth": [] }], "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }], "responses": { "200": { "description": "删除成功" } } }
            },

            // ===== 消息服务 =====
            "/api/messages": {
                "get": { "summary": "消息列表", "tags": ["消息"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "消息列表" } } }
            },
            "/api/messages/unread-count": {
                "get": { "summary": "未读消息数", "tags": ["消息"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "未读数量" } } }
            },
            "/api/messages/send": {
                "post": { "summary": "发送消息", "tags": ["消息"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "发送结果" } } }
            },
            "/api/messages/read": {
                "post": { "summary": "标记已读", "tags": ["消息"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "标记结果" } } }
            },
            "/api/messages/templates": {
                "get": { "summary": "消息模板列表", "tags": ["消息"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "模板列表" } } },
                "post": { "summary": "创建消息模板", "tags": ["消息"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },

            // ===== 文件服务 =====
            "/api/files": {
                "get": { "summary": "文件列表", "tags": ["文件"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "文件列表" } } }
            },
            "/api/files/upload": {
                "post": { "summary": "文件上传", "tags": ["文件"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "上传结果" } } }
            },
            "/api/files/download-url": {
                "post": { "summary": "获取下载链接", "tags": ["文件"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "下载 URL" } } }
            },
            "/api/files/folders": {
                "get": { "summary": "目录列表", "tags": ["文件"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "目录列表" } } },
                "post": { "summary": "创建目录", "tags": ["文件"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },

            // ===== 租户服务 =====
            "/api/tenants": {
                "get": { "summary": "租户列表", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "租户列表" } } },
                "post": { "summary": "创建租户", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },
            "/api/tenants/{id}": {
                "get": { "summary": "租户详情", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "租户信息" } } },
                "put": { "summary": "更新租户", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除租户", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "删除成功" } } }
            },
            "/api/tenants/{id}/users": {
                "get": { "summary": "租户用户列表", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "用户列表" } } },
                "post": { "summary": "添加租户用户", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "添加成功" } } }
            },
            "/api/tenants/{id}/config": {
                "get": { "summary": "租户配置", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "配置信息" } } },
                "put": { "summary": "更新租户配置", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } }
            },
            "/api/tenants/{id}/stats": {
                "get": { "summary": "使用量统计", "tags": ["租户"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "统计数据" } } }
            },

            // ===== 反馈服务 =====
            "/api/feedbacks": {
                "get": { "summary": "反馈列表", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "反馈列表" } } },
                "post": { "summary": "创建反馈", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },
            "/api/feedbacks/{id}": {
                "get": { "summary": "反馈详情", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "反馈信息" } } },
                "put": { "summary": "更新反馈", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除反馈", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "删除成功" } } }
            },
            "/api/feedbacks/{id}/replies": {
                "get": { "summary": "回复列表", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "回复列表" } } },
                "post": { "summary": "添加回复", "tags": ["反馈"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "添加成功" } } }
            },

            // ===== 工作流服务 =====
            "/api/workflows": {
                "get": { "summary": "工作流列表", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "工作流列表" } } },
                "post": { "summary": "创建工作流", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建成功" } } }
            },
            "/api/workflows/{id}": {
                "get": { "summary": "工作流详情", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "工作流信息" } } },
                "put": { "summary": "更新工作流", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除工作流", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "删除成功" } } }
            },
            "/api/workflows/start": {
                "post": { "summary": "启动工作流", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "启动结果" } } }
            },
            "/api/workflows/instances": {
                "get": { "summary": "实例列表", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "实例列表" } } }
            },
            "/api/workflows/tasks": {
                "get": { "summary": "任务列表", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "任务列表" } } }
            },
            "/api/workflows/tasks/{id}/complete": {
                "post": { "summary": "完成任务", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "完成结果" } } }
            },
            "/api/workflows/instances/{id}/cancel": {
                "post": { "summary": "取消实例", "tags": ["工作流"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "取消结果" } } }
            },

            // ===== 审计服务 =====
            "/api/audit/logs": {
                "get": { "summary": "审计日志列表", "tags": ["审计"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "日志列表" } } },
                "post": { "summary": "记录审计日志", "tags": ["审计"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "记录成功" } } }
            },
            "/api/audit/logs/{id}": {
                "get": { "summary": "审计日志详情", "tags": ["审计"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "日志详情" } } }
            },
            "/api/audit/stats": {
                "get": { "summary": "审计统计", "tags": ["审计"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "统计信息" } } }
            },
            "/api/audit/archive": {
                "post": { "summary": "归档审计日志", "tags": ["审计"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "归档结果" } } }
            },

            // ===== API Key 服务 =====
            "/api/api-keys": {
                "get": { "summary": "API Key 列表", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "Key 列表" } } },
                "post": { "summary": "创建 API Key", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "创建结果" } } }
            },
            "/api/api-keys/{id}": {
                "get": { "summary": "API Key 详情", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "Key 信息" } } },
                "put": { "summary": "更新 API Key", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "更新成功" } } },
                "delete": { "summary": "删除 API Key", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "删除成功" } } }
            },
            "/api/api-keys/validate": {
                "post": { "summary": "验证 API Key", "tags": ["API Key"], "responses": { "200": { "description": "验证结果" } } }
            },
            "/api/api-keys/{id}/rotate": {
                "post": { "summary": "轮换 API Key", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "轮换结果" } } }
            },
            "/api/api-keys/{id}/usage": {
                "get": { "summary": "API Key 使用统计", "tags": ["API Key"], "security": [{ "bearerAuth": [] }], "responses": { "200": { "description": "统计数据" } } }
            },

            // ===== 电动自行车 =====
            "/api/v1/ebike/login": {
                "post": { "summary": "ebike 用户登录", "tags": ["电动自行车"], "responses": { "200": { "description": "登录成功" } } }
            },
            "/api/v1/ebike/info": {
                "get": { "summary": "ebike 用户信息", "tags": ["电动自行车"], "responses": { "200": { "description": "用户信息" } } }
            },
            "/api/v1/ebike/car": {
                "post": { "summary": "ebike 车辆管理", "tags": ["电动自行车"], "responses": { "200": { "description": "操作结果" } } }
            },
            "/api/v1/ebike/storage": {
                "post": { "summary": "ebike 仓储管理", "tags": ["电动自行车"], "responses": { "200": { "description": "操作结果" } } }
            },
            "/api/v1/ebike/order": {
                "post": { "summary": "ebike 订单管理", "tags": ["电动自行车"], "responses": { "200": { "description": "操作结果" } } }
            },
            "/api/v1/ebike/options": {
                "post": { "summary": "ebike 系统选项", "tags": ["电动自行车"], "responses": { "200": { "description": "操作结果" } } }
            },

            // ===== WebSocket =====
            "/ws/status": {
                "get": { "summary": "系统状态 WebSocket", "description": "实时推送系统状态更新", "tags": ["WebSocket"], "responses": { "101": { "description": "WebSocket 升级成功" } } }
            },
            "/ws/messages": {
                "get": { "summary": "消息推送 WebSocket", "description": "实时推送消息通知", "tags": ["WebSocket"], "responses": { "101": { "description": "WebSocket 升级成功" } } }
            }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT",
                    "description": "JWT 认证 Token，在请求头中添加 `Authorization: Bearer <token>`"
                }
            }
        }
    })
}

/// `OpenAPI` JSON 接口处理函数
async fn openapi_json_handler() -> axum::Json<serde_json::Value> {
    axum::Json(openapi_spec())
}

/// Swagger UI 页面的 HTML
const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>MyAI API Gateway - Swagger UI</title>
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/5.17.14/swagger-ui.min.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/5.17.14/swagger-ui-bundle.min.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/api/openapi.json',
      dom_id: '#swagger-ui',
      deepLinking: true,
      presets: [
        SwaggerUIBundle.presets.apis,
        SwaggerUIBundle.SwaggerUIStandalonePreset
      ],
      layout: "BaseLayout",
      docExpansion: "list",
      defaultModelExpandDepth: 3,
      defaultModelsExpandDepth: 3
    });
  </script>
</body>
</html>"#;

/// Swagger UI 页面处理函数
async fn swagger_ui_handler() -> Html<&'static str> {
    Html(SWAGGER_UI_HTML)
}

/// 创建 `OpenAPI` 和 Swagger UI 路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/api/openapi.json", get(openapi_json_handler))
        .route("/api/docs", get(swagger_ui_handler))
}
