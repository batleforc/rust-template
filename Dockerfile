FROM rust:1-alpine3.18 as builder

RUN apk add --no-cache musl-dev openssl openssl-dev

WORKDIR /app

COPY . .
ENV RUST_BACKTRACE 1
ENV OPENSSL_STATIC=true

RUN cargo build --release

FROM alpine:3.18

RUN apk add --no-cache openssl

WORKDIR /app

ENV RUST_BACKTRACE 1

COPY --from=builder /app/target/release/api /app/api
CMD ["./api"]