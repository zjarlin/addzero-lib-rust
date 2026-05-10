FROM rust:1.88-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build -p az-drive-app --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/az-drive-app /usr/local/bin/az-drive-app
EXPOSE 8788
CMD ["az-drive-app", "serve", "--bind", "0.0.0.0:8788"]
