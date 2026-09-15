# MyAI Helm Chart

MyAI 微服务的 Kubernetes Helm Chart（release 名 `myai`，部署至 `myai-prod` 命名空间）。

## 结构

```
helm/myai/
├── Chart.yaml              # Chart 元信息（name: myai, version: 0.1.0）
├── values.yaml             # 默认配置
├── values-dev.yaml         # 开发环境覆盖
├── values-prod.yaml        # 生产环境覆盖
├── values-prod-harbor.yaml # 生产 + Harbor 镜像仓库配置（推荐/实际使用）
├── secrets.example.yaml    # 敏感信息示例
├── secret.yaml             # 敏感信息（实际值）
├── ENV_MAPPING.md          # 环境变量映射说明
└── templates/
    ├── _helpers.tpl        # 辅助函数（勿删，缺失导致 helm 渲染失败）
    ├── configmap.yaml      # 非敏感配置（含数据库连接池 DB_POOL_*）
    ├── secret.yaml         # 敏感配置（DATABASE_URL / REDIS_URL / JWT）
    ├── deployment.yaml     # 所有服务 Deployment
    ├── service.yaml        # 所有服务 Service
    ├── ingress.yaml        # Ingress（仅 HTTPS，websecure）
    ├── pvc.yaml            # 持久卷声明
    ├── pdb.yaml            # Pod 中断预算
    └── infrastructure.yaml # PostgreSQL / Redis（默认关闭，复用集群已有）
```

## 服务（22 个）

| 类型 | 服务 |
|------|------|
| 前端 | admin |
| 后端 | api-gateway, auth-service, user-service, cms-service, messaging-service, file-service, tenant-service, workflow-service, feedback-service, audit-service, api-key-service, hik-service, pay-service, clean-service, tow-service, ctp-service, xlt-service, ebike-service, lpr-service, social-ops-service, browser-service |

## 部署

### 前置

- 已连接 Rancher 集群（kubectl 可用）
- Harbor 凭据已配置（`harbor-secret` 或全局 imagePullSecrets）

### 部署命令

```bash
cd D:\Workspace\MyAI\helm

# 生产部署（推荐 values-prod-harbor.yaml）
helm upgrade --install myai ./myai \
  --namespace myai-prod \
  -f ./myai/values-prod-harbor.yaml \
  --set "image.tag=YYYYMMDDHHMM" \
  --timeout 10m
```

> ⚠️ 完整流程（构建/推送/部署）见 `docs/build-deploy-guide.md`，或直接运行：
> ```bash
> bash ../deploy/scripts/deploy-to-rancher.sh <YYYYMMDDHHMM>
> ```

### 关键配置

| 配置 | 值 | 说明 |
|------|-----|------|
| `global.imageRegistry` | `harbor.100.100.100.101.example.com:8003/erp` | Harbor 仓库 |
| `image.tag` | `202608070337` | 镜像标签（格式 YYYYMMDDHHMM，纯数字 12 位） |
| `ingress.className` | `traefik` | Ingress 控制器 |
| `ingress.annotations` | `router.entrypoints: websecure` | **仅 HTTPS** |
| `infrastructure.postgres.enabled` | `false` | 复用集群已有 pg-cluster-postgresql |
| `infrastructure.redis.enabled` | `false` | 复用集群已有 redis-replication |
| `configMap.databasePool` | max 3 / min 1 | 数据库连接池（22 服务共享） |

### 敏感配置（Secret）

`secret.yaml` / `values-prod-harbor.yaml` 的 `secret` 段包含：

| 变量 | 说明 |
|------|------|
| `DATABASE_URL` | PostgreSQL 连接串（库名 `myai`） |
| `REDIS_URL` | Redis 连接地址 |
| `JWT_SECRET` / `JWT_ISSUER` / `JWT_AUDIENCE` | JWT 配置 |
| 各业务密钥 | S3/OSS/HIK/PAY 等（可留空） |

> ⚠️ Secret 变更后需 rollout restart（运行中 Pod 不自动刷新 envFrom）：
> ```bash
> kubectl rollout restart deployment -n myai-prod -l app.kubernetes.io/name=myai
> ```

## 运维命令

```bash
# 查看状态
kubectl get pods -n myai-prod
kubectl get all -n myai-prod

# 查看日志
kubectl logs -f deployment/myai-api-gateway -n myai-prod

# 升级（tag 纯数字需加引号）
helm upgrade myai ./myai -n myai-prod -f ./myai/values-prod-harbor.yaml --set "image.tag=YYYYMMDDHHMM"

# 回滚
helm rollback myai -n myai-prod

# 卸载
helm uninstall myai -n myai-prod
```

## 常见问题

| 问题 | 处理 |
|------|------|
| `file-service` 卡 ContainerCreating（Multi-Attach） | 删旧 Pod + 删旧 RS，见 build-deploy-guide §7.3 |
| Pod ImagePullBackOff | 确认 Harbor 有对应 tag |
| Secret 更新不生效 | rollout restart（envFrom 不自动刷新） |
| `_helpers.tpl` 缺失 | helm 渲染报 "no template myai.labels" |
