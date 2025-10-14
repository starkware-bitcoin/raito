FROM rust:1.78-bookworm AS builder
WORKDIR /app

# Copy the entire workspace to ensure Cargo can resolve the workspace members
COPY . .

# Build only the raito-assumevalid binary in release mode
RUN cargo build --release -p raito-assumevalid

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/raito-assumevalid /bin/raito-assumevalid

# Default entrypoint; pass args via container args/command
ENTRYPOINT ["/bin/raito-assumevalid"]

