# erp_new · 全栈微服务管理平台

> 基于 Rust + Vue 3 + Quasar 的全栈微服务管理平台，采用 Service → gRPC → Gateway → HTTP 的架构，可部署至 K8s（Rancher 管理）。

## 1. 项目概览

| 维度 | 数据 |
|---|---|
| 后端 | 22 微服务 / 10 共享 Crates / ~340 Rust 源文件 |
| 前端 | 4 应用（admin 49 / tenant / ops / social）/ 10 共享包 / 199 Vue 组件 |
| 数据库 | PostgreSQL 16+ / 119 张表 |
| 部署 | Helm / docker-compose |

**技术栈**：

| 层 | 技术 |
|---|---|
| 后端框架 | Rust (Axum) + gRPC (Tonic) |
| 前端框架 | Vue 3.5 + Quasar 2 + Pinia + Vite |
| 数据库 | PostgreSQL（119 表） |
| 缓存 | Redis |
| 认证 | JWT Bearer |
| 部署 | Helm / docker-compose |
| 工程化 | pnpm workspaces + Turborepo |

## 2. 系统架构图

```mermaid
flowchart TB
    subgraph FE["前端四端（Frontend/）"]
        ADMIN["🖥️ admin 平台运营<br/>49 页面"]
        TENANT["🖥️ tenant 单位业务"]
        OPS["📊 ops 运营面板"]
        SOCIAL["💬 social 社交端"]
    end

    subgraph SHARED["共享层（packages/）"]
        API["api · 四端共享 API 封装"]
        BOOT["boot · alova 实例 + 认证注入"]
        COMP["components · 共享组件"]
        PAGES["pages · 共享页面模板"]
    end

    subgraph BACKEND["后端微服务（Backend/，Rust）"]
        GW["api-gateway<br/>统一入口 · JWT 鉴权 · 限流熔断"]
        subgraph BIZ["业务微服务（22个）"]
            AUTH["auth-service"]
            USER["user-service"]
            BILL["billing-service"]
            PAY["pay-service"]
            ..."]
        end
        subgraph CRATES["共享 Crates（10个）"]
            COMMON["common · 公共工具"]
            AUTHCORE["auth-core · 认证核心"]
            TENANTCORE["tenant-core · 租户核心"]
            ..."]
        end
    end

    subgraph INFRA["基础设施"]
        PG[("PostgreSQL 16+<br/>119 表")]
        RD[("Redis 7+<br/>缓存/会话")]
        MEILI[("Meilisearch<br/>全文搜索（可选）")]
    end

    FE --> SHARED
    SHARED --> BACKEND
    BACKEND --> INFRA
```

## 3. 快速开始

### 前置条件

- Rust 1.85+
- Node.js 20+ / pnpm 11+
- Podman/Docker（PostgreSQL + Redis）
- （可选）sqlx-cli：`cargo install sqlx-cli --no-default-features --features postgres`

### 本地启动（docker-compose）

```bash
# 1. 配置环境变量
cp .env.example .env

# 2. 启动基础设施（PostgreSQL + Redis + Meilisearch）
podman compose -f deploy/docker-compose.yml up -d

# 3. 初始化数据库
psql -h localhost -U postgres -d erp_new -f Backend/sql/000_init.sql
psql -h localhost -U postgres -d erp_new -f Backend/sql/001_saas_core.up.sql
psql -h localhost -U postgres -d erp_new -f Backend/sql/001_tenant_id_columns.sql
psql -h localhost -U postgres -d erp_new -f Backend/sql/002_missing_indexes.sql
psql -h localhost -U postgres -d erp_new -f Backend/sql/003_subscription_tables.sql

# 4. 编译后端
cd Backend
cargo sqlx prepare --workspace
cargo build --workspace

# 5. 启动后端（按需启动服务）
cargo run --package api-gateway     # HTTP 8090
cargo run --package user-service    # HTTP 8080 / gRPC 9090
cargo run --package auth-service    # HTTP 8081 / gRPC 9091

# 6. 启动前端
cd Frontend
pnpm install
pnpm --filter myai-admin dev    # http://localhost:9000
```

## 4. 生产部署（K8s）

```bash
# 构建并推送镜像
podman build -t erp-new/api-gateway:latest --build-arg SERVICE_NAME=api-gateway --build-arg HTTP_PORT=8090 -f Backend/Dockerfile .
podman push erp-new/api-gateway:latest

# Helm 部署
helm upgrade --install myai ./deploy/helm/myai \
  -n myai-prod \
  -f deploy/helm/myai/values-prod.yaml \
  --set image.tag=$(date +%Y%m%d%H%M)
```

## 5. 微服务清单

| 服务 | HTTP 端口 | gRPC 端口 | 说明 |
|---|---|---|---|
| api-gateway | 8090 | - | 统一入口 |
| auth-service | 8081 | 9091 | 认证授权 |
| user-service | 8080 | 9090 | 用户管理 |
| tenant-service | 8087 | 9095 | 多租户 |
| billing-service | 8088 | 9101 | 计费订阅 |
| pay-service | 8093 | 9093 | 支付服务 |
| audit-service | 8089 | 9010 | 审计日志 |
| file-service | 8084 | 9092 | 文件管理 |
| cms-service | 8082 | 9082 | 内容管理 |
| messaging-service | 8083 | 9083 | 即时通讯 |
| workflow-service | 8088 | 9088 | 工作流 |
| feedback-service | 8085 | 9085 | 意见反馈 |
| api-key-service | 8091 | 9094 | API Key |
| social-ops-service | 8110 | 9110 | 社交运营 |
| browser-service | 8120 | 8120 | 浏览器自动化 |
| hik-service | 8092 | 9092 | 海康设备 |
| lpr-service | 8099 | 9099 | 车牌识别 |
| xlt-service | 8097 | 9096 | 信路通停车 |
| ctp-service | 8096 | 9097 | CTP 平板锁 |
| ebike-service | 8098 | 9100 | 电动自行车 |
| tow-service | 8094 | 9086 | 拖车服务 |

## 6. 版本历史

| 版本 | 日期 | 说明 |
|---|---|---|
| v0.4.0 | 2026-09-15 | 全面优化：修复 panic!、清理桩页面、重构 Helm Chart、文档同步 |
| v0.3.0 | 2026-09-15 | 错误处理统一（AppError/AppResult）、清理冗余、精简部署目录 |
| v0.2.5 | 2026-09-14 | 文档同步修正：表数(127→119)、服务数(21→22)、Crates(11→10) |
| v0.2.0 | 2026-08-11 | 架构文档 |
| v0.1.0 | 2026-06-10 | 初始架构设计 |

## 7. 许可证

[MIT](LICENSE) © 2026 erp_new Contributors
