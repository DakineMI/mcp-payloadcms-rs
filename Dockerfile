# Build stage
FROM rust:1.81-slim-bullseye AS builder

WORKDIR /app
COPY . .

# Install build dependencies if needed (e.g., pkg-config, libssl-dev)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/mcp-payloadcms-rs /app/server

# Create a non-root user
RUN useradd -m -u 1000 mcpuser
USER mcpuser

# Set the default command to run the server in foreground with stdio enabled
# We explicitly disable other transports to ensure a clean stdio stream
CMD ["/app/server", "start", "--foreground"]
