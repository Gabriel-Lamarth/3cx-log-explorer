# syntax=docker/dockerfile:1
FROM rust:1-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
COPY src ./src
COPY web ./web
COPY rules ./rules
COPY docs ./docs
RUN cargo build --release --bin threecx-log-explorer-web

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 --home /nonexistent --shell /usr/sbin/nologin app
WORKDIR /app
COPY --from=builder /app/target/release/threecx-log-explorer-web /usr/local/bin/threecx-log-explorer-web
ENV BIND_ADDR=0.0.0.0:8080 DATA_DIR=/data SHARED_LOGS_DIR=/shared-logs
RUN mkdir -p /data /shared-logs && chown -R app:app /data /shared-logs
USER app
EXPOSE 8080
VOLUME ["/data", "/shared-logs"]
ENTRYPOINT ["/usr/local/bin/threecx-log-explorer-web"]
