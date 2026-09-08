# 安全策略

## 版本支持

| 版本 | 支持状态 |
|------|----------|
| 最新发行版 | ✅ 安全修复 |
| 旧版本 | ❌ 请升级至最新版 |

## 报告漏洞

如发现安全漏洞，请**不要**通过 Issue 或 PR 公开报告。

请通过以下方式私下联系维护者：

- **邮箱**：[维护者邮箱]（待配置，请先通过 GitHub Security Advisory 报告）
- **GitHub Security Advisory**：[创建私有报告](https://github.com/canplay/myai/security/advisories/new)

报告请包含：

1. 漏洞描述与影响范围
2. 复现步骤（PoC 代码或截图）
3. 修复建议（如有）

维护者将在 **48 小时内** 确认收到，**7 天内** 给出修复计划或拒绝理由。

## 安全最佳实践

部署本项目时，请务必遵循以下安全实践：

### 1. 密钥管理

- 生产环境部署前，必须替换 `.env.example` 中的所有占位密钥：
  ```bash
  # JWT 密钥（≥ 32 字符，Base64 编码）
  openssl rand -base64 48

  # 会话密钥
  openssl rand -base64 32

  # 数据库密码
  openssl rand -base64 24
  ```
- **切勿**将真实密钥提交到 Git 仓库
- 使用专用密钥管理服务（Vault / AWS Secrets Manager / K8s Secrets）

### 2. 数据库安全

- 为 `myai_app` 用户配置最小权限（GRANT 仅限必要表）
- 启用 SSL/TLS 连接（`sslmode=require`）
- 定期备份（WAL-G / pg_dump）
- 启用审计日志（pgaudit）

### 3. 网络安全

- Ingress 强制 HTTPS（HSTS 头部）
- 内网服务使用 NetworkPolicy 隔离
- gRPC 服务启用 mTLS（Istio / Linkerd）
- API 网关启用 Rate Limiting + Circuit Breaker

### 4. 容器安全

- 镜像构建使用多阶段构建 + distroless/chiseled 基础镜像
- 扫描镜像漏洞（Trivy / Grype）
- 以非 root 用户运行（`USER 1000`）
- 启用 Seccomp / AppArmor 配置文件

## 安全相关配置清单

部署前请确认以下配置：

- [ ] JWT_SECRET 已替换为强随机密钥（≥ 32 字符）
- [ ] 数据库密码已替换（非默认值）
- [ ] Redis 密码已设置（如启用）
- [ ] 生产环境关闭调试模式（`RUST_LOG=info`）
- [ ] CORS 白名单已配置（仅允许前端域名）
- [ ] Rate Limiting 已启用（防止暴力破解）
- [ ] 日志脱敏已启用（密码/Token 不输出）

## 已知限制

- 当前版本未实现 OAuth2/OIDC 集成（计划中）
- 审计日志仅记录操作元数据，不记录请求体

## 致谢

感谢以下安全研究人员负责任地披露漏洞：

（暂无）
