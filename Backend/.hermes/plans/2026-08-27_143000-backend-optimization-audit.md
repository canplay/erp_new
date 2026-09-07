# MyAI 后端深度优化与完善计划

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task (each workstream is independent and can be parallelized across subagents).

**Goal:** 对 MyAI 后端（21 个微服务 + 10 个共享 crate，Rust / tokio 1.52 / tonic 0.14 / sqlx 0.8 / axum 0.8）做一轮"性能提升、安全加固、合并冗余、删除无用/过时"的体检式优化，消除已知技术债并量化收益。

**Architecture:** 当前为 Cargo workspace（edition 2024）。共享 crate 几乎未被复用（`rate-limit-core` 0 服务依赖、`circuit-breaker-core`/`log-core`/`crypto-core` 各 1、`grpc-core` 2），每个服务都在自己的 `main.rs`/`lib.rs` 里重复写启动骨架与 gRPC 装配。优化策略分四条线并行推进：P0 安全（硬编码密钥、CORS、运行时 SQL）、P1 性能（tokio 特性裁剪、连接池、N+1、tracing 采样）、P2 冗余合并（统一启动骨架、grpc client 配置集中）、P3 清理（死 crate、未用 allow、println、过时依赖声明）。

**Tech Stack:** Rust 2024 edition · tokio · tonic 0.14 · sqlx 0.8 (postgres) · axum 0.8 · tower-http (cors) · redis · opentelemetry.

---

## 现状基线（已实测证据）

| 维度 | 实测数据 | 位置/来源 |
|------|----------|-----------|
| 服务 / 共享 crate 数 | 21 services / 10 crates | `Cargo.toml` workspace members |
| 死 crate（0 服务依赖） | `rate-limit-core`（0）、`circuit-breaker-core`(1)、`log-core`(1)、`crypto-core`(1)、`grpc-core`(2) | grep `Cargo.toml` |
| 硬编码密钥 | `ctp_device.rs` 中 `app_secret: "secret456"/"secret1"/"secret2"/"secret"`；`auth-core/middleware.rs:185,215` `secret: "your-secret-key-change-in-production"` | 见 Task S1/S2 |
| `unwrap/expect/panic/println` | api-gateway 44、workflow 33、pay 20、ebike 15… 共 ~190；其中 `println!` 17、`allow(dead_code/unused)` 44、TODO/FIXME 6 | grep 统计 |
| SQL 调用 | `sqlx::query!` 宏（编译期安全）285 处；**运行时 `sqlx::query(`（字符串拼接、无编译期校验）22 处** | grep 统计 |
| PgPool 创建点 | 分散 4 处（ebike 自写 1 + common 2 + 其它 1）；max_connections 默认 10 | `crates/common/src/config/database.rs`、`ebike-service/src/lib.rs` |
| CORS | api-gateway `cors.rs` 用 `allow_methods(Any)`（放行所有 HTTP 方法） | `services/api-gateway/src/middleware/cors.rs:62` |
| tokio 特性 | `features = ["full", ...]` 全量开启（含 fs/io-std/process 等后端用不到的） | `Cargo.toml:54` |
| 依赖声明方式 | 12 个服务直接写死版本（如 `futures-util = "0.3"`）而非 `workspace = true` → 版本漂移风险 | grep `Cargo.toml` |
| tokio::spawn | 24 处，其中 19 处 fire-and-forget 无错误处理 | grep `tokio::spawn` |
| 启动骨架重复 | 6+ 服务各自实现 `main.rs` + `impl GrpcServiceBuilder` | grep 统计 |

---

## 工作流总览

- **P0 安全加固（必须优先）** — S1~S4
- **P1 性能提升** — P1a~P1d
- **P2 合并冗余** — R1~R3
- **P3 删除无用/过时** — C1~C4
- **收尾验证** — V1

> 每条 Task 颗粒度 2–5 分钟，含确切文件路径、可执行命令与预期输出。改完一类即提交一次（frequent commits）。

---

## P0 安全加固

### Task S1: 清除 ctp-service 中的硬编码设备密钥
**Objective:** 将源码里写死的 `app_secret` 改为从环境变量/配置读取，杜绝密钥入库。
**Files:** Modify `services/ctp-service/src/services/ctp_device.rs:310,340,354,368,374,397`
**Step 1:** 在 `ctp_device.rs` 顶部引入配置读取（参考 `crates/common/src/config/mod.rs` 的 env 读取模式），新增：
```rust
fn app_secret_for(id: &str) -> String {
    std::env::var(format!("CTP_APP_SECRET_{}", id.to_uppercase()))
        .unwrap_or_else(|_| std::env::var("CTP_APP_SECRET_DEFAULT").unwrap_or_default())
}
```
**Step 2:** 将各 `app_secret: "secret456".to_string()` 改为 `app_secret: app_secret_for("...").to_string()`。
**Step 3:** 验证 `cargo check -p ctp-service`，预期：0 error。
**Step 4:** 提交 `git commit -m "security: ctp device secret from env (remove hardcoded)"`。

### Task S2: auth-core JWT 默认密钥改为强制从环境注入
**Objective:** 防止误用默认密钥 `your-secret-key-change-in-production` 上线。
**Files:** Modify `crates/auth-core/src/middleware.rs:185,215`
**Step 1:** 改为 `let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");`（或返回 `Result` 由调用方处理）。
**Step 2:** `cargo check -p auth-core` 预期 0 error；补充单测验证缺失 `JWT_SECRET` 时启动报错。
**Step 3:** 提交。

### Task S3: gRPC 服务间鉴权/TLS 复核（核对既有 GRPC_AUTH_TOKEN）
**Objective:** 确认服务间 gRPC 调用已启用鉴权头（已知 `GRPC_AUTH_TOKEN` 已落地），对仍走明文默认 URL 的 client 补齐鉴权注入。
**Files:** Inspect `services/api-gateway/src/grpc_clients.rs`、`crates/grpc-core/src/client.rs`
**Step 1:** 读 `grpc_clients.rs` 的 `ServiceDef` 与调用处，确认每个 `tonic::transport::Channel` 创建后都注入 `Authorization` / 自定义 metadata。
**Step 2:** 若缺失，在 `grpc-core` 的 client 构造统一注入（集中一处，避免每个服务重复）。
**Step 3:** `cargo check --workspace` 预期无新增 error。

### Task S4: 收紧 CORS 方法白名单
**Objective:** 将 `allow_methods(Any)` 改为显式允许的方法（GET/POST/PUT/DELETE/OPTIONS/PATCH）。
**Files:** Modify `services/api-gateway/src/middleware/cors.rs:62`
**Step 1:**
```rust
use tower_http::cors::{CorsLayer, Any, Method};
let methods = ["GET","POST","PUT","DELETE","OPTIONS","PATCH"]
    .iter().map(|m| m.parse::<Method>().unwrap()).collect::<Vec<_>>();
let mut cors = CorsLayer::new().allow_methods(methods);
```
**Step 2:** `cargo check -p api-gateway` 预期 0 error。
**Step 3:** 提交。

---

## P1 性能提升

### Task P1a: 裁剪 tokio 全量特性
**Objective:** `tokio = ["full"]` 拉入后端用不到的 fs/process/io-std，增大编译体积与攻击面。
**Files:** Modify `Cargo.toml:54`（workspace.dependencies）
**Step 1:** 改 `tokio` 特性为最小集（保留 net/sync/time/rt-multi-thread/rt/io-util/macros/fs 按需）：
```toml
tokio = { version = "1.52", features = ["rt-multi-thread", "net", "sync", "time", "io-util", "macros", "signal"] }
```
**Step 2:** `cargo check --workspace` 定位因缺特性报错的代码（如用到 `tokio::fs` 则补 `fs`），逐个补全/改写。
**Step 3:** 记录编译时间前后对比（`cargo check` 计时），预期编译体积下降、无功能回归。

### Task P1b: 统一 PgPool 创建与连接数调优
**Objective:** 消除 4 处分散的池创建，统一走 `common` 的 `create_pool`，并按服务负载设置 `max_connections`。
**Files:** Inspect `crates/common/src/config/database.rs:192,207`、`services/ebike-service/src/lib.rs:97`
**Step 1:** ebike-service 的自写池改为调用 `common::create_pool()`（保留其 `DB_POOL_MAX_CONNECTIONS` 读取）。
**Step 2:** 在 `common` 暴露 `max_connections` 可配；默认 10 不改行为，仅统一入口。
**Step 3:** `cargo check -p ebike-service` + workspace 验证。

### Task P1c: 排查 N+1 与运行时 `sqlx::query(` 注入点
**Objective:** 22 处运行时 `sqlx::query(` 无编译期校验，是性能与安全的双重风险。
**Files:** 列表见 `grep -rnE "sqlx::query\(" services crates --include=*.rs`
**Step 1:** 逐文件列出 22 处，将可静态化者改为 `query!`/`query_as!` 宏（需 `DATABASE_URL` 与离线缓存 `.sqlx`）。
**Step 2:** 对确须动态 SQL 者，确认参数均用 bind（无字符串拼接）。
**Step 3:** `cargo check` + 启动后跑一条真实查询验证。

### Task P1d: tracing 采样与 spawn 错误处理
**Objective:** 24 处 `tokio::spawn` 中 19 处 fire-and-forget，静默失败难排查；tracing 无采样易刷屏。
**Files:** `crates/log-core/src/log_sampling.rs`、`services/*/src/**/*.rs`
**Step 1:** 对关键 `tokio::spawn` 用 `let handle = tokio::spawn(...);` 后 `tokio::spawn(async move { if let Err(e)=handle.await {...} })` 或在 supervisor 处 `tokio::task::JoinSet` 统一回收。
**Step 2:** 在 `log-core` 启用采样层（已有 `log_sampling.rs` 43 行，确认是否接线）。
**Step 3:** 验证 supervisor 能捕获子任务 panic。

---

## P2 合并冗余

### Task R1: 提炼统一服务启动骨架（消除每服务重复 main.rs + GrpcServiceBuilder）
**Objective:** 6+ 服务各自实现 `main.rs`/`impl GrpcServiceBuilder`，样板高度重复。
**Files:** 新文件 `crates/common/src/service_bootstrap.rs`（已存在，扩充），各 `services/*/src/main.rs`
**Step 1:** 在 `common` 提供泛型 `run_service!(ServiceStruct, migrations...)` 宏或 `ServiceRunner` 结构，封装 db/redis/tracing/graceful-shutdown/gRPC。
**Step 2:** 各服务 `main.rs` 收敛为 ~15 行调用该宏。
**Step 3:** `cargo check --workspace` 预期 0 error；逐服务冒烟启动验证。

### Task R2: 集中 gRPC client 配置
**Objective:** `grpc_clients.rs` 中 15+ 条 `ServiceDef` 手写 URL 默认值是样板。
**Files:** Modify `services/api-gateway/src/grpc_clients.rs`
**Step 1:** 用配置表（yaml/env）驱动 `ServiceDef` 列表，或将默认映射收敛到一个 `const SERVICE_REGISTRY: &[(&str,&str,&str)]`。
**Step 2:** `cargo check -p api-gateway` 验证。

### Task R3: 共享 crate 复用率提升 or 合并
**Objective:** `rate-limit-core`(0)、`log-core`(1)、`crypto-core`(1)、`circuit-breaker-core`(1) 复用率极低。
**Files:** `crates/*`
**Step 1:** 评估每个低复用 crate：若逻辑应被复用，在 `common` 内增加封装并让服务改用；若确属死代码，移入 `archive/` 或删除（见 C1）。
**Step 2:** 决策表写入 `deploy/docs/`。

---

## P3 删除无用/过时

### Task C1: 删除死 crate `rate-limit-core`（0 服务依赖）
**Objective:** 无任何服务使用，纯死代码。
**Files:** `crates/rate-limit-core/`、workspace `Cargo.toml` members
**Step 1:** `grep -rl "rate-limit-core" services crates` 确认 0 引用；从 `Cargo.toml` members 移除并 `rm -rf crates/rate-limit-core`。
**Step 2:** `cargo check --workspace` 预期无因删除产生的 error。
**Step 3:** 提交。

### Task C2: 清理 `allow(dead_code/unused)` 44 处
**Objective:** 44 处 `#[allow(dead_code)]`/`unused` 掩盖了未用代码，应删除代码或仅对确有必要的保留并加注释。
**Files:** `grep -rn "allow(dead_code)\|allow(unused)" services crates`
**Step 1:** 逐个判断：死字段/死函数直接删除；确需保留的（如对外 trait 占位）加 `// 保留：用于未来 X` 注释。
**Step 2:** `cargo check` 应无新增 warning。

### Task C3: 替换 `println!` 17 处为 tracing
**Objective:** `println!` 绕过统一日志/tracing，不利于采集。
**Files:** `grep -rn "println!" services crates`
**Step 1:** 改为 `tracing::{info,warn,error}!`；调试用途的加 `tracing::debug!`。
**Step 2:** `cargo check` 验证。

### Task C4: 统一依赖为 `workspace = true`
**Objective:** 12 个服务直接写死版本（如 `futures-util = "0.3"`）导致版本漂移。
**Files:** 12 个 `services/*/Cargo.toml`
**Step 1:** 将共享依赖改为 `dep = { workspace = true }`（在 workspace.dependencies 中已声明的）。
**Step 2:** `cargo update -p <dep>` 校验；`cargo check --workspace` 验证。

---

## 收尾验证

### Task V1: 全量编译 + 冒烟
**Step 1:** `cargo check --workspace` → 预期 0 error（基线当前 25，须先清零历史编译错误）。
**Step 2:** `cargo build --workspace --release` 计时对比优化前后（记录二进制体积 `du -sh target/release/<svc>`）。
**Step 3:** 启动 2–3 个核心服务（auth/user/api-gateway）做一轮真实请求，确认无回归。
**Step 4:** 汇总优化清单与收益写 `deploy/docs/backend-optimization-20260827.md`。

---

## 风险 / 权衡 / 待确认

1. **tokio 特性裁剪（P1a）** 可能触发个别服务用到被裁特性的代码；需逐个 `cargo check` 修复，耗时但收益明确（编译更快、镜像更小）。
2. **统一启动骨架（R1）** 改动面大，建议先做 1 个服务试点（api-key 已最规范）验证后再推广。
3. **运行时 SQL 改宏（P1c）** 需要 `DATABASE_URL` + `.sqlx` 离线缓存，CI 需配 `SQLX_OFFLINE=true`；若 DB 不可达，宏无法编译，需先确认联调环境。
4. **密钥外置（S1/S2）** 上线需同步在 Helm/K8s Secret 注入对应环境变量，否则服务启动失败（预期内、属安全加固必要代价）。
5. **死 crate 删除（C1）** 须先确认 CI/构建脚本无引用；`rate-limit-core` 已确认 0 引用可安全删。
6. 历史遗留的 **25 个编译错误**（tonic 0.14 迁移未完成）须先于本计划清零，否则 `cargo check --workspace` 无法作为验证基线。建议先以一个独立 plan 收口编译，再执行本优化。

## 建议执行顺序
S1→S2→S4（安全快赢）→ 先清零 25 编译错误（前置）→ P1a→P1b→P1c→P1d → R1(试点)→R2→R3 → C1→C2→C3→C4 → V1。
