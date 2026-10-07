FROM gcc:13 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libcurl4-openssl-dev \
    libcjson-dev \
    make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Makefile ./
COPY src/ ./src/
COPY tests/ ./tests/

RUN make test && make

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libcurl4 \
    libcjson1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/build/health_checker ./health_checker

ENTRYPOINT ["./health_checker"]
