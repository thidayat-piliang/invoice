# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy and copy actual source
RUN rm -rf src
COPY src ./src

# Build actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/flashbill-api ./

# Copy migrations
COPY infrastructure/database/migrations ./infrastructure/database/migrations/

# Create non-root user
RUN useradd -m -u 1001 flashbill && \
    chown -R flashbill:flashbill /app
USER flashbill

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

EXPOSE 3000

CMD ["./flashbill-api"]
