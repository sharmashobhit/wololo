# Use the official Rust image as build environment
FROM rust:1.83-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the recipe and build dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build
COPY . .
RUN cargo build --release

# Use Ubuntu as runtime environment
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    iputils-ping \
    net-tools \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /app wololo

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/wololo ./

# Create default config
RUN echo 'server:\n  ip: "0.0.0.0"\n  port: 3000\n  external_url: "http://localhost:3000"\n\nsync:\n  enabled: false\n  interval_seconds: 30\n\ndevices: []' > ./config.yaml

# Change ownership to non-root user
RUN chown -R wololo:wololo /app

# Switch to non-root user
USER wololo

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/hello || exit 1

# Run the application
CMD ["./wololo"]