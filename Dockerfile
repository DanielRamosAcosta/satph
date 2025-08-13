# Multi-stage build for minimal production image
FROM rust:1.84-bookworm AS builder

# Install cross-compilation tools and musl
RUN apt-get update && apt-get install -y \
    musl-tools \
    musl-dev \
    gcc-x86-64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Add musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Set working directory
WORKDIR /app

# Configure cross-compilation environment for x86_64 musl
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-gnu-gcc

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN rm -rf src

# Copy source code
COPY src/ ./src/

# Build the application with static linking
RUN cargo build --target x86_64-unknown-linux-musl --release

# Production stage - minimal Alpine image for debugging
FROM alpine:3.20 AS production

# Install CA certificates for HTTPS connections
RUN apk --no-cache add ca-certificates

# Copy the statically linked binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/sftpgo-authelia-totp-hook /sftpgo-authelia-totp-hook

# Expose port 8080
EXPOSE 8080

# Run the binary
ENTRYPOINT ["/sftpgo-authelia-totp-hook"]
