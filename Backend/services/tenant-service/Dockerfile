# Multi-stage build for tenant-service
FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace
COPY . .

# Build the specific service
RUN cargo build --release -p tenant-service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tenant-service /usr/local/bin/tenant-service

EXPOSE 8080 9090

CMD ["/usr/local/bin/tenant-service"]
