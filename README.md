# MyAI — 全栈微服务管理平台

基于 Rust + Vue 3 + Quasar 的全栈微服务平台，采用 Service → gRPC → Gateway → HTTP 的微服务架构，已部署至 K8s（Rancher 管理）。

## 项目结构

```
MyAI/
├── Backend/                        # 后端（21 个 gRPC 微服务 + API Gateway）
│   ├── services/                   # 各微服务源码
│   ├── protos/                     # gRPC Proto 定义（20 个 .proto 文件）
│   ├── crates/                     # 共享 Crate（common/auth-core/grpc-proto 等）
│   ├── sql/
│   │   ├── schema.sql              # 数据库 schema（幂等，83+ 表）
│   │   └── migrations/             # sqlx 迁移（含网关 gw_ 表）
│   ├── compile.Dockerfile          # 容器内编译（产 ELF，SQLX_OFFLINE）
│   ├── base.Dockerfile             # 运行基础镜像
│   └── Dockerfile.local            # 服务运行镜像模板
│
├── Frontend/
│   ├── admin/                      # 管理后台（Vue3 + Quasar，生产主前端）
│   ├── ops/                        # 运营面板
│   ├── social/                     # 社交端
│   ├── nginx.conf                  # /api + /ws 反代（envsubst 模板）
│   └── Dockerfile.local-admin      # 前端镜像（本地产物 → nginx）
│
├── helm/
│   └── myai/                       # Helm Chart（部署到 myai-prod 命名空间）
│
├── deploy/
│   ├── scripts/                    # 构建/推送/部署/密钥/验证脚本（7 个）
│   └── docs/                       # 部署运维文档
│
├── docs/                           # 技术文档（见下方索引）
├── tools/                          # 开发辅助脚本（路由分析/代码修复）
└── .env.example                    # 统一环境配置模板
```

## 架构

```
┌──────────────────────────────────────────────────┐
│  Backend Services (21, 纯 gRPC)                   │
│  api-gateway user auth file cms audit message      │
│  feedback tenant api-key workflow social-ops       │
│  browser clean ctp ebike hik lpr pay tow xlt       │
└──────────────────────┬── gRPC ──────────────────┘
                       │
┌──────────────────────▼──────────────────────────┐
│  api-gateway (HTTP 唯一入口，统一 REST + WS)       │
└──────────────────────┬── HTTP ──────────────────┘
                       │
            ┌──────────┴──────────┬──────────┐
            ▼                     ▼          ▼
         admin (前端)           ops        social
       HTTPS :8901            (预留)      (预留)
```

## 快速开始（本地开发）

### 前置条件

- Rust 1.85+（后端）
- Node.js 20+ / pnpm 9+（前端）
- Docker/Podman（PostgreSQL + Redis）

### 本地启动

```bash
# 1. 配置环境变量
cp .env.example .env

# 2. 初始化数据库（幂等，可重复执行）
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

生产环境已通过 Helm 部署至 Rancher 集群（命名空间 `myai-prod`），完整流程见：

> 📖 **[docs/build-deploy-guide.md](docs/build-deploy-guide.md)** — 编译 → 镜像构建 → 推送 Harbor → Helm 部署 → 验证 + 故障排查

## 文档索引

| 文档 | 说明 |
|------|------|
| [docs/build-deploy-guide.md](docs/build-deploy-guide.md) | **编译→构建→推送→部署全流程** |
| [docs/ebike-api-guide.md](docs/ebike-api-guide.md) | **共享单车数据接入 API** |
| [docs/ebike-frontend-ready.md](docs/ebike-frontend-ready.md) | ebike 前端改造记录 |
| [docs/backend-rust-reference.md](docs/backend-rust-reference.md) | 后端架构参考 |
| [docs/frontend-admin-guide.md](docs/frontend-admin-guide.md) | 前端开发指南 |
| [docs/README.md](docs/README.md) | 完整文档索引 |

## 环境

| 项 | 值 |
|----|-----|
| 生产环境 | `https://admin.100.100.100.100.example.com:8901`（仅 HTTPS） |
| 集群命名空间 | `myai-prod` |
| 镜像仓库 | `harbor.100.100.100.101.example.com:8003/erp` |
| 镜像 tag 格式 | `YYYYMMDDHHMM`（纯数字 12 位，如 `202608070337`） |
| 数据库 | 复用 `pg-cluster-postgresql`（KubeBlocks），库名 `myai` |
| Redis | 复用 `redis-replication`（KubeBlocks） |

## 里程碑

| 阶段 | 完成项 |
|------|--------|
| 架构统一 | 21 service 全部纯 gRPC，业务 HTTP 由网关承担 |
| 部署自动化 | Helm Chart + 构建/推送/部署/验证脚本（sh 版） |
| 安全加固 | Ingress 仅 HTTPS + 密钥外置 + gRPC 鉴权 + myai_app 最小权限 |
| 网关持久化 | 7 个 Repository 内存态迁 DB（gw_ 表 + sqlx 迁移） |
| 前端完善 | 细粒度 RBAC 权限码 + i18n 生效 + 死代码清理 |

## 许可证

[MIT](LICENSE) © 2026 MyAI Contributors
