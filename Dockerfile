#FROM rust:1-alpine3.18 as builder
#
#RUN apk add --no-cache musl-dev openssl openssl-dev
#
#WORKDIR /app
#
#COPY . .
#ENV RUST_BACKTRACE 1
#ENV OPENSSL_STATIC=true
#
#RUN cargo build --release
#
#FROM alpine:3.18
#
#RUN apk add --no-cache openssl
#
#WORKDIR /app
#
#ENV RUST_BACKTRACE 1
#
#COPY --from=builder /app/target/release/api /app/api
#CMD ["./api"]

FROM rust:1.71 as builder

WORKDIR /usr/src/api
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/api /usr/local/bin/api
CMD ["api"]