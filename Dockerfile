ARG RUST_VERSION=1.89.0
FROM rust:${RUST_VERSION}-slim-bullseye AS builder

WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -ex
cargo build --release
cp ./target/release/blackbox-http-discovery /bin/server
EOF

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /bin/server /bin/
COPY config.yaml .

USER 1000

ENTRYPOINT ["/bin/server"]