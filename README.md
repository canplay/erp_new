# MyAI · 全栈微服务管理平台

> 基于 Rust + Vue 3 + Quasar 的全栈微服务管理平台，采用 Service → gRPC → Gateway → HTTP 的架构，已部署至 K8s（Rancher 管理）。

## 项目简介

MyAI 是一套面向多业务场景的全栈微服务管理平台，采用 Rust gRPC 微服务架构，提供用户认证、权限管理、审计日志、即时通讯、内容管理、工作流、支付、AI 服务等核心能力。

## 目录结构

```
myai/
├── Backend/                        # 后端（22 个微服务 + API Gateway + 11 个共享 Crates）
│   ├── services/                   # 各微服务源码（21 个）
│   │   ├── api-gateway/            # HTTP 网关（统一 REST + WS 入口）
│   │   ├── auth-service/           # 认证服务
│   │   ├── user-service/           # 用户服务
│   │   ├── tenant-service/         # 多租户服务
│   │   ├── audit-service/          # 审计日志
│   │   ├── file-service/           # 文件服务
│   │   ├── cms-service/            # 内容管理
│   │   ├── messaging-service/      # 即时通讯
│   │   ├── workflow-service/       # 工作流
│   │   ├── pay-service/            # 支付服务
│   │   ├── feedback-service/       # 反馈服务
│   │   ├── api-key-service/        # API Key 管理
│   │   ├── social-ops-service/     # 社交运营
│   │   ├── ctp-service/            # CTP 服务
│   │   ├── ebike-service/          # 电单车
│   │   ├── hik-service/            # 海康接入
│   │   ├── lpr-service/            # 车牌识别
│   │   ├── tow-service/            # 拖车
│   │   ├── browser-service/        # 浏览器服务
│   │   ├── clean-service/          # 清洁服务
│   │   ├── xlt-service/            # XLT 服务
│   │   └── billing-service/        # 计费服务（计划/订阅/发票/用量）
│   ├── crates/                     # 共享 Crates（11 个）
│   │   ├── auth-core/              # 认证核心
│   │   ├── cache-core/             # 缓存核心
│   │   ├── circuit-breaker-core/   # 熔断核心
│   │   ├── common/                 # 公共库
│   │   ├── crypto-core/            # 加密核心
│   │   ├── billing-core/           # 计费核心（计划/订阅/发票/用量）
│   │   ├── grpc-core/              # gRPC 核心
│   │   ├── grpc-proto/             # gRPC Proto 生成
│   │   ├── log-core/               # 日志核心
│   │   └── tenant-core/            # 租户核心（多租户隔离）
│   ├── protos/                     # gRPC Proto 定义（20 个 .proto）
│   ├── sql/
│   │   ├── 000_init.sql            # 数据库 schema（103 表）
│   │   ├── 001_saas_core.up.sql    # SaaS 核心表（租户/订阅/计费）
│   │   ├── 001_tenant_id_columns.sql  # tenant_id 字段补充
│   │   ├── 002_missing_indexes.sql    # 缺失索引补充
│   │   ├── 003_subscription_tables.sql # 订阅计费表（5 表）
│   │   ├── gw_tables.sql              # Gateway 专属表（7 表）
│   │   ├── 999_reset.sql              # 数据库重置（开发用）
│   │   └── schema.sql                 # Schema 版本追踪
│   ├── Cargo.toml                  # 工作区配置
│   └── Dockerfile.*                # 多阶段构建
│
├── Frontend/                       # 前端（pnpm + Turborepo）
│   ├── admin/                      # 管理后台（平台运营视角）
│   ├── tenant/                     # 单位业务视角
│   ├── ops/                        # 运营面板
│   ├── social/                     # 社交端
│   ├── packages/
│   │   ├── api/                    # API 封装（alova）
│   │   ├── boot/                   # 启动引导（alova 实例）
│   │   ├── capabilities/           # 能力开关
│   │   ├── components/             # 共享组件
│   │   ├── composables/            # 组合式函数
│   │   ├── i18n/                   # 国际化
│   │   ├── pages/                  # 共享页面
│   │   ├── stores/                 # 共享状态
│   │   ├── types/                  # 类型定义
│   │   └── utils/                  # 工具函数
│   ├── package.json                # 工作区配置
│   └── Dockerfile.*                # 前端镜像
│
├── Scripts/                        # 构建/部署脚本
│   ├── build/                      # 构建脚本
│   ├── deploy/                     # 部署脚本
│   │   ├── backend/                # 后端部署
│   │   ├── frontend/               # 前端部署
│   │   └── helm-deploy-auto-tags.sh
│   ├── helm/myai/                  # Helm Chart
│   ├── lib/                        # 共享函数库
│   └── verify-deployment.sh        # 部署验证
│
├── Docs/                           # 文档
│   ├── 设计文档.md                  # 架构设计
│   ├── 部署文档.md                  # 部署指南
│   ├── 后端待办.md                  # 后端待办
│   ├── 前端待办.md                  # 前端待办
│   └── ...
│
├── .env.example                    # 后端运行时环境变量（30+ 变量）
├── Scripts/.env.example            # 构建部署环境变量（Harbor/Rancher/NAMESPACE）
└── LICENSE                         # MIT 许可证
```

## 技术栈

| 层 | 技术 |
|---|---|
| 后端框架 | Rust (Axum) + gRPC (Tonic) |
|| 前端框架 | Vue 3.5 + Quasar 2 + Pinia + Vite（admin 平台 / tenant 单位 / ops 运营 / social 社交） |
|| 数据库 | PostgreSQL（127 表：000_init 103 + 001_saas + 001_tenant_id + 002_index + 003_subscription 5 + gw_tables 7 + 999_reset + schema） |
| 缓存 / 任务 | Redis |
| 认证 | JWT Bearer |
| 容器化 | Docker / Podman / Kubernetes |
| 部署 | Helm / docker-compose |
| 工程化 | pnpm workspaces + Turborepo |

## 快速开始

### 前置条件

- Rust 1.85+
- Node.js 20+ / pnpm 11+
- Docker/Podman（PostgreSQL + Redis）
- （可选）sqlx-cli：`cargo install sqlx-cli --no-default-features --features postgres`

### 本地启动

```bash
# 1. 配置环境变量
cp .env.example .env
cp Scripts/.env.example Scripts/.env

# 2. 初始化数据库（共 127 表）
psql -h <host> -U postgres -d myai -f Backend/sql/000_init.sql
psql -h <host> -U postgres -d myai -f Backend/sql/001_saas_core.up.sql
psql -h <host> -U postgres -d myai -f Backend/sql/001_tenant_id_columns.sql
psql -h <host> -U postgres -d myai -f Backend/sql/002_missing_indexes.sql
psql -h <host> -U postgres -d myai -f Backend/sql/003_subscription_tables.sql
psql -h <host> -U postgres -d myai -f Backend/sql/gw_tables.sql

# 3. 编译时 SQL 验证（离线模式，无需数据库连接）
cd Backend
cargo sqlx prepare --workspace

# 4. 启动后端（按需启动服务）
cargo run --package api-gateway     # HTTP 8090
cargo run --package user-service    # HTTP 8080 / gRPC 9090
cargo run --package auth-service    # HTTP 8081 / gRPC 9091
cargo run --package billing-service # HTTP 8088 / gRPC 9101

# 5. 启动前端
cd Frontend
pnpm install
pnpm dev:admin    # http://localhost:9000
```

## 生产部署（K8s）

```bash
# 构建
bash Scripts/build.sh backend
bash Scripts/build.sh admin

# 推送 Harbor
bash Scripts/push.sh <组件>

# Helm 部署
KUBECONFIG=rancher.kubeconfig bash Scripts/deploy.sh myai

# 验证
bash Scripts/verify-deployment.sh myai
```

## 环境

| 项 | 值 |
|---|---|
| 生产环境 | `https://admin.100.100.100.100.example.com:8901`（仅 HTTPS） |
| 集群命名空间 | `myai-prod` |
| 镜像仓库 | `harbor.100.100.100.101.example.com:8003/myai` |
| 镜像 tag 格式 | `YYYYMMDDHHMM`（纯数字 12 位） |
| 数据库 | `myai` (PostgreSQL) |
| Redis | `redis-replication` |

## 许可证

[MIT](LICENSE) © 2026 MyAI Contributors
