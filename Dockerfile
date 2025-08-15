# syntax=docker/dockerfile:1.7

########## STAGE 1: build ##########
FROM --platform=$BUILDPLATFORM rust:1-bookworm AS builder
ARG TARGETPLATFORM
ARG TARGETARCH

WORKDIR /app

# Elegir el target musl según la arquitectura
RUN case "$TARGETARCH" in \
      "amd64")  echo x86_64-unknown-linux-musl  > /rust_target ;; \
      "arm64")  echo aarch64-unknown-linux-musl > /rust_target ;; \
      *) echo "Arquitectura no soportada: $TARGETARCH" && exit 1 ;; \
    esac \
 && rustup target add $(cat /rust_target)

# Herramientas necesarias para compilar estático con musl
RUN apt-get update \
 && apt-get install -y --no-install-recommends musl-tools pkg-config ca-certificates \
 && update-ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# --- Cacheo de dependencias de Cargo ---
# Si usas workspace, copia también el Cargo.toml del workspace y los Cargo.toml de crates relevantes.
COPY Cargo.toml Cargo.lock ./
# Copia el src después para aprovechar capas de caché de dependencias
COPY src ./src

# Compilar en release para el target elegido (with separate cache per target)
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-$TARGETARCH \
    --mount=type=cache,target=/app/target,id=target-$TARGETARCH \
    RUST_TARGET=$(cat /rust_target) \
 && cargo build --release --target $RUST_TARGET \
 && install -Dm755 target/$RUST_TARGET/release/sftpgo-authelia-totp-hook /out/sftpgo-authelia-totp-hook

########## STAGE 2: runtime ##########
FROM scratch AS runtime

# (Opcional pero útil) Certificados raíz para TLS/HTTPS
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Binario estático
COPY --from=builder /out/sftpgo-authelia-totp-hook /sftpgo-authelia-totp-hook

# Usuario no root (ID arbitrario). En scratch no hay /etc/passwd, pero esto funciona.
USER 10001:10001

ENTRYPOINT ["/sftpgo-authelia-totp-hook"]
