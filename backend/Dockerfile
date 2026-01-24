# FROM rust:latest as builder

# WORKDIR /app

# # Install nightly toolchain
# RUN rustup toolchain install nightly && \
#     rustup default nightly

# # Copy dependency files
# COPY Cargo.toml ./
# COPY src ./src

# # Build release binary
# RUN cargo build --release

# # Runtime stage
# FROM debian:bookworm-slim

# # Install runtime dependencies
# RUN apt-get update && apt-get install -y \
#     ca-certificates \
#     libssl3 \
#     && rm -rf /var/lib/apt/lists/*

# WORKDIR /app

# # Copy binary from builder
# COPY --from=builder /app/target/release/esms-backend /app/esms-backend

# # Expose port
# EXPOSE 8080

# # Set environment variables
# ENV RUST_LOG=info
# ENV SERIAL_PORT=/dev/cu.usbmodem113401

# # Run the application
# CMD ["./esms-backend"]


FROM rust:latest as builder

WORKDIR /app

# Install nightly toolchain
RUN rustup toolchain install nightly && rustup default nightly

# Copy only Cargo.toml first to cache dependencies
COPY Cargo.toml ./

# Pre-fetch dependencies to use layer cache in CI/CD
RUN cargo fetch

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/esms-backend /app/esms-backend

EXPOSE 8080

ENV RUST_LOG=info
ENV SERIAL_PORT=/dev/cu.usbmodem113401

CMD ["./esms-backend"]
