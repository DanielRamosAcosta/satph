# syntax=docker/dockerfile:1.7

########## STAGE 1: build ##########
FROM --platform=$BUILDPLATFORM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Build application
COPY src ./src
RUN cargo build --release

########## STAGE 2: runtime ##########
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sftpgo-authelia-totp-hook /usr/local/bin/sftpgo-authelia-totp-hook

USER 10001:10001

ENTRYPOINT ["sftpgo-authelia-totp-hook"]
