# Stage 1: Build web dashboard
FROM node:22-slim AS dashboard
WORKDIR /build/web-dashboard
COPY web-dashboard/package*.json ./
RUN npm ci
COPY web-dashboard/ ./
RUN npm run build

# Stage 2: Build Rust binaries
FROM rust:1.85-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY . .
RUN cargo build --release -p openusbd

# Stage 3: Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    usbip hwdata ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create directories
RUN mkdir -p /etc/openusb /var/lib/openusb /var/log/openusb /usr/share/openusb/web

# Copy binaries
COPY --from=builder /build/target/release/openusbd /usr/local/bin/openusbd

# Copy web dashboard
COPY --from=dashboard /build/web-dashboard/dist/ /usr/share/openusb/web/

# Copy default config
COPY server/config/openusb.toml.example /etc/openusb/openusb.toml

# USB/IP port + API port
EXPOSE 3240 8443

# Data volume for database
VOLUME ["/var/lib/openusb"]

ENTRYPOINT ["openusbd"]
CMD ["--config", "/etc/openusb/openusb.toml"]
