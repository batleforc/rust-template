FROM rust:1-alpine as builder

RUN apk add --no-cache musl-dev openssl

WORKDIR /app

COPY . .

RUN cargo build --release

FROM alpine:3.17

WORKDIR /app

COPY --from=builder /app/target/release/app /app/app
CMD ["app"]