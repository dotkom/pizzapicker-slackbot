FROM rust:1.76 AS builder

COPY Cargo.lock Cargo.toml ./
COPY src ./src
COPY config ./config

RUN cargo build --release

FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y ca-certificates libssl3

COPY --from=builder ./target/release/pizzapicker ./target/release/pizzapicker

EXPOSE 3000

CMD ["/target/release/pizzapicker"]
