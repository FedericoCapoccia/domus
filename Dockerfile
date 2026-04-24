FROM rust:slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN useradd --system --user-group --no-create-home --shell /usr/sbin/nologin domus
COPY --from=builder /app/target/release/domus ./domus
USER domus
CMD ["./domus"]
