# Rust as the base image
FROM rust:bookworm as build

# Create a new empty shell project
RUN USER=root cargo new --bin femto
WORKDIR /build

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy the source code
COPY ./src ./src
COPY ./migrations ./migrations
# Build for release.
RUN cargo build --release --bin femto

# The final base image
FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

COPY .env /app

# Copy from the previous build
COPY --from=build /build/target/release/femto /app
# COPY --from=build /holodeck/target/release/holodeck/target/x86_64-unknown-linux-musl/release/holodeck .

# Run the binary
CMD ["/app/femto"]