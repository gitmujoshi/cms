# Builder stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/contract-management-system ./

# Copy migrations
COPY --from=builder /usr/src/app/migrations ./migrations

# Create config directory
RUN mkdir -p /etc/contract-management-system

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0

# Run the binary
CMD ["contract-management-system"] 