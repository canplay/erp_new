# MyAI 部署脚本

## 目录结构

```
Scripts/
├── build/          # 镜像构建
│   ├── backend/     # 后端服务镜像构建
│   └── frontend/    # 前端镜像构建
├── deploy/          # 部署到K8s/Rancher
│   ├── backend/     # 后端部署
│   ├── frontend/   # 前端部署
│   ├── apply-secrets.sh  # 密钥注入
│   └── helm-deploy-auto-tags.sh  # Helm自动标签
├── helm/              # Helm Chart
│   └── myai/
├── lib/               # 公共库
│   ├── project-root.sh  # 项目根目录定位
│   ├── logging.sh      # 统一日志
│   ├── services.sh    # 服务列表
│   ├── wait-for-pods.sh  # Pod就绪等待
│   ├── env.sh           # 环境变量
│   ├── harbor.sh       # Harbor操作
│   ├── helm.sh         # Helm操作
│   ├── kubectl.sh       # kubectl操作
│   ├── podman.sh       # Podman操作
│   ├── rancher.sh       # Rancher认证
│   ├── exit-codes.sh    # 统一退出码
│   ├── check-scripts.sh  # 脚本一致性检查
│   ├── secrets.sh      # 密码读取函数
│   ├── all.sh           # 统一导出所有公共函数
│   ├── shellcheck.sh    # ShellCheck 检查
│   ├── validate.sh    # 参数验证函数
│   ├── version.sh     # 版本管理函数
│   └── check-licenses.sh # 许可证检查
└── README.md
```

## 快速开始

### 构建镜像

```bash
# 构建后端镜像
./Scripts/build/backend/build-images.sh -t 202608171700

# 构建前端镜像
./Scripts/build/frontend/build-images.sh -t 202608171700
```

### 部署到Rancher

```bash
# 后端部署
./Scripts/deploy/backend/deploy-to-rancher.sh -t 202608171700

# 前端部署
./Scripts/deploy/frontend/deploy-to-rancher.sh -t 202608171700
```

## 环境变量

| 变量 | 说明 | 默认值 |
|---|---|---|
| `MYAI_ROOT` | 项目根目录 | 自动定位 |
| `LOG_LEVEL` | 日志级别 | `info` |
| `HARBOR_URL` | Harbor地址 | `harbor.100.100.100.101.example.com:8003` |
| `RANCHER_URL` | Rancher地址 | `https://100.100.100.101.example.com:8800` |
| `NAMESPACE` | K8s命名空间 | `myai-prod` |

## CI/CD 集成

所有脚本支持通过环境变量提供密码，无需交互式输入：

```bash
export HARBOR_PASSWORD=xxx
export RANCHER_PASSWORD=xxx
export MYAI_ROOT=/path/to/MyAI

# 非交互式构建+部署
./Scripts/build/backend/build-images.sh -t 202608171700
./Scripts/deploy/backend/deploy-to-rancher.sh -t 202608171700
```

## 脚本一致性检查

运行以下命令检查所有脚本的一致性：

```bash
./Scripts/lib/check-scripts.sh
```

## 注意事项

1. 所有脚本使用 `set -euo pipefail` 确保错误时退出
2. 项目根目录自动定位，也可通过 `MYAI_ROOT` 环境变量覆盖
3. 日志级别可通过 `LOG_LEVEL` 环境变量控制（debug/info/warn/error）
4. 密码可通过环境变量或密钥文件（~/.myai/secrets/）提供
