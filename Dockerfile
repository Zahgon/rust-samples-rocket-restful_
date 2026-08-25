# ------------------------------------------------------------------------------
# Stage 1: Builder
# This stage builds the application and its dependencies.
# ------------------------------------------------------------------------------
FROM --platform=linux/amd64 rust:1.88-slim-bookworm AS builder

# -- Install system packages ---------------------------------------------------
# gcc / pkg-config: required by libsqlite3-sys (bundled feature compiles from
#   source via the cc crate)
# musl-tools: provides musl-gcc, the C linker required for the
#   x86_64-unknown-linux-musl target
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    pkg-config \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# -- Pre-build dependencies (cached) ------------------------------------------
# Copy only the manifests and a stub main.rs so Cargo can compile all
# dependencies in isolation. This layer is cached and only invalidated when
# Cargo.toml or Cargo.lock change — not when application source changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    CC_x86_64_unknown_linux_musl=musl-gcc \
    cargo build --release --target x86_64-unknown-linux-musl

# -- Build application ---------------------------------------------------------
# Overlay with the real sources. The stub main.rs is overwritten by the COPY,
# but Cargo uses mtime to detect changes — touching main.rs after the COPY
# ensures Cargo recompiles the application crate without re-compiling
# dependencies (which remain in the cache-mounted target/).
COPY src/ ./src/
COPY migrations/ ./migrations/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    touch src/main.rs && \
    CC_x86_64_unknown_linux_musl=musl-gcc \
    cargo build --release --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/rust-samples-rocket-restful /app/rust-samples-rocket-restful

# ------------------------------------------------------------------------------
# Stage 2: Runtime
# This stage creates the final, minimal image to run the application.
# ------------------------------------------------------------------------------
FROM --platform=linux/amd64 alpine:3.23 AS runtime

WORKDIR /app

# -- Install system packages ---------------------------------------------------
RUN apk add --no-cache curl

# -- Metadata ------------------------------------------------------------------
LABEL org.opencontainers.image.title="🧪 RESTful API with Rust and Axum"
LABEL org.opencontainers.image.description="Proof of Concept for a RESTful API made with Rust and Axum"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/nanotaboada/rust-samples-rocket-restful"
LABEL org.sonarsource.docker.dockerfile="/Dockerfile"

# -- Copy artifacts ------------------------------------------------------------
COPY --from=builder /app/rust-samples-rocket-restful .
COPY --chmod=444    README.md                        ./
COPY --chmod=555    scripts/entrypoint.sh            ./entrypoint.sh
COPY --chmod=555    scripts/healthcheck.sh           ./healthcheck.sh

# -- Configure runtime ---------------------------------------------------------
# https://rules.sonarsource.com/docker/RSPEC-6504/
RUN addgroup -S app && \
    adduser -S -G app app && \
    mkdir -p /storage && \
    chown -R app:app /storage

ENV STORAGE_PATH=/storage/players-sqlite3.db

USER app

EXPOSE 9000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["./healthcheck.sh"]

ENTRYPOINT ["./entrypoint.sh"]
CMD ["./rust-samples-rocket-restful"]
