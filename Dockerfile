FROM rust:1.76 as builder

WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install --path .


FROM debian:bullseye-slim as runner

RUN apt update; apt install -y libssl1.1

COPY --from=builder /usr/local/cargo/bin/blackbox-http-discovery .
COPY config.yaml .

USER 1000

ENTRYPOINT ["./blackbox-http-discovery"]