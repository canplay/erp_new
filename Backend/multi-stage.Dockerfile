# =============================================================================
# MyAI Backend - Multi-stage Unified Build Dockerfile
# =============================================================================
# 阶段 1: 统一编译所有服务
# 阶段 2-N: 将编译产物复制到各自的服务镜像中
# =============================================================================

# =============================================================================
# Stage 1: Unified Build Stage
# =============================================================================
FROM rust:1.98.0-alpine AS builder

WORKDIR /myai

# Install build dependencies (Alpine packages)
RUN apk add --no-cache \
    ca-certificates \
    openssl \
    pkgconf \
    cmake \
    git \
    musl-dev \
    openssl-dev \
    pkgconfig \
    protoc \
    protobuf-dev \
    libuuid \
    libcap

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock /myai/

# Copy source code
COPY . /myai/

WORKDIR /myai

# Build all binaries in one stage
RUN cargo build --release && \
    cp target/release/* /myai/bin/ && \
    rm -rf target

# =============================================================================
# Stage 2+: Service-specific runtime images
# =============================================================================

# myai-api-gateway
FROM alpine:latest AS api-gateway
WORKDIR /myai
COPY --from=builder /myai/bin/api-gateway /myai/bin/api-gateway
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8090
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8090/health || exit 1
CMD ["./entrypoint.sh"]

# myai-auth-service
FROM alpine:latest AS auth-service
WORKDIR /myai
COPY --from=builder /myai/bin/auth-service /myai/bin/auth-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8081
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1
CMD ["./entrypoint.sh"]

# myai-user-service
FROM alpine:latest AS user-service
WORKDIR /myai
COPY --from=builder /myai/bin/user-service /myai/bin/user-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
CMD ["./entrypoint.sh"]

# myai-cms-service
FROM alpine:latest AS cms-service
WORKDIR /myai
COPY --from=builder /myai/bin/cms-service /myai/bin/cms-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8082
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8082/health || exit 1
CMD ["./entrypoint.sh"]

# myai-messaging-service
FROM alpine:latest AS messaging-service
WORKDIR /myai
COPY --from=builder /myai/bin/messaging-service /myai/bin/messaging-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8083
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8083/health || exit 1
CMD ["./entrypoint.sh"]

# myai-file-service
FROM alpine:latest AS file-service
WORKDIR /myai
COPY --from=builder /myai/bin/file-service /myai/bin/file-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8086
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8086/health || exit 1
CMD ["./entrypoint.sh"]

# myai-tenant-service
FROM alpine:latest AS tenant-service
WORKDIR /myai
COPY --from=builder /myai/bin/tenant-service /myai/bin/tenant-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8087
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8087/health || exit 1
CMD ["./entrypoint.sh"]

# myai-workflow-service
FROM alpine:latest AS workflow-service
WORKDIR /myai
COPY --from=builder /myai/bin/workflow-service /myai/bin/workflow-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8088
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8088/health || exit 1
CMD ["./entrypoint.sh"]

# myai-feedback-service
FROM alpine:latest AS feedback-service
WORKDIR /myai
COPY --from=builder /myai/bin/feedback-service /myai/bin/feedback-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8085
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8085/health || exit 1
CMD ["./entrypoint.sh"]

# myai-audit-service
FROM alpine:latest AS audit-service
WORKDIR /myai
COPY --from=builder /myai/bin/audit-service /myai/bin/audit-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8089
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8089/health || exit 1
CMD ["./entrypoint.sh"]

# myai-api-key-service
FROM alpine:latest AS api-key-service
WORKDIR /myai
COPY --from=builder /myai/bin/api-key-service /myai/bin/api-key-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 9091
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9091/health || exit 1
CMD ["./entrypoint.sh"]

# myai-hik-service
FROM alpine:latest AS hik-service
WORKDIR /myai
COPY --from=builder /myai/bin/hik-service /myai/bin/hik-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8092
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8092/health || exit 1
CMD ["./entrypoint.sh"]

# myai-pay-service
FROM alpine:latest AS pay-service
WORKDIR /myai
COPY --from=builder /myai/bin/pay-service /myai/bin/pay-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8093
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8093/health || exit 1
CMD ["./entrypoint.sh"]

# myai-clean-service
FROM alpine:latest AS clean-service
WORKDIR /myai
COPY --from=builder /myai/bin/clean-service /myai/bin/clean-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8095
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8095/health || exit 1
CMD ["./entrypoint.sh"]

# myai-tow-service
FROM alpine:latest AS tow-service
WORKDIR /myai
COPY --from=builder /myai/bin/tow-service /myai/bin/tow-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8094
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8094/health || exit 1
CMD ["./entrypoint.sh"]

# myai-ctp-service
FROM alpine:latest AS ctp-service
WORKDIR /myai
COPY --from=builder /myai/bin/ctp-service /myai/bin/ctp-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8096
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8096/health || exit 1
CMD ["./entrypoint.sh"]

# myai-xlt-service
FROM alpine:latest AS xlt-service
WORKDIR /myai
COPY --from=builder /myai/bin/xlt-service /myai/bin/xlt-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8097
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8097/health || exit 1
CMD ["./entrypoint.sh"]

# myai-ebike-service
FROM alpine:latest AS ebike-service
WORKDIR /myai
COPY --from=builder /myai/bin/ebike-service /myai/bin/ebike-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8098
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8098/health || exit 1
CMD ["./entrypoint.sh"]

# myai-lpr-service
FROM alpine:latest AS lpr-service
WORKDIR /myai
COPY --from=builder /myai/bin/lpr-service /myai/bin/lpr-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8099
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8099/health || exit 1
CMD ["./entrypoint.sh"]

# myai-social-ops-service
FROM alpine:latest AS social-ops-service
WORKDIR /myai
COPY --from=builder /myai/bin/social-ops-service /myai/bin/social-ops-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8110
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8110/health || exit 1
CMD ["./entrypoint.sh"]

# myai-browser-service
FROM alpine:latest AS browser-service
WORKDIR /myai
COPY --from=builder /myai/bin/browser-service /myai/bin/browser-service
COPY template/requirements.txt /myai/template/requirements.txt
COPY template/ /myai/template/
COPY env.example /myai/.env.example
COPY entrypoint.sh /myai/entrypoint.sh
ENV PYTHONPATH=/myai/template
EXPOSE 8120
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8120/health || exit 1
CMD ["./entrypoint.sh"]
