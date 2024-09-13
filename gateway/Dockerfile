FROM rust:latest AS build
# Create a new empty shell project
RUN USER=root cargo new --bin femto-gateway
WORKDIR /build
COPY ./.env ./.env
# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy the source code
COPY ./.sqlx ./.sqlx
COPY ./src ./src
COPY ./migrations ./migrations

# Build for release.
RUN cargo build --release --bin femto-gateway

FROM debian:bookworm-slim

# install openssl
RUN apt-get update && apt install -y openssl
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Copy from the previous build
COPY --from=build /build/target/release/femto-gateway /app

# Run the binary
CMD ["/app/femto-gateway"]