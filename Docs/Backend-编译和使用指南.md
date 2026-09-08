# MyAI Backend - 编译和使用指南

> **版本**: v0.2.0 | **最后更新**: 2026-08-11  
> **目标读者**: 后端开发人员、运维工程师、新入职成员

---

## 目录

1. [项目架构概述](#1-项目架构概述)
2. [环境要求](#2-环境要求)
3. [快速开始](#3-快速开始)
4. [编译步骤](#4-编译步骤)
5. [运行方式](#5-运行方式)
6. [环境变量配置](#6-环境变量配置)
7. [服务端口清单](#7-服务端口清单)
8. [数据库初始化](#8-数据库初始化)
9. [API 文档访问](#9-api-文档访问)
10. [Docker 构建与部署](#10-docker-构建与部署)
11. [服务健康检查](#11-服务健康检查)
12. [常见问题与排查](#12-常见问题与排查)
13. [附录](#13-附录)

---

## 1. 项目架构概述

MyAI 后端采用 **微服务架构**，基于 Rust 语言开发，使用 Axum 作为 HTTP 框架，Tonic 作为 gRPC 框架。

### 1.1 架构分层

```
┌─────────────────────────────────────────────────────────┐
│                    前端 (Vue/Quasar)                   │
└────────────────────────┬─────────────────────────────────┘
                                         │ HTTP/WS
┌─────────────────────────────────────────────────────────┐
│              api-gateway (端口 8090)                    │
│   • JWT 鉴权  • 限流  • 熔断  • 路由              │
└────────────────────────┬─────────────────────────────────┘
                                         │ gRPC
┌─────────────────────────────────────────────────────────┐
│                   微服务集群 (21个服务)                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │auth-svc │ │user-svc  │ │cms-svc  │ ...      │
│  │9091/8081│ │9090/8080│ │9082/8082│          │
│  └──────────┘ └──────────┘ └──────────┘          │
└─────────────────────────────────────────────────────────┘
         │           │           │
    ┌─────────────────────────────────────────────┐
    │ PostgreSQL (共享库 + Schema隔离)              │
    │ Redis (缓存/会话)                          │
    └─────────────────────────────────────────────┘
```

### 1.2 服务分类

| 类别 | 服务 | 说明 |
|---------|---------|---------|
| **核心网关** | api-gateway | 统一入口，JWT鉴权、限流、熔断 |
| **认证授权** | auth-service | RBAC权限、JWT签发验证 |
| **用户管理** | user-service | 用户、部门、角色、会话 |
| **内容管理** | cms-service | 文章、分类、评论 |
| **消息通知** | messaging-service | 站内信、公告 |
| **文件管理** | file-service | 文件存储、目录管理 |
| **多租户** | tenant-service | 租户管理、配置、使用量统计 |
| **工作流** | workflow-service | 工作流引擎、任务调度 |
| **意见反馈** | feedback-service | 反馈提交、回复管理 |
| **审计安全** | audit-service | 操作日志、敏感操作审计、IP白名单 |
| **API Key** | api-key-service | API密钥管理、轮换、使用统计 |
| **支付服务** | pay-service | 建行/汇付支付集成 |
| **数据清理** | clean-service | 报表生成、数据清理、定时任务 |
| **海康设备** | hik-service | 海康威视设备集成 |
| **CTP平板锁** | ctp-service | CTP协议对接 |
| **车牌识别** | lpr-service | 车牌识别、相机回调 |
| **信路通** | xlt-service | 信路通MQTT停车协议 |
| **拖车服务** | tow-service | 车辆管理、扣押原因 |
| **电动自行车** | ebike-service | 车辆、订单、仓储管理 |
| **浏览器自动化** | browser-service | 浏览器自动化控制 |
| **社交运营** | social-ops-service | 社交运营相关功能 |

### 1.3 技术栈

| 技术 | 版本 | 用途 |
|---------|---------|---------|
| Rust | 2024 edition | 主语言 |
| Axum | 0.8 | HTTP 框架 |
| Tonic | 0.14 | gRPC 框架 |
| SQLx | 0.8 | PostgreSQL 异步ORM |
| Tokio | 1.52 | 异步运行时 |
| Redis | 1.2 | 缓存/会话 |
| JWT | 10 | 令牌签发验证 |
| Serde | 1.0 | 序列化 |
| Tracing | 0.1 | 结构化日志 |
| OpenTelemetry | 0.32 | 分布式追踪 |
| Protobuf | 0.14 | gRPC 协议定义 |

---

## 2. 环境要求

### 2.1 必需环境

| 环境 | 最低版本 | 说明 |
|----------------|----------------|---------|
| **Rust** | 1.97.0+ | 使用 `rustup` 安装 |
| **PostgreSQL** | 18+ | 数据库 |
| **Redis** | 7+ | 缓存/会话存储 |
| **protobuf-compiler** | 最新 | gRPC 编译 |
| **cargo-sqlx** | 最新 | SQLx 编译时验证 |

### 2.2 推荐工具

| 工具 | 用途 |
|----------------|---------|
| **docker** | 本地开发环境编排 |
| **kubectl** | K8s 集群管理 |
| **helm** | Helm chart 管理 |
| **podman** | 容器构建替代方案 |

### 2.3 安装 Rust

```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装最新稳定版
rustup default stable

# 验证安装
rustc --version  # 应显示 1.97.0 或更高
```

### 2.4 安装 SQLx CLI

```bash
# SQLx 编译时验证需要 cargo-sqlx
cargo install sqlx-cli

# 验证
sqlx --version
```

### 2.5 安装 protobuf-compiler

```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Windows (使用 vcpkg 或 Chocolatey)
choco install protobuf
```

---

## 3. 快速开始

### 3.1 克隆项目

```bash
git clone https://your-repo/myai.git
cd myai
```

### 3.2 复制环境变量

```bash
# 复制环境变量配置
cp .env.example .env

# 编辑 .env 文件，修改数据库连接等配置
# 使用你喜欢的编辑器打开 .env 文件
# 说明: 后端启动时 init_env() 通过 dotenvy 自动加载 .env (仅本地开发,
#       生产 K8s 环境由 ConfigMap/Secret 注入, 无 .env 文件时静默跳过)
```

### 3.3 本地开发启动

```bash
# 本地开发: dotenv 自动加载 .env (init_env → dotenvy::dotenv)
cd Backend
cargo build --release  # 首次编译全 workspace

# 启动指定服务 (API Gateway 为例)
cargo run -p api-gateway
```

### 3.4 验证服务

```bash
# 检查 API Gateway 健康状态
curl http://localhost:8090/health

# 预期响应: OK
```

---

## 4. 编译步骤

### 4.1 完整编译

```bash
cd Backend

# 1. 准备 SQLx 查询缓存 (首次编译需要数据库连接)
export DATABASE_URL=postgresql://postgres:***@localhost:5432/myai
cargo sqlx prepare --all-targets

# 2. 离线编译 (推荐，无需数据库连接)
export SQLX_OFFLINE=true
cargo build --release --all-targets

# 3. 编译所有服务二进制
ls -lh target/release/ | grep -E '(api-gateway|auth-service|user-service)'
```

### 4.2 增量编译

```bash
# 仅编译变更的服务 (开发模式)
cargo build --bin api-gateway

# 编译特定服务
cargo build --bin auth-service
cargo build --bin user-service
```

### 4.3 编译优化说明

项目配置了生产构建优化 (`Cargo.toml`):

```toml
[profile.release]
lto = true              # 链接时优化
codegen-units = 1           # 单一代码生成单元
opt-level = 3               # 最大优化
strip = true                # 移除调试符号
panic = "abort"             # 崩溃时直接退出
```

**效果**:
- 二进制体积减少 30-50%
- 运行时性能提升 10-20%
- 内存占用减少

### 4.4 跨平台编译

```bash
# 目标平台: linux/amd64 (Docker 容器)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# 目标平台: linux/arm64 (ARM 服务器)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

---

## 5. 运行方式

### 5.1 单服务运行

```bash
# 启动 API Gateway
cd Backend/services/api-gateway
API_GATEWAY_PORT=8090 \
JWT_SECRET=your-secret-key-here \
DATABASE_URL=postgresql://postgres:***@localhost:5432/myai \
cargo run --release
```

### 5.2 全部服务并行运行 (开发)

```bash
# 使用 tmux 或 screen 并行启动
# 终端1: API Gateway
cd Backend/services/api-gateway && cargo run --release

# 终端2: Auth Service
cd Backend/services/auth-service && cargo run --release

# 终端3: User Service
cd Backend/services/user-service && cargo run --release

# ... 其他服务类似
```

### 5.3 Docker Compose 启动 (推荐)

```bash
# 在项目根目录
docker compose up -d

# 查看日志
docker compose logs -f api-gateway
docker compose logs -f auth-service
```

### 5.4 K8s 部署

```bash
# 使用 Helm chart 部署
cd deploy
helm install myai helm/myai/ \
  --set harbor.registry=localhost:5000 \
  --set harbor.namespace=myai
```

---

## 6. 环境变量配置

### 6.1 核心环境变量

> **注意**: 所有环境变量详见 `.env.example` 文件

| 变量名 | 必填 | 默认值 | 说明 |
|----------------|------|---------|---------|
| `DATABASE_URL` | 是 | `postgresql://postgres:***@100.100.100.100:8991/myai` | PostgreSQL 连接字符串 |
| `REDIS_URL` | 是 | `redis://localhost:6379/0` | Redis 连接字符串 |
| `JWT_SECRET` | 是 | 随机生成 | JWT 签名密钥 (≥32字符) |
| `JWT_ISSUER` | 否 | `myai` | JWT 签发者 |
| `JWT_AUDIENCE` | 否 | `myai-users` | JWT 受众 |

### 6.2 服务端口环境变量

| 变量名 | 默认值 | 说明 |
|----------------|---------|---------|
| `API_GATEWAY_PORT` | `8090` | API Gateway HTTP 端口 |
| `AUTH_SERVICE_PORT` | `8081` | Auth Service HTTP 端口 |
| `USER_SERVICE_PORT` | `8080` | User Service HTTP 端口 |
| `GRPC_PORT` | `9090` | gRPC 通用端口 |

### 6.3 限流配置

| 变量名 | 默认值 | 说明 |
|----------------|---------|---------|
| `RATE_LIMIT_MAX` | `1000` | 时间窗口内最大请求数 |
| `RATE_LIMIT_WINDOW` | `60` | 时间窗口 (秒) |
| `RATE_LIMIT_BURST` | `100` | 突发请求允许量 |

### 6.4 熔断器配置

| 变量名 | 默认值 | 说明 |
|----------------|---------|---------|
| `CIRCUIT_BREAKER_FAILURE_THRESHOLD` | `5` | 熔断失败阈值 |
| `CIRCUIT_BREAKER_TIMEOUT_SECS` | `60` | 熔断超时 (秒) |
| `CIRCUIT_BREAKER_HALF_OPEN_REQUESTS` | `3` | 半开状态请求数 |

### 6.5 数据库连接池配置

| 变量名 | 默认值 | 说明 |
|----------------|---------|---------|
| `DB_POOL_MAX_CONNECTIONS` | `10` | 最大连接数 |
| `DB_POOL_MIN_CONNECTIONS` | `2` | 最小空闲连接数 |
| `DB_POOL_CONNECT_TIMEOUT` | `30` | 连接超时 (秒) |
| `DB_POOL_IDLE_TIMEOUT` | `600` | 空闲超时 (秒) |
| `DB_POOL_MAX_LIFETIME` | `1800` | 连接最大存活时间 (秒) |

### 6.6 Schema 隔离配置

```bash
# 启用 Schema 隔离 (每个服务独立 schema)
export SERVICE_SCHEMA=user_service
# 表将创建在 user_service schema 下而非 public schema
```

### 6.7 生成 JWT Secret

```bash
# 使用 OpenSSL 生成安全的 JWT Secret
openssl rand -base64 48

# 输出示例: aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789...
# 将此值设置到 .env 的 JWT_SECRET
```

---

## 7. 服务端口清单

### 7.1 HTTP 端口

| 服务 | HTTP 端口 | gRPC 端口 | 说明 |
|----------------|---------|---------|---------|
| api-gateway | 8090 | - | 统一入口 |
| auth-service | 8081 | 9091 | 认证授权 |
| user-service | 8080 | 9090 | 用户管理 |
| cms-service | 8082 | 9082 | 内容管理 |
| messaging-service | 8083 | 9083 | 消息通知 |
| file-service | 8084 | 9092 | 文件管理 |
| tenant-service | 8087 | 9095 | 多租户 |
| workflow-service | 8088 | 9088 | 工作流 |
| feedback-service | 8085 | 9085 | 意见反馈 |
| audit-service | 8089 | 9010 | 审计安全 |
| api-key-service | 8091 | 9014 | API Key |
| hik-service | 8092 | 9096 | 海康设备 |
| ctp-service | 8094 | - | CTP平板锁 |
| lpr-service | 8097 | 9099 | 车牌识别 |
| xlt-service | 8095 | 9096 | 信路通 |
| tow-service | 0 | 9086 | 拖车服务 |
| pay-service | 8093 | 9097 | 支付服务 |
| clean-service | 8095 | 9087 | 数据清理 |
| ebike-service | 8096 | 9098 | 电动自行车 |
| browser-service | - | 8120 | 浏览器自动化 |
| social-ops-service | - | 9110 | 社交运营 |

### 7.2 端口分配规则

- **HTTP 端口**: 8080-8100 范围
- **gRPC 端口**: 9080-9120 范围
- **API Gateway**: 唯一入口 8090

---

## 8. 数据库初始化

### 8.1 首次部署

```bash
# 执行 schema.sql (注意: 包含 DROP TABLE IF EXISTS)
psql -U postgres -d myai -f Backend/sql/schema.sql
```

### 8.2 增量迁移

```bash
# 对于已有数据的数据库，手动执行 CREATE TABLE IF NOT EXISTS
# 参考 schema.sql 中的 IF NOT EXISTS 语句
```

### 8.3 Schema 隔离

```bash
# 启用 Schema 隔离
export SERVICE_SCHEMA=auth_service

# 表将创建在 auth_service schema 下
# 每个服务可独立配置 schema
```

### 8.4 数据库角色

```sql
-- 创建应用专用角色 (修复: 原使用 postgres 超级用户)
CREATE ROLE myai_app LOGIN PASSWORD 'your-password';
GRANT USAGE ON SCHEMA public TO myai_app;
GRANT USAGE ON SCHEMA socialops TO myai_app;
```

---

## 9. API 文档访问

### 9.1 Swagger UI

```bash
# 启动服务后访问
open http://localhost:8090/api/docs
```

### 9.2 OpenAPI JSON

```bash
# 获取 OpenAPI 规范
curl http://localhost:8090/api/openapi.json
```

### 9.3 健康检查端点

```bash
# 简单健康检查
curl http://localhost:8090/health
# 响应: OK

# 详细健康检查 (包含所有上游服务状态)
curl http://localhost:8090/health/detailed

# 服务发现状态
curl http://localhost:8090/health/discovery
```

### 9.4 熔断器状态

```bash
# 查看所有服务的熔断器状态 (需认证)
curl http://localhost:8090/circuit-breaker/status \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

## 10. Docker 构建与部署

### 10.1 多阶段构建

项目使用 `multi-stage.Dockerfile` 进行多阶段构建：

```dockerfile
# 构建阶段: Rust + protobuf-compiler
FROM rust:1.97.0-slim AS builder

# 运行阶段: Alpine Linux (静态编译，最小体积)
FROM ubuntu:26.10
```

### 10.2 构建单个服务镜像

```bash
# 构建 api-gateway 镜像
docker build -t localhost/myai/api-gateway:latest \
  --build-arg SERVICE_NAME=api-gateway \
  -f Backend/multi-stage.Dockerfile Backend

# 构建 auth-service 镜像
docker build -t localhost/myai/auth-service:latest \
  --build-arg SERVICE_NAME=auth-service \
  -f Backend/multi-stage.Dockerfile Backend
```

### 10.3 镜像体积优化

**优化前**: 每个镜像 ~3.28GB (复制整个编译目录)  
**优化后**: 每个镜像 ~100MB (仅复制单个二进制)

### 10.4 Helm Chart 部署

```bash
# 查看 Helm values
cat helm/myai/values-prod-harbor.yaml

# 部署
helm upgrade --install myai helm/myai/ \
  --set harbor.registry=localhost:5000 \
  --set namespace=myai \
  --values helm/myai/values-prod-harbor.yaml
```

---

## 11. 服务健康检查

### 11.1 健康检查脚本

```bash
#!/bin/bash
# health-check.sh - 服务健康检查脚本

SERVICES=(
    "api-gateway:http://localhost:8090/health"
    "auth-service:http://localhost:8081/health"
    "user-service:http://localhost:8080/health"
    # ... 其他服务
)

for service in "${SERVICES[@]}"; do
    name="${service%%:*}"
    url="${service##*:}"
    
    if curl -s --max-time 5 "$url" | grep -q "OK"; then
        echo "✓ $name 运行正常"
    else
        echo "✗ $name 异常"
    fi
done
```

### 11.2 Prometheus 指标

```bash
# 获取 Prometheus 格式指标
curl http://localhost:8090/metrics
```

---

## 12. 常见问题与排查

### 12.1 编译错误

**问题**: `sqlx::query!` 宏失败

```bash
# 解决方案: 准备 SQLx 查询缓存
export DATABASE_URL=postgresql://postgres:***@localhost:5432/myai
cargo sqlx prepare --all-targets

# 然后离线编译
export SQLX_OFFLINE=true
cargo build --release
```

**问题**: `protoc` 命令未找到

```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Windows
choco install protobuf
```

### 12.2 运行时错误

**问题**: `JWT_SECRET 环境变量必须配置`

```bash
# 解决方案: 设置安全的 JWT Secret
export JWT_SECRET=$(openssl rand -base64 48)
```

**问题**: `服务发现: 已注册 N 个上游服务，已连接 0 个`

```bash
# 解决方案: 检查 gRPC URL 环境变量
# 参考 .env.example 中的服务 gRPC URL 配置
export AUTH_SERVICE_GRPC_URL=auth-service:9091
export USER_SERVICE_GRPC_URL=user-service:9090
# ... 其他服务
```

**问题**: `数据库连接失败`

```bash
# 解决方案: 检查数据库连接
export DATABASE_URL=postgresql://postgres:***@localhost:5432/myai
psql -U postgres -d myai -c "SELECT 1"
```

### 12.3 限流问题

**现象**: 客户端收到 `429 Too Many Requests`

```bash
# 调整限流配置
export RATE_LIMIT_MAX=2000      # 增加最大请求数
export RATE_LIMIT_WINDOW=120     # 增加时间窗口
export RATE_LIMIT_BURST=200        # 增加突发量
```

### 12.4 熔断器问题

**现象**: 服务间调用频繁失败

```bash
# 调整熔断器配置
export CIRCUIT_BREAKER_FAILURE_THRESHOLD=10    # 增加失败阈值
export CIRCUIT_BREAKER_TIMEOUT_SECS=30     # 减少超时时间
export CIRCUIT_BREAKER_HALF_OPEN_REQUESTS=5    # 增加半开请求数
```

---

## 13. 附录

### 13.1 项目结构

```
MyAI/
├── Backend/
│   ├── Cargo.toml              # Workspace 配置
│   ├── Cargo.lock             # 依赖锁定文件
│   ├── multi-stage.Dockerfile  # 多阶段构建 Dockerfile
│   ├── .sqlx/                 # SQLx 查询缓存
│   ├── crates/               # 公共 crate
│   │   ├── common/            # 公共工具
│   │   ├── auth-core/          # 认证核心
│   │   ├── cache-core/          # 缓存核心
│   │   ├── grpc-core/           # gRPC 核心
│   │   ├── grpc-proto/           # gRPC Proto 定义
│   │   └── ...
│   ├── services/             # 微服务
│   │   ├── api-gateway/        # API 网关
│   │   ├── auth-service/      # 认证服务
│   │   ├── user-service/      # 用户服务
│   │   └── ... (21个服务)
│   ├── protos/               # Protobuf 源文件
│   ├── sql/                 # 数据库 Schema
│   └── template/             # Docker 模板
├── Frontend/
│   └── admin/               # Vue/Quasar 前端
├── helm/
│   └── myai/               # Helm Chart
├── deploy/
│   └── scripts/             # 部署脚本
├── .env.example          # 环境变量示例
└── Docs/
    └── Backend-编译和使用指南.md  # 本文档
```

### 13.2 依赖关系图

```
api-gateway
├── grpc-core (服务发现/客户端)
├── auth-core (JWT验证)
├── circuit-breaker-core (熔断器)
├── rate-limit-core (限流)
├── cache-core (缓存)
└── common (公共工具/错误/配置)

每个微服务
├── grpc-proto (Proto 定义)
├── grpc-core (gRPC 客户端)
├── auth-core (JWT验证)
├── cache-core (缓存)
├── common (公共工具)
└── sqlx (数据库)
```

### 13.3 关键配置项速查表

| 配置项 | 文件位置 | 说明 |
|----------------|---------|---------|
| JWT Secret | `.env` → `JWT_SECRET` | ≥32字符 |
| 数据库连接 | `.env` → `DATABASE_URL` | PostgreSQL DSN |
| Redis 连接 | `.env` → `REDIS_URL` | Redis DSN |
| 服务端口 | `.env` → `*_PORT` | HTTP/gRPC 端口 |
| 限流配置 | `.env` → `RATE_LIMIT_*` | 请求限流 |
| 熔断器配置 | `.env` → `CIRCUIT_BREAKER_*` | 服务熔断 |
| Schema 隔离 | `.env` → `SERVICE_SCHEMA` | 数据库 schema |
| 日志级别 | `.env` → `RUST_LOG` | `info`/`debug`/`warn` |

### 13.4 版本历史

| 版本 | 日期 | 说明 |
|----------------|---------|---------|
| v0.2.0 | 2026-08-11 | 本文档首次创建 |
| v0.1.0 | 2026-06-10 | 初始架构设计 |

---

## 快速命令参考

```bash
# 快速编译
cd Backend && SQLX_OFFLINE=true cargo build --release

# 快速运行 (单服务)
cd Backend/services/api-gateway && cargo run --release

# 快速启动 (Docker)
docker compose up -d

# 快速健康检查
curl http://localhost:8090/health

# 快速查看 API 文档
open http://localhost:8090/api/docs

# 快速查看服务状态
curl http://localhost:8090/health/detailed
```

---

**文档维护**: 如有更新请提交 PR 到 `Docs/` 目录  
**问题反馈**: 请在项目 Issue 中提交
