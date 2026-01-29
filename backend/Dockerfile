FROM rust:latest as builder

WORKDIR /app

# Install nightly toolchain
RUN rustup toolchain install nightly && rustup default nightly

# ✅ Install system dependencies needed for serialport / libudev
RUN apt-get update && apt-get install -y \
    pkg-config \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy everything at once (Cargo.toml + src)
COPY Cargo.toml ./ 
COPY src ./src



# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/esms-backend /app/esms-backend

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV SERIAL_PORT=/dev/cu.usbmodem113401

# Run the application
CMD ["./esms-backend"]

