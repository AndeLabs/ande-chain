# =============================================================================
# ANDE CHAIN - PRODUCTION OPTIMIZED DOCKERFILE
# Multi-stage build for ande-reth with enhanced security and performance
# Base: Reth v1.8.2 + Custom EVM (ANDE Precompile + Parallel Execution + MEV)
# Target: ~100MB final image, optimized for production deployment
# Compatible with Docker Legacy Builder and BuildKit
# =============================================================================

ARG DEBIAN_VERSION=bookworm

# =============================================================================
# BUILDER STAGE: Maximum Performance Optimized Build
# =============================================================================
# Note: Using stable base + rustup to install nightly for edition2024 support
# This is more reliable than depending on rust:nightly Docker tags
FROM rust:1.83-slim-${DEBIAN_VERSION} AS builder

# Build arguments
ARG BUILD_PROFILE=maxperf
ARG FEATURES="jemalloc asm-keccak"

# Set working directory
WORKDIR /build

# Metadata labels
LABEL stage=builder
LABEL org.opencontainers.image.source="https://github.com/ande-labs/ande-chain"

# Install build dependencies in a single layer
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        clang \
        libclang-dev \
        llvm-dev \
        git \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install Rust nightly toolchain for edition2024 support required by Reth v1.8.2
RUN rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rust-src && \
    cargo --version && \
    rustc --version

# Configure Rust for MAXIMUM performance (Reth best practices)
# - target-cpu=native: Optimize for build machine CPU (use x86-64-v3 for portability)
# - link-arg=-fuse-ld=lld: Use LLVM's fast linker
# Note: embed-bitcode removed to allow LTO (required by maxperf profile)
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld"
ENV CARGO_BUILD_JOBS=8
ENV CARGO_INCREMENTAL=0
ENV CARGO_NET_RETRY=10
ENV RUST_BACKTRACE=0
ENV CARGO_PROFILE_MAXPERF_BUILD_OVERRIDE_DEBUG=false

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY bindings ./bindings
COPY tools ./tools
COPY tests ./tests

# Build ANDE Reth - Production node with native precompiles
#
# ✅ PRODUCCIÓN (2025-11-15):
# ande-reth es un nodo Reth completo con precompiles nativos.
# Integra AndePrecompileProvider en 0xFD directamente en el EVM.
#
# Ver: docs/PRECOMPILE_INTEGRATION_FINDINGS.md
RUN cargo build \
        --profile ${BUILD_PROFILE} \
        --bin ande-reth \
        --features "${FEATURES}" \
    && cp target/${BUILD_PROFILE}/ande-reth /ande-node

# Strip binary and verify
RUN strip --strip-all /ande-node && \
    chmod +x /ande-node && \
    /ande-node --version

# =============================================================================
# RUNTIME STAGE: Minimal distroless image for security
# =============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

# Metadata for production image
LABEL org.opencontainers.image.title="ANDE-CHAIN"
LABEL org.opencontainers.image.description="ANDE Chain Node - Sovereign Rollup with Token Duality, Parallel EVM & MEV Protection"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.vendor="ANDE Labs"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.url="https://github.com/ande-labs/ande-chain"
LABEL org.opencontainers.image.documentation="https://docs.ande.network"

# Copy binary with proper ownership
COPY --from=builder --chown=nonroot:nonroot /ande-node /usr/local/bin/ande-node

# Create data directory
USER nonroot:nonroot
WORKDIR /data

# Network and API ports
EXPOSE 30303 30303/udp 8545 8546 8551 9001 9091 9092

# Volume for persistent data
VOLUME ["/data"]

# Production health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
    CMD ["/usr/local/bin/ande-node", "--version"] || exit 1

# Default configuration
ENTRYPOINT ["/usr/local/bin/ande-node"]
CMD ["node", \
     "--datadir", "/data", \
     "--http", \
     "--http.addr", "0.0.0.0", \
     "--http.port", "8545", \
     "--http.api", "admin,eth,net,web3,txpool,debug,trace", \
     "--ws", \
     "--ws.addr", "0.0.0.0", \
     "--ws.port", "8546", \
     "--authrpc.addr", "0.0.0.0", \
     "--authrpc.port", "8551", \
     "--metrics", "0.0.0.0:9001", \
     "--port", "30303"]

# Build info
ARG BUILD_DATE
ARG VCS_REF
LABEL org.opencontainers.image.created=$BUILD_DATE
LABEL org.opencontainers.image.revision=$VCS_REF
