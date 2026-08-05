FROM rust:1.97 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
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
