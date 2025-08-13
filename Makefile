# Makefile for sftpgo-authelia-totp-hook

.PHONY: fmt clippy test test-verbose build build-release run run-dev check-all clean doc

# Format code
fmt:
	cargo fmt

# Run clippy with warnings as errors
clippy:
	cargo clippy -- -D warnings

# Run tests
test:
	cargo test -- --test-threads=1

# Run tests with output
test-verbose:
	cargo test -- --nocapture --test-threads=1

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
	AUTHELIA_BASE_URL=https://authelia.local \
	HTTP_BIND=0.0.0.0:8080 \
	HTTP_CLIENT_TIMEOUT_MS=5000 \
	LOG_LEVEL=debug \
	TLS_INSECURE=true \
	cargo run

# Check everything (format, clippy, test, build)
check-all: fmt clippy test build

# Clean build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --open
