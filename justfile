# Justfile for sftpgo-authelia-totp-hook

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy with warnings as errors
clippy:
    cargo clippy -- -D warnings

# Run clippy on all targets and features
clippy-all:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run security audit
audit:
    cargo audit

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the application (requires AUTHELIA_BASE_URL env var)
run:
    cargo run

# Run with sample environment
run-dev:
    #!/usr/bin/env bash
    export AUTHELIA_BASE_URL=https://authelia.local
    export HTTP_BIND=0.0.0.0:8080
    export HTTP_CLIENT_TIMEOUT_MS=5000
    export LOG_LEVEL=debug
    export TLS_INSECURE=true
    cargo run

# Run CI checks locally (what GitHub Actions runs)
ci: fmt-check clippy-all test audit

# Check everything (format, clippy, test, build)
check-all: fmt clippy test build

# Clean build artifacts
clean:
    cargo clean

# Run in watch mode for development
watch:
    cargo watch -x "run"

# Generate documentation
doc:
    cargo doc --open

# Build Docker image with current git hash
docker-build:
    #!/usr/bin/env bash
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d '"' -f 2)
    GIT_SHA=$(git rev-parse --short HEAD)
    docker build -t sftpgo-authelia-totp-hook:${VERSION}-${GIT_SHA} .
    docker tag sftpgo-authelia-totp-hook:${VERSION}-${GIT_SHA} sftpgo-authelia-totp-hook:latest
    echo "Built: sftpgo-authelia-totp-hook:${VERSION}-${GIT_SHA}"

# Run Docker container
docker-run: docker-build
    #!/usr/bin/env bash
    docker run -p 8080:8080 \
        -e AUTHELIA_BASE_URL=https://authelia.local \
        -e HTTP_BIND=0.0.0.0:8080 \
        -e LOG_LEVEL=debug \
        sftpgo-authelia-totp-hook:latest
