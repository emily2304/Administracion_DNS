# ---- Etapa de build ----
FROM rust:1.82-slim AS builder

WORKDIR /app

# Copiamos primero los manifests para aprovechar la cache de capas de Docker
COPY Cargo.toml Cargo.lock* ./

# Truco para cachear dependencias: creamos un main.rs dummy y compilamos
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Ahora copiamos el código real y compilamos de verdad
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---- Etapa final (imagen liviana) ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/dns-interceptor /app/dns-interceptor

# El interceptor escucha DNS por UDP/53
EXPOSE 53/udp

# Variables de entorno configurables en runtime (ver .env.example)
ENV DNS_API_URL=http://dns-api:8080

ENTRYPOINT ["/app/dns-interceptor"]
