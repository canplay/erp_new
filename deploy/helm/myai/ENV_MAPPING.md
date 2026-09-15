# 环境变量映射说明

本文档说明 `.env.example` 中的环境变量如何映射到 Helm Chart 配置。

## 映射关系

### 1. 基础设施配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `POSTGRES_DB` | `infrastructure.postgres.env.POSTGRES_DB` | PostgreSQL 数据库名 |
| `POSTGRES_USER` | `infrastructure.postgres.env.POSTGRES_USER` | PostgreSQL 用户名 |
| `POSTGRES_PASSWORD` | `secrets.yaml` → `secret.database.POSTGRES_PASSWORD` | PostgreSQL 密码（敏感） |
| `DATABASE_URL` | `secrets.yaml` → `secret.database.DATABASE_URL` | 数据库连接字符串（敏感） |
| `REDIS_URL` | `secrets.yaml` → `secret.redis.REDIS_URL` | Redis 连接字符串（敏感） |
| `REDIS_PASSWORD` | `secrets.yaml` → `secret.redis.RED.REDIS_PASSWORD` | Redis 密码（敏感） |

### 2. 日志配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `RUST_LOG` | `configMap.logging.RUST_LOG` | 日志级别 |
| `LOG_FORMAT` | `configMap.logging.LOG_FORMAT` | 日志格式 |
| `LOG_MAX_SIZE` | `configMap.logging.LOG_MAX_SIZE` | 日志文件最大大小 |
| `LOG_MAX_FILES` | `configMap.logging.LOG_MAX_FILES` | 日志文件保留数量 |

### 3. OpenTelemetry 配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `OTEL_ENABLED` | `configMap.otel.OTEL_ENABLED` | 是否启用 OpenTelemetry |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `otel.exporterOtlpEndpoint` | OTLP 端点 |
| `OTEL_SERVICE_NAME` | `configMap.otel.OTEL_SERVICE_NAME` | 服务名称 |

### 4. API Gateway 配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `API_GATEWAY_PORT` | `services.apiGateway.ports.http` | HTTP 端口 |
| `RATE_LIMIT_MAX` | `services.apiGateway.env.RATE_LIMIT_MAX` | 限流最大请求数 |
| `RATE_LIMIT_WINDOW` | `services.apiGateway.env.RATE_LIMIT_WINDOW` | 限流时间窗口 |
| `RATE_LIMIT_BURST` | `services.apiGateway.env.RATE_LIMIT_BURST` | 限流突发请求数 |
| `CIRCUIT_BREAKER_FAILURE_THRESHOLD` | `services.apiGateway.env.CIRCUIT_BREAKER_FAILURE_THRESHOLD` | 熔断器失败阈值 |
| `CIRCUIT_BREAKER_TIMEOUT_SECS` | `services.apiGateway.env.CIRCUIT_BREAKER_TIMEOUT_SECS` | 熔断器超时时间 |
| `CIRCUIT_BREAKER_HALF_OPEN_REQUESTS` | `services.apiGateway.env.CIRCUIT_BREAKER_HALF_OPEN_REQUESTS` | 熔断器半开请求数 |
| `CORS_ALLOWED_ORIGINS` | `services.apiGateway.env.CORS_ALLOWED_ORIGINS` | CORS 允许的源 |
| `CORS_ALLOWED_METHODS` | `services.apiGateway.env.CORS_ALLOWED_METHODS` | CORS 允许的方法 |
| `CORS_ALLOWED_HEADERS` | `services.apiGateway.env.CORS_ALLOWED_HEADERS` | CORS 允许的请求头 |
| `CORS_MAX_AGE` | `services.apiGateway.env.CORS_MAX_AGE` | CORS 缓存时间 |

### 5. 认证配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `JWT_SECRET` | `secrets.yaml` → `secret.jwt.JWT_SECRET` | JWT 密钥（敏感） |
| `JWT_ISSUER` | `secrets.yaml` → `secret.jwt.JWT_ISSUER` | JWT 签发者 |
| `JWT_AUDIENCE` | `secrets.yaml` → `secret.jwt.JWT_AUDIENCE` | JWT 受众 |
| `JWT_ACCESS_EXPIRY` | `secrets.yaml` → `secret.jwt.JWT_ACCESS_EXPIRY` | JWT 访问令牌有效期 |
| `JWT_REFRESH_EXPIRY` | `secrets.yaml` → `secret.jwt.JWT_REFRESH_EXPIRY` | JWT 刷新令牌有效期 |
| `SESSION_MAX_LOGINS` | `services.authService.env.SESSION_MAX_LOGINS` | 最大并发登录数 |
| `SESSION_TIMEOUT_SECS` | `services.authService.env.SESSION_TIMEOUT_SECS` | 会话超时时间 |
| `DEVICE_LIMIT` | `services.authService.env.DEVICE_LIMIT` | 每用户最大设备数 |
| `PASSWORD_MIN_LENGTH` | `services.authService.env.PASSWORD_MIN_LENGTH` | 密码最小长度 |
| `LOGIN_MAX_ATTEMPTS` | `services.authService.env.LOGIN_MAX_ATTEMPTS` | 最大登录尝试次数 |
| `LOGIN_LOCKOUT_SECS` | `services.authService.env.LOGIN_LOCKOUT_SECS` | 登录锁定时长 |

### 6. 上游微服务 gRPC 地址

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `AUTH_SERVICE_GRPC_URL` | `services.apiGateway.env.AUTH_SERVICE_GRPC_URL` | 认证服务 gRPC 地址 |
| `USER_SERVICE_GRPC_URL` | `services.apiGateway.env.USER_SERVICE_GRPC_URL` | 用户服务 gRPC 地址 |
| `CMS_SERVICE_GRPC_URL` | `services.apiGateway.env.CMS_SERVICE_GRPC_URL` | CMS 服务 gRPC 地址 |
| `MESSAGE_SERVICE_GRPC_URL` | `services.apiGateway.env.MESSAGE_SERVICE_GRPC_URL` | 消息服务 gRPC 地址 |
| `FILE_SERVICE_GRPC_URL` | `services.apiGateway.env.FILE_SERVICE_GRPC_URL` | 文件服务 gRPC 地址 |
| `TENANT_SERVICE_GRPC_URL` | `services.apiGateway.env.TENANT_SERVICE_GRPC_URL` | 租户服务 gRPC 地址 |
| `WORKFLOW_SERVICE_GRPC_URL` | `services.apiGateway.env.WORKFLOW_SERVICE_GRPC_URL` | 工作流服务 gRPC 地址 |
| `AUDIT_SERVICE_GRPC_URL` | `services.apiGateway.env.AUDIT_SERVICE_GRPC_URL` | 审计服务 gRPC 地址 |
| `API_KEY_SERVICE_GRPC_URL` | `services.apiGateway.env.API_KEY_SERVICE_GRPC_URL` | API Key 服务 gRPC 地址 |
| `FEEDBACK_SERVICE_GRPC_URL` | `services.apiGateway.env.FEEDBACK_SERVICE_GRPC_URL` | 反馈服务 gRPC 地址 |
| `HIK_SERVICE_GRPC_URL` | `services.apiGateway.env.HIK_SERVICE_GRPC_URL` | 海康服务 gRPC 地址 |
| `PAY_SERVICE_GRPC_URL` | `services.apiGateway.env.PAY_SERVICE_GRPC_URL` | 支付服务 gRPC 地址 |
| `CLEAN_SERVICE_GRPC_URL` | `services.apiGateway.env.CLEAN_SERVICE_GRPC_URL` | 清洁服务 gRPC 地址 |
| `TOW_SERVICE_GRPC_URL` | `services.apiGateway.env.TOW_SERVICE_GRPC_URL` | 拖车服务 gRPC 地址 |

### 7. 文件存储配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `FILE_STORAGE_TYPE` | `services.fileService.env.FILE_STORAGE_TYPE` | 存储类型 |
| `FILE_STORAGE_PATH` | `services.fileService.env.FILE_STORAGE_PATH` | 本地存储路径 |
| `S3_ENDPOINT` | `services.fileService.env.S3_ENDPOINT` | S3 端点 |
| `S3_REGION` | `services.fileService.env.S3_REGION` | S3 区域 |
| `S3_BUCKET` | `services.fileService.env.S3_BUCKET` | S3 存储桶 |
| `S3_ACCESS_KEY` | `secrets.yaml` → `secret.s3.S3_ACCESS_KEY` | S3 访问密钥（敏感） |
| `S3_SECRET_KEY` | `secrets.yaml` → `secret.s3.S3_SECRET_KEY` | S3 密钥（敏感） |
| `S3_USE_PATH_STYLE` | `services.fileService.env.S3_USE_PATH_STYLE` | S3 路径样式 |
| `OSS_ENDPOINT` | `services.fileService.env.OSS_ENDPOINT` | OSS 端点 |
| `OSS_BUCKET` | `services.fileService.env.OSS_BUCKET` | OSS 存储桶 |
| `OSS_ACCESS_KEY_ID` | `secrets.yaml` → `secret.oss.OSS_ACCESS_KEY_ID` | OSS 访问密钥 ID（敏感） |
| `OSS_ACCESS_KEY_SECRET` | `secrets.yaml` → `secret.oss.OSS_ACCESS_KEY_SECRET` | OSS 密钥（敏感） |

### 8. 工作流配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `WORKFLOW_SCRIPT_ENABLED` | `services.workflowService.env.WORKFLOW_SCRIPT_ENABLED` | 是否启用脚本 |
| `WORKFLOW_SCRIPT_TIMEOUT_SECS` | `services.workflowService.env.WORKFLOW_SCRIPT_TIMEOUT_SECS` | 脚本超时时间 |
| `WORKFLOW_MAX_EXECUTION_TIME_SECS` | `services.workflowService.env.WORKFLOW_MAX_EXECUTION_TIME_SECS` | 最大执行时间 |
| `WORKFLOW_CRON_CHECK_INTERVAL_SECS` | `services.workflowService.env.WORKFLOW_CRON_CHECK_INTERVAL_SECS` | Cron 检查间隔 |

### 9. 支付配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `PAY_ENV` | `services.payService.env.PAY_ENV` | 支付环境 |
| `CCB_MERCHANT_ID` | `secrets.yaml` → `secret.pay.CCB_MERCHANT_ID` | 建行商户 ID（敏感） |
| `CCB_MERCHANT_NAME` | `secrets.yaml` → `secret.pay.CCB_MERCHANT_NAME` | 建行商户名称（敏感） |
| `CCB_TERMINAL_ID` | `secrets.yaml` → `secret.pay.CCB_TERMINAL_ID` | 建行终端 ID（敏感） |
| `CCB_KEY` | `secrets.yaml` → `secret.pay.CCB_KEY` | 建行密钥（敏感） |
| `CCB_CERT` | `secrets.yaml` → `secret.pay.CCB_CERT` | 建行证书（敏感） |
| `CCB_SIGN_TYPE` | `services.payService.env.CCB_SIGN_TYPE` | 建行签名类型 |
| `CCB_GATEWAY_URL` | `secrets.yaml` → `secret.pay.CCB_GATEWAY_URL` | 建行网关 URL（敏感） |
| `UMS_MERCHANT_ID` | `secrets.yaml` → `secret.pay.UMS_MERCHANT_ID` | 银联商户 ID（敏感） |
| `UMS_TERMINAL_ID` | `secrets.yaml` → `secret.pay.UMS_TERMINAL_ID` | 银联终端 ID（敏感） |
| `UMS_KEY` | `secrets.yaml` → `secret.pay.UMS_KEY` | 银联密钥（敏感） |
| `UMS_GATEWAY_URL` | `secrets.yaml` → `secret.pay.UMS_GATEWAY_URL` | 银联网关 URL（敏感） |
| `UMS_SIGN_TYPE` | `services.payService.env.UMS_SIGN_TYPE` | 银联签名类型 |
| `PAY_NOTIFY_URL` | `secrets.yaml` → `secret.pay.PAY_NOTIFY_URL` | 支付通知 URL（敏感） |
| `PAY_RETURN_URL` | `secrets.yaml` → `secret.pay.PAY_RETURN_URL` | 支付返回 URL（敏感） |

### 10. 海康配置

| .env.example | Helm Chart 配置文件 | 说明 |
|-------------|-------------------|------|
| `HIK_URL` | `secrets.yaml` → `secret.hik.HIK_URL` | 海康 URL（敏感） |
| `HIK_CLIENT_ID` | `secrets.yaml` → `secret.hik.HIK_CLIENT_ID` | 海康客户端 ID（敏感） |
| `HIK_CLIENT_SECRET` | `secrets.yaml` → `secret.hik.HIK_CLIENT_SECRET` | 海康客户端密钥（敏感） |
| `HIK_APP_KEY` | `secrets.yaml` → `secret.hik.HIK_APP_KEY` | 海康应用密钥（敏感） |
| `HIK_APP_SECRET` | `secrets.yaml` → `secret.hik.HIK_APP_SECRET` | 海康应用密钥（敏感） |

## 配置迁移指南

### 从 .env 迁移到 Helm Chart

1. **复制 secrets.example.yaml 为 secrets.yaml**
   ```bash
   cp helm/myai/secrets.example.yaml helm/myai/secret.yaml
   ```

2. **编辑 secret.yaml，填入敏感信息**
   ```bash
   vim helm/myai/secret.yaml
   ```

3. **根据需要修改 values.yaml**
   - 非敏感配置（日志级别、限流、连接池等）在 `values.yaml` 中
   - 生产配置在 `values-prod-harbor.yaml`（推荐）

4. **部署时使用配置文件**
   ```bash
   helm upgrade --install myai ./helm/myai \
     --namespace myai-prod \
     -f ./helm/myai/values-prod-harbor.yaml \
     --set image.tag=YYYYMMDD.N
   ```

## 注意事项

1. **敏感信息**：所有标记为"敏感"的配置必须放在 `secret.yaml` 中
2. **不要提交**：`secret.yaml` 已添加到 `.gitignore`，不会提交到版本控制
3. **环境差异**：不同环境使用不同的 values 文件（`values-dev.yaml`、`values-prod.yaml`）
4. **服务发现**：在 Kubernetes 中，服务间通信使用服务名（如 `auth-service:9091`）
5. **配置优先级**：Helm 会合并多个 values 文件，后指定的文件会覆盖前面的配置
6. **Secret 变更**：需 rollout restart（运行中 Pod 不自动刷新 envFrom）


## 9. 运维脚本层环境变量 (Scripts/.env, 2026-08-11 新增)

由 `Scripts/lib/env.sh` 加载, 优先级: 已存在环境变量 > Scripts/.env > 脚本默认值。

| 变量 | 脚本默认值 | 说明 |
|------|-----------|------|
| `HARBOR_URL` | `harbor.100.100.100.101.example.com:8003` | Harbor 仓库地址 |
| `HARBOR_PROJECT` | `erp` | Harbor 项目 |
| `HARBOR_USER` | `admin` | Harbor 用户 |
| `RANCHER_URL` | `https://100.100.100.101.example.com:8800` | Rancher 地址 |
| `RANCHER_USER` | `admin` | Rancher 用户 |
| `NAMESPACE` | `myai-prod` | K8s 命名空间 |
| `MYAI_ROOT` | (自动搜索) | 项目根路径覆盖 |
| `MYAI_ENV_FILE` | `Scripts/.env` | .env 路径覆盖 |
