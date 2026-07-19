FROM rust:bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY benches ./benches

RUN cargo build --release --bin asgard-rust

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/asgard-rust /usr/local/bin/asgard-rust

ENV APP_HOST=0.0.0.0 \
  APP_PORT=8080 \
  RUST_LOG=info

EXPOSE 8080
CMD ["asgard-rust"]
