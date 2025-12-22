# Build stage
FROM rust:1.81-slim-bullseye AS builder

WORKDIR /app
COPY . .

# Install build dependencies if needed (e.g., pkg-config, libssl-dev)
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev > /dev/null 2>&1 && rm -rf /var/lib/apt/lists/*

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 > /dev/null 2>&1 && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/mcp-payloadcms-rs /app/server

# Create a non-root user
RUN useradd -m -u 1000 mcpuser
USER mcpuser

# Set the entrypoint to the server binary
ENTRYPOINT ["/app/server"]

# Default arguments
CMD ["start", "--foreground"]
