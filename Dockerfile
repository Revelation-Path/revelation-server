# syntax=docker/dockerfile:1.9

# ─────────────────────────────────────────────────────────────────────────────
# Chef stage - install cargo-chef and sccache
# ─────────────────────────────────────────────────────────────────────────────
FROM rust:1.92-slim AS chef

ENV SCCACHE_VERSION=0.8.2
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV SCCACHE_DIR=/sccache

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz -C /usr/local/bin --strip-components=1 --wildcards '*/sccache'

ENV RUSTC_WRAPPER=/usr/local/bin/sccache
RUN cargo install cargo-chef --locked

WORKDIR /app

# ─────────────────────────────────────────────────────────────────────────────
# Planner stage
# ─────────────────────────────────────────────────────────────────────────────
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ─────────────────────────────────────────────────────────────────────────────
# Builder stage
# ─────────────────────────────────────────────────────────────────────────────
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/sccache,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json \
    && sccache --show-stats

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src

ENV SQLX_OFFLINE=true

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release \
    && sccache --show-stats \
    && mkdir -p /out \
    && cp target/release/revelation-server /out/

# ─────────────────────────────────────────────────────────────────────────────
# Runtime stage
# ─────────────────────────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /out/revelation-server /usr/local/bin/
COPY migrations ./migrations
EXPOSE 3000
CMD ["revelation-server"]
