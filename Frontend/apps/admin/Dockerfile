FROM nginx:1.31.4-alpine-otel

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy SPA files
COPY dist/spa/ /usr/share/nginx/html/

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
