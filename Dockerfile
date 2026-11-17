# ---- build ----
FROM rust:1-slim-bookworm AS builder
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo build --release -p varex-wisp --bin varex-server

# ---- runtime: distroless, non-root ----
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /src/target/release/varex-server /usr/local/bin/varex-server
COPY --from=builder /src/packages/varex-demo/ /srv/varex/demo/
COPY --from=builder /src/varex.example.toml /etc/varex/varex.toml
ENV VAREX_STATIC_ROOT=/srv/varex/demo
EXPOSE 4000
ENTRYPOINT ["/usr/local/bin/varex-server"]
CMD ["--config", "/etc/varex/varex.toml"]
