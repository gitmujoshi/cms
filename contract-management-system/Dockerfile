# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/contract-management-system .

ENV HOST=0.0.0.0
EXPOSE 8080

CMD ["contract-management-system"] 