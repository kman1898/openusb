FROM rust:1.85-slim AS builder

WORKDIR /build
COPY . .
RUN cargo build --release -p openusbd

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    linux-tools-generic usbip ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/openusbd /usr/local/bin/openusbd
COPY server/config/openusb.toml.example /etc/openusb/openusb.toml

EXPOSE 3240 8443

ENTRYPOINT ["openusbd"]
CMD ["--config", "/etc/openusb/openusb.toml"]
