# Build stage
FROM rust:1.94-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    coreutils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/ZeldaServer .
COPY src/content/ src/content/
COPY .env.local .env.local

RUN mkdir -p logs
VOLUME /app/logs

EXPOSE ${PORT}

ENTRYPOINT ["sh", "-c", "stdbuf -oL ./ZeldaServer --port ${PORT} ${VERBOSITY} 2>&1 | tee -a ./logs/${PORT}_serverlog_$(date +%m_%d_%y_%s).log"]
