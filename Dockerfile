FROM rust:1.82-slim-bullseye AS builder

# Set environment variables for Rust
ENV RUST_BACKTRACE=1

WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    g++ libx11-dev libxi-dev libxrandr-dev libxcursor-dev libvulkan-dev libudev-dev pkg-config libasound2-dev libxkbcommon-x11-0 libxkbcommon-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy application files into the container
COPY . .
RUN cargo build --release

FROM ubuntu:20.04 AS final

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libx11-6 libxi6 libxrandr2 libxcursor1 libvulkan1 libudev1 libasound2 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the built application from the builder stage
COPY --from=builder /app/target/release/manytris /usr/local/bin/manytris

# Expose the port that the application listens on.
EXPOSE 9989

# Set environment variables for Rust
ENV RUST_BACKTRACE=full

# What the container should run when it is started.
CMD ["manytris", "server", "--headless", "--host=0.0.0.0", "--port=9989"]
