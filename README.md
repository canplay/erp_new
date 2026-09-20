# erp_new · 全栈微服务管理平台

> 基于 Rust + Vue 3 + Quasar 的全栈微服务管理平台，采用 Service → gRPC → Gateway → HTTP 架构，可部署至 K8s（Rancher 管理）。

## 1. 项目概览

| 维度 | 数据 |
|---|---|
| 后端 | 22 微服务 / 10 共享 Crates / 492 Rust 源文件 |
| 前端 | 4 应用（admin 47 / tenant / ops / social）/ 10 共享包 / 309 Vue + 540 TS 文件 |
| 数据库 | PostgreSQL 16+ / 117 张表 |
| 测试 | `cargo test --workspace --lib` 380 passed / 0 failed |
| 部署 | Helm / docker-compose |

**技术栈**：

| 层 | 技术 |
|---|---|
| 后端框架 | Rust (Axum) + gRPC (Tonic) |
| 前端框架 | Vue 3.5 + Quasar 2 + Pinia + Vite |
| 数据库 | PostgreSQL（117 表） |
| 缓存 | Redis |
| 认证 | JWT Bearer |
| 部署 | Helm / docker-compose |
| 工程化 | pnpm workspaces + Turborepo |

## 2. 系统架构图

```mermaid
flowchart TB
    subgraph FE["前端四端（Frontend/）"]
        ADMIN["🖥️ admin 平台运营<br/>47 页面"]
        TENANT["🖥️ tenant 单位业务<br/>共享 pages"]
        OPS["📊 ops 运营面板"]
        SOCIAL["💬 social 社交端"]
    end

    subgraph SHARED["共享层（packages/）"]
        API["api · 四端共享 API 封装"]
        BOOT["boot · alova 实例 + 认证注入"]
        COMP["components · 共享组件 138"]
        PAGES["pages · 页面模板（111 文件）"]
    end

    subgraph BACKEND["后端微服务（Backend/，Rust）"]
        GW["api-gateway<br/>统一入口 · JWT 鉴权 · 限流熔断"]
        subgraph BIZ["业务微服务（22个）"]
            AUTH["auth-service"]
            USER["user-service"]
            BILL["billing-service"]
            PAY["pay-service"]
            ...[""]
        end
        subgraph CRATES["共享 Crates（10个）"]
            COMMON["common · 公共工具"]
            AUTHCORE["auth-core · 认证核心"]
            TENANTCORE["tenant-core · 租户核心"]
            ...[""]
        end
    end

    subgraph INFRA["基础设施"]
        PG[("PostgreSQL 16+<br/>117 表")]
        RD[("Redis 7+<br/>缓存/会话")]
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

# 3. 初始化数据库（自动 via docker-compose volumes）
# 或手动: psql -h localhost -U postgres -d erp_new -f Backend/sql/schema.sql

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
pnpm --filter myai-admin dev      # http://localhost:9000
pnpm --filter myai-tenant dev     # http://localhost:9001
pnpm --filter myai-ops dev        # http://localhost:9002
pnpm --filter myai-social dev     # http://localhost:9003
```

## 4. 生产部署（K8s）

```bash
# 构建并推送镜像
podman build -t erp-new/api-gateway:latest --build-arg SERVICE_NAME=api-gateway --build-arg HTTP_PORT=8090 -f Backend/Dockerfile .
podman push erp-new/api-gateway:latest

# Helm 部署
helm upgrade --install erp-new ./deploy/helm/myai \
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
| clean-service | 8100 | 8100 | 清洁服务 |

## 6. 数据库迁移

```bash
# 全量初始化（docker-compose 自动执行）
psql -h <host> -U postgres -d erp_new -f Backend/sql/schema.sql

# 重置（开发用，危险！）
psql -h <host> -U postgres -d erp_new -f Backend/sql/999_reset.sql
```

## 7. 可观测性

| 端点 | 说明 |
|---|---|
| `/health` | 健康检查（所有服务） |
| `/metrics` | Prometheus 指标（api-gateway + 21 服务） |

## 8. CI/CD

GitHub Actions 自动执行：
- Backend: `cargo fmt`, `clippy`, `check`, `test`
- Frontend: `vue-tsc` 类型检查, `quasar build`
- Helm: `lint` 验证

## 9. 版本历史

| 版本 | 日期 | 说明 |
|---|---|---|
| v0.9.0 | 2026-09-20 | 深度重构: N+1 修复(31处审计+11处修复), 超大文件拆分(9文件→42模块), common re-export 精简(40+→12), as unknown as 清零, sqlx 离线缓存重建, 380 测试全通过 |
| v0.8.0 | 2026-09-19 | 全面优化: f64→Decimal, batch事务, SQL白名单, SELECT *, 租户隔离, pay回调, 类型安全, 日志统一, dead_code清零 |
| v0.6.0 | 2026-09-16 | CI/CD, Prometheus metrics, docker-compose auto-init |
| v0.5.0 | 2026-09-16 | SQL 合并, migrations 模块, any 清理 |
| v0.4.0 | 2026-09-15 | 修复 panic!, 清理桩页面, 重构 Helm |
| v0.3.0 | 2026-09-15 | 错误处理统一, 清理冗余 |
| v0.2.5 | 2026-09-14 | 文档同步修正 |
| v0.2.0 | 2026-08-11 | 架构文档 |
| v0.1.0 | 2026-06-10 | 初始架构设计 |

## 10. 许可证

[MIT](LICENSE) © 2026 erp_new Contributors
