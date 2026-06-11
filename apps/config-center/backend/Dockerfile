FROM rust:1.95-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build -p az-config-center-app --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/az-config-center-app /usr/local/bin/az-config-center-app
ENV CONFIG_CENTER_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["az-config-center-app"]
