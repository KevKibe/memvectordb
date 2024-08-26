FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /memvectordb

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /memvectordb/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin memvectordb

FROM debian:bookworm-slim as runtime
WORKDIR /memvectordb
COPY --from=builder /memvectordb/target/release/memvectordb /usr/local/bin/memvectordb

EXPOSE 8000

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
