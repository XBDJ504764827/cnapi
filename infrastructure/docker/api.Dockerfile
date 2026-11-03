# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
FROM rust:1.97-slim-bookworm AS builder

WORKDIR /src

# rustls is used for TLS, no OpenSSL headers are required, but a C toolchain is
# needed by a few dependencies (ring, libz).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Manifests first so dependency compilation can be cached between source changes.
COPY Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml ./apps/api/Cargo.toml

# The workspace member needs a target to exist before dependencies are fetched.
# Building the stub warms the dependency cache; the real build below only has to
# recompile the crate itself.
RUN mkdir -p apps/api/src \
    && printf 'fn main() {}\n' > apps/api/src/main.rs \
    && printf '' > apps/api/src/lib.rs \
    && cargo fetch --locked \
    && cargo build --release --locked --bin cngokz-api

COPY apps/api ./apps/api

# `sqlx::migrate!` embeds ./migrations at compile time, so sources must be
# copied before the release build.
RUN cargo build --release --locked --bin cngokz-api

# ---------------------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 cngokz \
    && useradd --system --uid 10001 --gid cngokz --home /nonexistent --shell /usr/sbin/nologin cngokz

WORKDIR /app

COPY --from=builder /src/target/release/cngokz-api /usr/local/bin/cngokz-api

ENV APP_ENV=production \
    APP_HOST=0.0.0.0 \
    APP_PORT=8080 \
    RUST_LOG=info

USER 10001:10001
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/cngokz-api"]
