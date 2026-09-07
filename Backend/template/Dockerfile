# =============================================================================
# MyAI Backend - 优化版多阶段构建 Dockerfile
# 构建: Rust (Cargo BuildKit cache mounts)
# 运行: Alpine Linux (静态编译，最小体积)
# 服务选择: 通过 SERVICE_NAME 环境变量指定启动哪个二进制
# =============================================================================

# ---- 构建阶段 ----
FROM rust:1.97.0-slim AS builder

# Rust mirrors (China acceleration)
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=120
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN mkdir -p /usr/local/cargo/config.d && printf '%s\n%s\n%s\n%s\n%s\n%s\n%s\n%s\n%s\n%s\n' \
    '[source.crates-io]' \
    'replace-with = "rsproxy"' \
    '' \
    '[source.rsproxy]' \
    'registry = "https://rsproxy.cn/crates.io-index"' \
    '' \
    '[registries.rsproxy]' \
    'index = "https://rsproxy.cn/crates.io-index"' \
    '' \
    '[source.rsproxy-sparse]' \
    'registry = "sparse+https://rsproxy.cn/index/"' \
    > /usr/local/cargo/config.d/mirror.toml

# Enable Docker BuildKit
# ARG BUILDKIT_INLINE_CACHE=1

WORKDIR /app

# 安装编译依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        protobuf-compiler \
        pkg-config \
        libssl-dev \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 复制 Cargo 配置文件（利用 Docker 层缓存）
COPY Cargo.toml Cargo.lock ./

# 创建 protos 目录结构
RUN mkdir -p protos crates services

# 复制源代码（先只复制依赖声明，利用缓存）
COPY crates/ ./crates/
COPY services/ ./services/
COPY protos/ ./protos/
COPY .sqlx/ ./.sqlx/

# 启用 sqlx 离线模式：macros 使用 .sqlx/ 缓存编译，无需连接数据库
ENV SQLX_OFFLINE=true

# 编译全部 workspace 成员（cargo cache mount 自动利用层缓存）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --workspace --offline 2>&1

# ---- 运行阶段 ----
FROM ubuntu:26.10

ARG SERVICE_NAME=api-gateway
ARG HTTP_PORT=8090
ARG BUILD_TAG=latest

LABEL maintainer="canplay <canplay@example.com>"
LABEL description="MyAI Backend Service"


RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        tzdata \
        curl && \
    rm -rf /var/lib/apt/lists/*

# 设置时区
ENV TZ=Asia/Shanghai
RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && echo "Asia/Shanghai" > /etc/timezone

# 创建非 root 用户

RUN groupadd -r app && \
    useradd -r -g app -u 1001 app

WORKDIR /app

# 从构建阶段复制选中的二进制文件
COPY --from=builder --chown=app:app \
    /app/target/release/${SERVICE_NAME} \
    /app/service

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${HTTP_PORT}/health || exit 1

EXPOSE ${HTTP_PORT}

USER app

# 日志通过 log-core 输出到 stdout（容器日志），无需 tail
# 直接前台运行，确保 SIGTERM 正确传递
CMD ["/app/service"]
