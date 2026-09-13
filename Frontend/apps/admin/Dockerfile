# 前端在 CI/本地先编译（pnpm --filter admin build → apps/admin/dist/spa），
# 再基于 nginx:1.31.4-alpine-otel 构建纯静态镜像（不再在容器内编译）。
FROM nginx:1.31.4-alpine-otel
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY apps/admin/dist/spa /usr/share/nginx/html
EXPOSE 80
