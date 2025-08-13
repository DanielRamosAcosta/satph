# Justfile for sftpgo-authelia-totp-hook

# Format code
fmt:
    cargo fmt

# Run clippy with warnings as errors
clippy:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

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
