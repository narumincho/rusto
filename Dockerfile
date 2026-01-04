# Builder stage
FROM rust as builder

# Install wasm-pack
RUN curl -L https://github.com/rustwasm/wasm-pack/releases/download/v0.13.1/wasm-pack-v0.13.1-x86_64-unknown-linux-musl.tar.gz | tar -xz \
    && mv wasm-pack-v0.13.1-x86_64-unknown-linux-musl/wasm-pack /usr/local/bin/wasm-pack \
    && rm -rf wasm-pack-v0.13.1-x86_64-unknown-linux-musl

WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Runner stage
FROM debian:bookworm-slim

# Install OpenSSL (often needed for rust binaries) and ca-certificates
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy the binary
COPY --from=builder /usr/src/app/target/release/rusto .

# Copy the frontend assets (required by main.rs)
# We need to preserve the directory structure expected by std::fs::read("frontend/pkg/...")
COPY --from=builder /usr/src/app/frontend/pkg ./frontend/pkg

# Service must listen to $PORT environment variable.
ENV PORT 8080

# Run the web service
CMD ["./rusto"]
