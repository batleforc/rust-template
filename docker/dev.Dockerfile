FROM rust:1-alpine

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY . .

CMD ["cargo", "run", "--release"]
