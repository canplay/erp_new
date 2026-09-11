# =============================================================================
# MyAI Admin Frontend - 纯 Nginx 镜像构建
# 构建阶段: 本地 pnpm build (Node + pnpm)
# 运行阶段: Nginx Alpine (静态 SPA 文件)
# 说明: 取消 node:24 构建阶段, 改为本地构建后只复制 dist 产物到 nginx 镜像
# =============================================================================

ARG BUILD_SERVICE=admin
ARG BUILD_TAG=latest

# ---- 运行阶段 ----
FROM nginx:alpine

# envsubst 模板支持: 渲染 nginx.conf 中的 API_GATEWAY_HOST/PORT
RUN apk add --no-cache gettext

RUN rm -rf /etc/nginx/conf.d/default.conf

# 复制 nginx 配置（K8s 使用 templates/ 目录进行 envsubst 渲染）
COPY nginx.conf /etc/nginx/templates/default.conf.template

ARG BUILD_SERVICE
ARG BUILD_TAG

# 反代上游默认值（envsubst 渲染 nginx.conf 使用；部署可覆盖）
ENV API_GATEWAY_HOST=api-gateway \
    API_GATEWAY_PORT=8090

RUN mkdir -p /usr/share/nginx/html

# 复制本地构建的 dist 产物
COPY dist/spa/ /usr/share/nginx/html/

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD ["nginx","-t"]

CMD ["nginx","-g","daemon off;"]
