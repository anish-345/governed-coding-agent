# syntax=docker/dockerfile:1

# ─── Stage 1: Builder ───
FROM rust:slim AS builder
WORKDIR /app

# Install build dependencies (pkg-config, OpenSSL, ca-certificates)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy project manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code and static assets
COPY src ./src
COPY static ./static

# Build the release binary
RUN cargo build --release

# ─── Stage 2: Runtime ───
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies: ca-certificates for TLS/HTTPS (Nebius, GitHub, Tavily) and git
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary and static assets from builder
COPY --from=builder /app/target/release/governed-coding-agent /app/governed-coding-agent
COPY --from=builder /app/static /app/static

# Configuration & port
ENV HOST=0.0.0.0
ENV PORT=8080
ENV SANDBOX_BACKEND=host
ENV RUST_LOG=info

EXPOSE 8080

# Run the governed coding agent server
CMD ["/app/governed-coding-agent"]
