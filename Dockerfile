FROM messense/rust-musl-cross as builder

WORKDIR /usr/src

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install --path .


FROM alpine as runner

RUN apk --no-cache add ca-certificates

COPY --from=builder /usr/local/cargo/bin/blackbox-http-discovery .
COPY config.yaml .

USER 1000

ENTRYPOINT ["./blackbox-http-discovery"]