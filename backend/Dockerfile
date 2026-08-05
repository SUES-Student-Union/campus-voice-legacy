FROM rust:1.97 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
# 国内构建加速：crates.io 走镜像（可 --build-arg CARGO_REGISTRY_MIRROR= 覆盖/留空禁用）
ARG CARGO_REGISTRY_MIRROR="sparse+https://rsproxy.cn/index/"
RUN if [ -n "$CARGO_REGISTRY_MIRROR" ]; then \
      mkdir -p /root/.cargo && \
      printf '[source.crates-io]\nreplace-with = "mirror"\n\n[source.mirror]\nregistry = "%s"\n\n[net]\ngit-fetch-with-cli = true\n' "$CARGO_REGISTRY_MIRROR" > /root/.cargo/config.toml; \
    fi
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/campus-voice-backend /usr/local/bin/campus-voice-backend
ENV UPLOAD_DIR=/data/uploads
EXPOSE 8080
CMD ["campus-voice-backend"]
