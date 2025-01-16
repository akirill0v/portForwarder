# syntax=docker/dockerfile:1.4
# Cargo build stage
FROM rust:1.80-slim AS cargo-builder
# Install Rust dependencies
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/usr/local/cargo/registry \
    apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl wget git libssl-dev
COPY . /app
WORKDIR /app
# Build cargo packages and store cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo fetch && \
    cargo build --release && \
    mkdir -p /cargo-cache && \
    cp -R /usr/local/cargo/registry /cargo-cache/
# Verify the content of /cargo-cache && clean unnecessary files
RUN ls -la /cargo-cache/ && rm -rfd /cargo-cache/registry/src

# Cargo build stage
FROM rust:1.80-slim AS builder
# Install Rust dependencies
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/usr/local/cargo/registry \
    apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl wget git libssl-dev
COPY . /app
WORKDIR /app

# Copy Rust build artifacts
# COPY --from=cargo-builder /cargo-cache/git /usr/local/cargo/git
COPY --from=cargo-builder /cargo-cache/registry /usr/local/cargo/registry

RUN cargo build --release
RUN ldd /app/target/release/portfd

# Result image
FROM debian:bookworm-slim
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/usr/local/cargo/registry \
    apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates

COPY --from=builder /app/target/release/portfd /usr/local/bin/portfd
EXPOSE 6443
ENTRYPOINT ["/usr/local/bin/portfd"]
