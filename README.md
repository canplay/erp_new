# MyAI · 全栈微服务管理平台

> 基于 Rust + Vue 3 + Quasar 的全栈微服务管理平台，采用 Service → gRPC → Gateway → HTTP 的架构，已部署至 K8s（Rancher 管理）。

## 项目简介

MyAI 是一套面向多业务场景的全栈微服务管理平台，采用 Rust gRPC 微服务架构，提供用户认证、权限管理、审计日志、即时通讯、内容管理、工作流、支付、AI 服务等核心能力。

## 目录结构

```
myai/
├── Backend/                        # 后端（22 个 gRPC 微服务 + API Gateway）
│   ├── services/                   # 各微服务源码
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
│   │   ├── social-ops-service:     # 社交运营
│   │   ├── ctp-service/            # CTP 服务
│   │   ├── ebike-service/          # 电单车
│   │   ├── hik-service/            # 海康接入
│   │   ├── lpr-service/            # 车牌识别
│   │   ├── tow-service/            # 拖车
│   │   ├── browser-service:        # 浏览器服务
│   │   ├── clean-service/          # 清洁服务
│   │   ├── xlt-service/            # XLT 服务
│   │   └── ...
│   ├── crates/                     # 共享 Crate（9 个）
│   │   ├── auth-core/              # 认证核心
│   │   ├── cache-core/             # 缓存核心
│   │   ├── circuit-breaker-core/   # 熔断核心
│   │   ├── common/                 # 公共库
│   │   ├── crypto-core/            # 加密核心
│   │   ├── grpc-core/              # gRPC 核心
│   │   ├── grpc-proto/             # gRPC Proto 生成
│   │   └── log-core/               # 日志核心
│   ├── protos/                     # gRPC Proto 定义（20+ .proto）
│   ├── sql/
│   │   └── schema.sql              # 数据库 schema（83+ 表）
│   ├── Cargo.toml                  # 工作区配置
│   └── Dockerfile.*                # 多阶段构建
│
├── Frontend/                       # 前端（pnpm + Turborepo）
│   ├── admin/                      # 管理后台（Vue3 + Quasar）
│   ├── ops/                        # 运营面板
│   ├── social/                     # 社交端
│   ├── packages/
│   │   └── shared/                 # 共享包
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
├── tools/                          # 开发辅助工具
│   ├── gen_demo_pages.py           # 演示页面生成
│   └── gen_report_docx.py          # 报告文档生成
│
├── Docs/                           # 文档
│   ├── build-deploy-guide.md       # 构建部署全流程
│   ├── ebike-api-guide.md          # 电单车 API
│   ├── backend-rust-reference.md   # 后端架构参考
│   └── ...
│
├── .env.example                    # 环境变量模板
└── LICENSE                         # MIT 许可证
```

## 技术栈

| 层 | 技术 |
|---|---|
| 后端框架 | Rust (Actix Web) + gRPC (Tonic) |
| 前端框架 | Vue 3.5 + Quasar 2 + Pinia + Vite |
| 数据库 | PostgreSQL |
| 缓存 / 任务 | Redis / Hangfire |
| 认证 | JWT Bearer + ASP.NET Identity |
| 容器化 | Docker / Podman / Kubernetes |
| 部署 | Helm / docker-compose |
| 工程化 | pnpm workspaces + Turborepo |

## 快速开始

### 前置条件

- Rust 1.85+
- Node.js 20+ / pnpm 9+
- Docker/Podman（PostgreSQL + Redis）

### 本地启动

```bash
# 1. 配置环境变量
cp .env.example .env

# 2. 初始化数据库
psql -h <host> -U postgres -d myai -f Backend/sql/schema.sql

# 3. 启动后端（按需启动服务）
cd Backend
cargo run --package api-gateway     # HTTP 8090
cargo run --package user-service    # gRPC 9090

# 4. 启动前端
cd Frontend/admin
pnpm install
npx quasar dev    # http://localhost:9000
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
