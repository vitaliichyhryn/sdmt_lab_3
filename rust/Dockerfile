FROM rust:1.87.0

WORKDIR /app

COPY . .

RUN cargo build --release

EXPOSE 3000

CMD ["/app/target/release/heads_or_tails"]
