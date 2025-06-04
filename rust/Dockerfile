FROM rust:1.87.0

WORKDIR /app

# Build dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# then the whole thing
COPY src ./src
RUN cargo build --release

COPY static ./static

EXPOSE 3000

CMD ["/app/target/release/heads_or_tails"]
