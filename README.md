# erp_new · 全栈微服务管理平台

> 基于 Rust + Vue 3 + Quasar 的全栈微服务管理平台，采用 Service → gRPC → Gateway → HTTP 的架构，可部署至 K8s（Rancher 管理）。

## 1. 项目概览

| 维度 | 数据 |
|---|---|
| 后端 | 22 微服务 / 10 共享 Crates / 394 HTTP 端点 / 119 数据库表 |
| 前端 | 4 应用（admin 56 / tenant 86 / ops 19 / social 13）/ 10 共享包 |
| 架构 | admin=平台运营 / tenant=单位业务 / ops=运营面板 / social=社交端 |
| 部署 | Helm / docker-compose |

**技术栈**：

| 层 | 技术 |
|---|---|
| 后端框架 | Rust (Axum) + gRPC (Tonic) |
| 前端框架 | Vue 3.5 + Quasar 2 + Pinia + Vite |
| 数据库 | PostgreSQL |
| 缓存 | Redis |
| 认证 | JWT Bearer |
| 容器化 | Docker / Podman / Kubernetes |
| 部署 | Helm / docker-compose |
| 工程化 | pnpm workspaces + Turborepo |

## 2. 系统架构图

```mermaid
flowchart TB
    subgraph FE["前端四端（Frontend/）"]
        ADMIN["🖥️ admin 平台运营<br/>Quasar SPA<br/>56 页面"]
        TENANT["🖥️ tenant 单位业务<br/>Quasar SPA<br/>86 页面"]
        OPS["📊 ops 运营面板<br/>Quasar SPA<br/>19 页面"]
        SOCIAL["💬 social 社交端<br/>Quasar SPA<br/>13 页面"]
    end

    subgraph SHARED["共享层（packages/）"]
        API["api · 四端共享 API 封装"]
        BOOT["boot · alova 实例 + 认证注入"]
        COMP["components · 共享组件"]
        PAGES["pages · 共享页面"]
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
        PG[("PostgreSQL<br/>119 表")]
        RD[("Redis<br/>缓存/会话")]
        MEILI[("Meilisearch<br/>全文搜索（可选）")]
    end

    FE --> SHARED
    SHARED --> BACKEND
    BACKEND --> INFRA
```

## 3. 目录结构

```
erp_new/
├── Backend/                        # 后端（22 个微服务 + 10 个共享 Crates）
│   ├── services/                   # 各微服务源码
│   │   ├── api-gateway/            # HTTP 网关（统一 REST + WS 入口）
│   │   ├── auth-service/           # 认证服务
│   │   ├── user-service/           # 用户服务
│   │   ├── billing-service/        # 计费服务
│   │   ├── pay-service/            # 支付服务
│   │   └── ...（共 22 个）
│   ├── crates/                     # 共享 Crates（10 个）
│   │   ├── common/                 # 公共库（错误、配置、日志）
│   │   ├── auth-core/              # 认证核心
│   │   ├── tenant-core/            # 租户核心
│   │   └── ...
│   ├── protos/                     # gRPC Proto 定义（21 个 .proto）
│   ├── sql/                        # 数据库迁移脚本（8 文件，119 表）
│   └── Cargo.toml                  # 工作区配置
│
├── Frontend/                       # 前端（pnpm + Turborepo）
│   ├── apps/
│   │   ├── admin/                  # 管理后台（56 页面，平台运营）
│   │   ├── tenant/                 # 单位业务（86 页面）
│   │   ├── ops/                    # 运营面板（19 页面）
│   │   └── social/                 # 社交端（13 页面）
│   ├── packages/                   # 共享包（10 个）
│   │   ├── api/                    # API 封装（alova）
│   │   ├── boot/                   # 启动引导
│   │   ├── components/             # 共享组件
│   │   ├── composables/            # 组合式函数
│   │   ├── i18n/                   # 国际化
│   │   ├── pages/                  # 共享页面
│   │   ├── stores/                 # 共享状态
│   │   ├── types/                  # 类型定义
│   │   └── utils/                  # 工具函数
│   └── package.json
│
├── deploy/                         # 部署配置
│   ├── docker-compose.yml          # 本地开发编排
│   ├── helm/myai/                  # Helm Chart
│   └── scripts/                    # 构建/部署脚本
│       ├── build-backend.sh        # 后端镜像构建
│       ├── build-frontend.sh       # 前端镜像构建
│       ├── deploy-backend.sh       # 后端部署
│       ├── deploy-frontend.sh      # 前端部署
│       ├── build-and-deploy.sh     # 全栈流水线
│       └── lib/                    # 共享函数库
│
├── Docs/                           # 文档
│   ├── 设计文档.md                  # 架构设计
│   ├── 部署文档.md                  # 部署指南
│   ├── 后端待办.md                  # 后端待办
│   └── 前端待办.md                  # 前端待办
│
├── .env.example                    # 后端运行时环境变量
└── LICENSE
```

## 4. 快速开始

### 前置条件

- Rust 1.85+
- Node.js 20+ / pnpm 11+
- Docker/Podman（PostgreSQL + Redis）
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
psql -h localhost -U postgres -d erp_new -f Backend/sql/gw_tables.sql

# 4. 编译时 SQL 验证（离线模式）
cd Backend
cargo sqlx prepare --workspace

# 5. 启动后端（按需启动服务）
cargo run --package api-gateway     # HTTP 8090
cargo run --package user-service    # HTTP 8080 / gRPC 9090
cargo run --package auth-service    # HTTP 8081 / gRPC 9091

# 6. 启动前端
cd Frontend
pnpm install
pnpm --filter myai-admin dev    # http://localhost:9000
pnpm --filter myai-ops dev      # http://localhost:9001
pnpm --filter myai-social dev   # http://localhost:9002
```

### 本地启动（纯本地，无 docker）

```bash
# 1. 确保本地 PostgreSQL 和 Redis 已启动
#    修改 .env 中的 DATABASE_URL 和 REDIS_URL 指向本地

# 2. 初始化数据库（同上）

# 3. 启动后端
cd Backend
cargo run --package api-gateway

# 4. 启动前端
cd Frontend
pnpm install
pnpm --filter myai-admin dev
```

## 5. 生产部署（K8s）

```bash
# 构建镜像
bash deploy/scripts/build-backend.sh
bash deploy/scripts/build-frontend.sh

# 推送 Harbor
bash deploy/scripts/push.sh <组件>

# Helm 部署
KUBECONFIG=rancher.kubeconfig bash deploy/scripts/deploy-backend.sh
KUBECONFIG=rancher.kubeconfig bash deploy/scripts/deploy-frontend.sh

# 验证
bash deploy/scripts/verify-deployment.sh
```

## 6. 微服务清单

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

## 7. 环境

| 项 | 值 |
|---|---|
| 数据库 | `erp_new` (PostgreSQL) |
| Redis | `redis://localhost:6379/0` |
| 镜像 tag 格式 | `YYYYMMDDHHMM`（纯数字 12 位） |

## 8. 版本历史

| 版本 | 日期 | 说明 |
|---|---|---|
| v0.3.0 | 2026-09-15 | 错误处理统一（AppError/AppResult）、清理冗余、精简部署目录、文档同步 |
| v0.2.5 | 2026-09-14 | 文档同步修正：表数(127→119)、服务数(21→22)、Crates(11→10) |
| v0.2.0 | 2026-08-11 | 架构文档 |
| v0.1.0 | 2026-06-10 | 初始架构设计 |

## 9. 许可证

[MIT](LICENSE) © 2026 erp_new Contributors
