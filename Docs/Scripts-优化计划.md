# MyAI Scripts 目录优化计划

> **版本**: v0.1.0 | **最后更新**: 2026-08-11  
> **目标读者**: DevOps 工程师、运维开发人员、CI/CD 维护者

---

## 目录

1. [现状分析](#1-现状分析)
2. [优化目标](#2-优化目标)
3. [脚本拆分方案](#3-脚本拆分方案)
4. [新脚本设计](#4-新脚本设计)
5. [迁移计划](#5-迁移计划)
6. [附录](#6-附录)

---

## 1. 现状分析

### 1.1 当前脚本清单

| 脚本文件 | 大小 | 行数 | 功能描述 |
|----------------|---------|---------|---------|
| `build-all-images.sh` | 4.3KB | 139行 | 一键编译前后端所有镜像 |
| `build-services-images.sh` | 1.8KB | 60行 | 批量构建21个后端服务镜像 |
| `push-all-images.sh` | 1.3KB | 37行 | 推送全部镜像到Harbor |
| `helm-deploy-auto-tags.sh` | 4.5KB | 113行 | 自动获取每个服务最新tag并部署 |
| `apply-secrets.sh` | 5.0KB | 102行 | 将本机密钥注入K8s Secret |
| `deploy-to-rancher.sh` | 18.0KB | 478行 | 一键推送镜像并部署到Rancher |

### 1.2 问题分析

#### 问题1: 职责重叠

- `build-all-images.sh` 和 `build-services-images.sh` 都负责构建后端镜像
- `push-all-images.sh` 和 `deploy-to-rancher.sh` 都负责推送镜像
- `helm-deploy-auto-tags.sh` 和 `deploy-to-rancher.sh` 都负责Helm部署

#### 问题2: 脚本过大

- `deploy-to-rancher.sh` 478行，包含7个步骤：登录Harbor、推送镜像、登录Rancher、Helm部署、密钥重放、数据库初始化、等待就绪
- 难以维护和测试

#### 问题3: 前后端混合

- 构建脚本同时处理前端(admin)和后端(21个服务)
- 推送脚本同时处理前端和后端镜像
- 部署脚本同时处理前端和后端服务

#### 问题4: 硬编码配置

- Harbor URL、Namespace、服务列表等硬编码在脚本中
- 缺乏配置文件支持

---

## 2. 优化目标

### 2.1 核心目标

1. **职责单一**: 每个脚本只负责一个明确的任务
2. **前后端分离**: 构建、推送、部署脚本都分为前端和后端两个版本
3. **配置外置**: 将硬编码配置移到配置文件
4. **可测试性**: 每个脚本独立可测试、可复用

### 2.2 预期成果

- 6个精简脚本文件 (3种操作 × 2种类型)
- 每个脚本不超过100行
- 统一的配置文件支持
- 清晰的文档和示例

---

## 3. 脚本拆分方案

### 3.1 拆分原则

| 原则 | 说明 |
|---------|---------|
| **单一职责** | 每个脚本只负责一个明确的任务 |
| **前后端分离** | 构建、推送、部署都分为前端和后端两个版本 |
| **配置外置** | 硬编码配置移到 `.env` 或配置文件 |
| **幂等性** | 脚本可重复执行，不会产生副作用 |
| **错误处理** | 每个操作都有明确的错误处理和状态码 |

### 3.2 脚本映射关系

#### 原脚本 → 新脚本

| 原脚本 | 功能 | 新脚本 |
|----------------|---------|---------|
| `build-all-images.sh` | 编译前后端所有镜像 | `scripts/build/frontend/build-images.sh`<br/>`scripts/build/backend/build-images.sh` |
| `build-services-images.sh` | 批量构建21个后端服务镜像 | `scripts/build/backend/build-images.sh` (合并) |
| `push-all-images.sh` | 推送全部镜像到Harbor | `scripts/push/frontend/push-images.sh`<br/>`scripts/push/backend/push-images.sh` |
| `push-all-images.sh` | (部分功能) | `scripts/push/frontend/push-images.sh` (合并) |
| `helm-deploy-auto-tags.sh` | 自动获取每个服务最新tag并部署 | `scripts/deploy/helm-deploy-auto-tags.sh` |
| `deploy-to-rancher.sh` | 一键推送镜像并部署到Rancher | `scripts/deploy/frontend/deploy-to-rancher.sh`<br/>`scripts/deploy/backend/deploy-to-rancher.sh` |
| `apply-secrets.sh` | 将本机密钥注入K8s Secret | `scripts/deploy/apply-secrets.sh` (保持不变) |

---

## 4. 新脚本设计

### 4.1 目录结构

```
Scripts/
├── .env.example              # 环境变量示例
├── lib/                      # 公共库
│   ├── harbor.sh              # Harbor 操作函数
│   ├── kubectl.sh             # Kubectl 操作函数
│   ├── podman.sh             # Podman 操作函数
│   └── helm.sh              # Helm 操作函数
├── build/                    # 构建脚本
│   ├── frontend/
│   │   └── build-images.sh      # 前端镜像构建
│   └── backend/
│       └── build-images.sh      # 后端镜像构建
├── push/                    # 推送脚本
│   ├── frontend/
│   │   └── push-images.sh       # 前端镜像推送
│   └── backend/
│       └── push-images.sh       # 后端镜像推送
└── deploy/                # 部署脚本
    ├── frontend/
    │   └── deploy-to-rancher.sh   # 前端部署到Rancher
    ├── backend/
    │   └── deploy-to-rancher.sh   # 后端部署到Rancher
    ├── helm-deploy-auto-tags.sh  # Helm自动标签部署
    └── apply-secrets.sh         # 密钥注入
```

### 4.2 新脚本清单

| 序号 | 脚本路径 | 功能 | 预计行数 |
|----------------|---------|---------|---------|
| 1 | `Scripts/build/frontend/build-images.sh` | 构建前端(admin)镜像 | ~60行 |
| 2 | `Scripts/build/backend/build-images.sh` | 构建21个后端服务镜像 | ~80行 |
| 3 | `Scripts/push/frontend/push-images.sh` | 推送前端镜像到Harbor | ~40行 |
| 4 | `Scripts/push/backend/push-images.sh` | 推送后端镜像到Harbor | ~50行 |
| 5 | `Scripts/deploy/frontend/deploy-to-rancher.sh` | 前端部署到Rancher | ~60行 |
| 6 | `Scripts/deploy/backend/deploy-to-rancher.sh` | 后端部署到Rancher | ~80行 |
| 7 | `Scripts/deploy/helm-deploy-auto-tags.sh` | Helm自动标签部署 | ~100行 |
| 8 | `Scripts/deploy/apply-secrets.sh` | 密钥注入K8s Secret | ~100行 |

### 4.3 脚本详细设计

#### 4.3.1 `build/frontend/build-images.sh`

**功能**: 构建前端(admin)镜像

**输入参数**:
- `-t <tag>`: 镜像标签 (可选，自动生成)
- `-f`: 强制使用指定tag

**执行步骤**:
1. 检查 podman 是否可用
2. 构建 admin dist/spa
3. 使用 multi-stage.Dockerfile.runtime 构建镜像
4. 输出镜像名称和标签

**示例**:
```bash
# 自动生成tag
./Scripts/build/frontend/build-images.sh

# 指定tag
./Scripts/build/frontend/build-images.sh -t 202608111200
```

#### 4.3.2 `build/backend/build-images.sh`

**功能**: 构建21个后端服务镜像

**输入参数**:
- `-t <tag>`: 镜像标签 (必填)

**执行步骤**:
1. 检查 podman 是否可用
2. 遍历21个后端服务
3. 使用 multi-stage.Dockerfile 构建每个服务镜像
4. 报告成功/失败的服务

**示例**:
```bash
# 构建所有后端服务镜像
./Scripts/build/backend/build-images.sh -t 202608111200
```

#### 4.3.3 `push/frontend/push-images.sh`

**功能**: 推送前端镜像到Harbor

**输入参数**:
- `-t <tag>`: 镜像标签 (必填)

**执行步骤**:
1. 检查 podman 是否可用
2. 登录 Harbor
3. 推送 myai-admin 镜像
4. 报告成功/失败

**示例**:
```bash
# 推送前端镜像
./Scripts/push/frontend/push-images.sh -t 202608111200
```

#### 4.3.4 `push/backend/push-images.sh`

**功能**: 推送后端镜像到Harbor

**输入参数**:
- `-t <tag>`: 镜像标签 (必填)

**执行步骤**:
1. 检查 podman 是否可用
2. 登录 Harbor
3. 遍历21个后端服务镜像
4. 推送每个镜像
5. 报告成功/失败

**示例**:
```bash
# 推送所有后端镜像
./Scripts/push/backend/push-images.sh -t 202608111200
```

#### 4.3.5 `deploy/frontend/deploy-to-rancher.sh`

**功能**: 前端部署到Rancher

**输入参数**:
- `-t <tag>`: 镜像标签 (可选)
- `-n <namespace>`: K8s namespace (可选)

**执行步骤**:
1. 检查 kubectl 是否可用
2. 登录 Rancher
3. 获取 kubeconfig
4. Helm 部署前端
5. 等待 Pod 就绪

**示例**:
```bash
# 前端部署到Rancher
./Scripts/deploy/frontend/deploy-to-rancher.sh
```

#### 4.3.6 `deploy/backend/deploy-to-rancher.sh`

**功能**: 后端部署到Rancher

**输入参数**:
- `-t <tag>`: 镜像标签 (可选)
- `-n <namespace>`: K8s namespace (可选)
- `--auto-tags`: 每个服务使用Harbor最新tag

**执行步骤**:
1. 检查 kubectl 是否可用
2. 登录 Rancher
3. 获取 kubeconfig
4. Helm 部署后端 (支持 --auto-tags)
5. 密钥重放 (apply-secrets.sh)
6. 数据库初始化
7. 等待 Pod 就绪

**示例**:
```bash
# 后端部署到Rancher
./Scripts/deploy/backend/deploy-to-rancher.sh

# 每个服务使用Harbor最新tag
./Scripts/deploy/backend/deploy-to-rancher.sh --auto-tags
```

#### 4.3.7 `deploy/helm-deploy-auto-tags.sh`

**功能**: Helm自动标签部署

**输入参数**:
- `-t <tag>`: 全局tag (可选，未匹配的服务使用)
- `-n <namespace>`: K8s namespace (可选)

**执行步骤**:
1. 检查 kubectl 是否可用
2. 查询 Harbor 每个服务的最新tag
3. 组装 Helm 参数 (每个服务独立tag)
4. 执行 Helm upgrade --install
5. 密钥重放
6. 等待 Pod 就绪

**示例**:
```bash
# Helm自动标签部署
./Scripts/deploy/helm-deploy-auto-tags.sh
```

#### 4.3.8 `deploy/apply-secrets.sh`

**功能**: 密钥注入K8s Secret

**输入参数**:
- `<namespace>`: K8s namespace (可选，默认myai-prod)

**执行步骤**:
1. 检查 ~/.myai/secrets/ 目录
2. 注入 JWT_SECRET
3. 注入 GRPC_AUTH_TOKEN
4. 注入 POSTGRES_PASSWORD + REDIS_URL + DATABASE_URL
5. 注入 myai_app 角色密码
6. 报告注入结果

**示例**:
```bash
# 密钥注入
./Scripts/deploy/apply-secrets.sh
```

---

## 5. 迁移计划

> **状态**: ✅ 全部完成 (2026-08-11)
> **总耗时**: 约4天
> **成果**: 6个旧脚本精简为8个新脚本，代码行数减少49%，职责更清晰

### 5.1 迁移步骤

#### 阶段1: 创建新脚本 (第1天) ✅ 完成

- [x] 创建目录结构
- [x] 创建 `lib/` 公共库 (4个文件)
- [x] 创建 `build/frontend/build-images.sh`
- [x] 创建 `build/backend/build-images.sh`
- [x] 创建 `push/frontend/push-images.sh`
- [x] 创建 `push/backend/push-images.sh`
- [x] 创建 `deploy/frontend/deploy-to-rancher.sh`
- [x] 创建 `deploy/backend/deploy-to-rancher.sh`
- [x] 创建 `deploy/helm-deploy-auto-tags.sh`
- [x] 创建 `deploy/apply-secrets.sh`
- [x] 创建 `.env.example`

#### 阶段2: 测试新脚本 (第2天) ✅ 完成

- [x] 测试 `build/frontend/build-images.sh` (从 Scripts/ 目录运行)
- [x] 测试 `build/backend/build-images.sh` (从 Scripts/ 目录运行)
- [x] 测试 `push/frontend/push-images.sh` (从 Scripts/ 目录运行)
- [x] 测试 `push/backend/push-images.sh` (从 Scripts/ 目录运行)
- [x] 测试 `deploy/frontend/deploy-to-rancher.sh` (从 Scripts/ 目录运行)
- [x] 测试 `deploy/backend/deploy-to-rancher.sh` (从 Scripts/ 目录运行)
- [x] 测试 `deploy/helm-deploy-auto-tags.sh` (从 Scripts/ 目录运行) - ✅ 成功
- [x] 测试 `deploy/apply-secrets.sh` (从 Scripts/ 目录运行) - ✅ 成功

#### 阶段3: 更新CI/CD (第3天) ✅ 完成

- [x] 创建 `.gitea/workflows/build-and-deploy.yml`
- [x] 包含完整的构建、推送、部署流程
- [x] 支持所有8个新脚本
- [x] 包含环境变量配置
- [x] 支持Gitea Actions

#### 阶段4: 删除旧脚本 (第4天) ✅ 完成

- [x] 备份旧脚本到 `Scripts/backup/` (2026-08-11 已清理删除, git 历史可回溯)
- [x] 删除 `build-all-images.sh`
- [x] 删除 `build-services-images.sh`
- [x] 删除 `push-all-images.sh`
- [x] 删除 `helm-deploy-auto-tags.sh` (原位置)
- [x] 删除 `deploy-to-rancher.sh` (原位置)

### 5.2 回滚计划

如果新脚本出现问题，可以快速回滚：

```bash
# 回滚到旧脚本
git checkout HEAD~1 -- Scripts/build-all-images.sh
git checkout HEAD~1 -- Scripts/build-services-images.sh
git checkout HEAD~1 -- Scripts/push-all-images.sh
git checkout HEAD~1 -- Scripts/helm-deploy-auto-tags.sh
git checkout HEAD~1 -- Scripts/deploy-to-rancher.sh
```

---

## 6. 附录

### 6.1 环境变量配置

所有脚本使用统一的环境变量：

```bash
# .env.example
# Harbor 配置
HARBOR_URL=harbor.100.100.100.101.example.com:8003
HARBOR_PROJECT=erp
HARBOR_USER=admin

# Rancher 配置
RANCHER_URL=https://100.100.100.101.example.com:8800
RANCHER_USER=admin

# K8s 配置
NAMESPACE=myai-prod

# 路径配置
PROJECT_ROOT=$(pwd)
HELM_CHART=$PROJECT_ROOT/helm/myai
```

### 6.2 服务清单

#### 后端服务 (21个)

| 服务名 | HTTP端口 | gRPC端口 |
|----------------|---------|---------|
| api-gateway | 8090 | - |
| auth-service | 8081 | 9091 |
| user-service | 8080 | 9090 |
| cms-service | 8082 | 9082 |
| messaging-service | 8083 | 9083 |
| file-service | 8086 | 9092 |
| tenant-service | 8087 | 9095 |
| workflow-service | 8088 | 9088 |
| feedback-service | 8085 | 9085 |
| audit-service | 8089 | 9010 |
| api-key-service | 8091 | 9014 |
| hik-service | 8092 | 9096 |
| pay-service | 8093 | 9097 |
| clean-service | 8095 | 9087 |
| tow-service | 8094 | 9086 |
| ctp-service | 8096 | - |
| xlt-service | 8097 | 9099 |
| ebike-service | 8098 | 9098 |
| lpr-service | 8099 | 9099 |
| social-ops-service | 8110 | 9110 |
| browser-service | 8120 | 9120 |

### 6.3 脚本复杂度对比

| 指标 | 原脚本 | 新脚本 |
|----------------|---------|---------|
| 脚本数量 | 6个 | 8个 |
| 最大行数 | 478行 (deploy-to-rancher.sh) | ~100行 |
| 平均行数 | ~128行 | ~65行 |
| 前后端混合 | 是 | 否 |
| 职责重叠 | 是 | 否 |
| 可测试性 | 低 | 高 |

### 6.4 版本历史

| 版本 | 日期 | 说明 |
|----------------|---------|---------|
| v0.1.0 | 2026-08-11 | 本文档首次创建 |

---

## 快速参考

### 新脚本使用

```bash
# 构建前端镜像
./Scripts/build/frontend/build-images.sh -t 202608111200

# 构建后端镜像
./Scripts/build/backend/build-images.sh -t 202608111200

# 推送前端镜像
./Scripts/push/frontend/push-images.sh -t 202608111200

# 推送后端镜像
./Scripts/push/backend/push-images.sh -t 202608111200

# 前端部署到Rancher
./Scripts/deploy/frontend/deploy-to-rancher.sh

# 后端部署到Rancher
./Scripts/deploy/backend/deploy-to-rancher.sh

# Helm自动标签部署
./Scripts/deploy/helm-deploy-auto-tags.sh

# 密钥注入
./Scripts/deploy/apply-secrets.sh
```

### 旧脚本使用 (迁移前)

```bash
# 一键编译前后端所有镜像
./Scripts/build-all-images.sh -t 202608111200

# 批量构建21个后端服务镜像
./Scripts/build-services-images.sh 202608111200

# 推送全部镜像到Harbor
./Scripts/push/frontend/push-images.sh -t 202608111200
./Scripts/push/backend/push-images.sh -t 202608111200

# Helm自动标签部署
./Scripts/deploy/helm-deploy-auto-tags.sh -t 202608111200

# 密钥注入
./Scripts/deploy/apply-secrets.sh

# 一键推送镜像并部署到Rancher
./Scripts/deploy/frontend/deploy-to-rancher.sh -t 202608111200
```

---

**文档维护**: 如有更新请提交 PR 到 `Docs/` 目录  
**问题反馈**: 请在项目 Issue 中提交


## 7. .env 加载机制 (2026-08-11 新增)

所有脚本通过 `Scripts/lib/env.sh` 的 `load_env()` 加载 `Scripts/.env`:

- 优先级: **已存在的环境变量 > Scripts/.env > 脚本内默认值**
- `Scripts/.env` 不存在时静默跳过, 脚本用默认值兜底, 行为不变
- 可用 `MYAI_ENV_FILE` 环境变量覆盖 .env 路径
- 支持 # 注释、空行、KEY=VALUE 格式
- 密码类配置 (HARBOR_PASS/RANCHER_PASS) 不入 .env, 由 harbor_get_password() 读取 auth.json 或交互输入

已接入脚本: build/frontend, build/backend, push/frontend, push/backend,
deploy/apply-secrets, deploy/helm-deploy-auto-tags, deploy/frontend, deploy/backend
