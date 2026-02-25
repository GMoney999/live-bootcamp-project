##### IMAGE: Alpine Linux – official rust 1.88
### New stage: "chef"
FROM rust:1.88-alpine AS chef
# Switch to the root user so you can install system packages and tools.
USER root
# Install:
#     - musl-dev C library headers
#     - cargo-chef
RUN apk add --no-cache musl-dev & cargo install cargo-chef
# cd /app
WORKDIR /app

### New stage: "planner"
FROM chef AS planner
# Copy full project source tree into "planner"
COPY . .
# Set up build dependencies
RUN cargo chef prepare --recipe-path recipe.json

### New stage: "builder"
FROM chef AS builder
# Build recipe.json (like Cargo.lock)
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (like 'cargo build --release'
RUN cargo chef cook --release --recipe-path recipe.json
# Copy full project source tree into "builder"
COPY . .
# Set env var to set SQLx to offline mode, using pre-generated query metadata instead of hitting a live db
ENV SQLX_OFFLINE true
# compile 'auth-service' binary
RUN cargo build --release --bin auth-service

##### IMAGE: Alpine Linux 3.18
### New stage: "runtime"
# NOTE: Muse alpine to match the musl-compiled binary's DNS resolver expectations
# NOTE: We do not need the Rust toolchain to run the binary
FROM alpine:3.18 AS runtime
# Install CA certificates for HTTPS requests
RUN apk add --no-cache ca-certificates
# cd /app
WORKDIR /app
# From "builder" stage — copy auth-service binary into the runtime image's executable path
COPY --from=builder /app/target/release/auth-service /usr/local/bin
# From "builder" stage — copy assets so static resources are available at runtime
COPY --from=builder /app/assets /app/assets
# When the container starts, run auth-service binary
ENTRYPOINT ["/usr/local/bin/auth-service"]
