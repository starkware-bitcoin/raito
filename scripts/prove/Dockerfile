FROM rust:1.78-bookworm AS builder
WORKDIR /app

# Copy the entire workspace to ensure Cargo can resolve the workspace members
COPY . .

# Install bootloader hints before building with SSH authentication
# This requires building with: docker build --ssh default ...
RUN --mount=type=ssh \
    mkdir -p -m 0700 ~/.ssh && \
    ssh-keyscan github.com >> ~/.ssh/known_hosts && \
    make install-bootloader-hints

# Build only the raito-assumevalid binary in release mode
RUN cargo build --release -p raito-assumevalid

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/raito-assumevalid /bin/raito-assumevalid

# Copy required runtime assets (Cairo executable, bootloader, prover params)
# into stable locations inside the image so the binary can reference them.
COPY --from=builder /app/target/proving/assumevalid-syscalls.executable.json /opt/raito/assumevalid-syscalls.executable.json
COPY --from=builder /app/bootloaders/simple_bootloader_compiled.json /opt/raito/bootloader.json
COPY --from=builder /app/packages/assumevalid/prover_params.json /opt/raito/prover_params.json

# Default entrypoint; pass args via container args/command
ENTRYPOINT ["/bin/raito-assumevalid"]

