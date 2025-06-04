FROM rust:1.87.0-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/heads_or_tails .

COPY static ./static

EXPOSE 3000

CMD ["/app/heads_or_tails"]
