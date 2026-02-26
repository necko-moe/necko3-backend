FROM lukemathwalker/cargo-chef:latest-rust-alpine3.23 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher
WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig openssl-dev git curl

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig openssl-dev git curl

COPY . .
COPY --from=cacher /app/target target

RUN cargo build --release --locked

FROM alpine:3.23

RUN apk add --no-cache ca-certificates libssl3 libgcc curl

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

WORKDIR /app

COPY --from=builder /app/target/release/necko3 /app/necko3

CMD ["/app/necko3"]