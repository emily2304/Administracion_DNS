# DNS Interceptor

Componente **DNS Interceptor** del Proyecto 1 de IC-7602 (Redes). Aplicación en Rust que escucha en **UDP/53**, clasifica cada paquete DNS en Caso 1 o Caso 2, y se comunica con el **DNS API** vía HTTP(S).

No necesitás tener Rust instalado: todo se compila y corre dentro de Docker.

## Estructura del proyecto

```
dns-interceptor/
├── Cargo.toml / Cargo.lock  # Dependencias del proyecto Rust (versiones fijas)
├── Dockerfile               # Build multi-stage: compila y arma la imagen final
├── docker-compose.yml       # Levanta el interceptor (y el DNS API cuando esté listo)
├── .env.example             # Variables de entorno de ejemplo
├── src/
│   ├── lib.rs                # Expone los módulos (usado también por los tests)
│   ├── main.rs                # Punto de entrada: socket UDP + loop principal
│   ├── config.rs              # Carga de configuración desde variables de entorno
│   ├── api_client.rs          # Cliente HTTP hacia el DNS API (BASE64 + JSON)
│   ├── dns/
│   │   ├── header.rs           # Parseo/serialización del header DNS
│   │   ├── question.rs         # Parseo de la sección Question
│   │   ├── record.rs           # Construcción de registros de respuesta (A)
│   │   └── message.rs          # DnsMessage + clasificación Caso 1 / Caso 2
│   └── resolver/
│       └── standard_query.rs  # Lógica de resolución (single/multi/weight/geo/round-trip)
└── tests/
    └── dns_parsing_tests.rs  # Tests unitarios del parseo DNS
```

## Cómo correrlo (todo vía Docker)

```bash

# 1. Construir y levantar el contenedor
docker compose up --build
```

Eso ya deja el interceptor escuchando en `localhost:5353/udp` (mapeado desde el 53 del contenedor). Se puede probar con:

```bash
nslookup -port=5353 example.com localhost
```

Se va a ver en los logs del contenedor (`RUST_LOG=info`) cómo cada paquete se clasifica como **Caso 1** o **Caso 2** antes de reenviarse.

Para reconstruir después de cambiar código:

```bash
docker compose up --build
```

Para correr los tests dentro de un contenedor (sin instalar Rust en la máquina):

```bash
docker run --rm -v "$PWD":/app -w /app rust:1.82-slim cargo test
```

## Identificación de Caso 1 / Caso 2

Según el enunciado, cada paquete recibido se clasifica así (`dns::RequestCase` en `dns/message.rs`):

- **Caso 1 — no es query estándar** (`QR ≠ 0` o `OPCODE ≠ 0`): se codifica el paquete completo en BASE64 y se reenvía tal cual al DNS API vía `POST /api/dns_resolver`. La respuesta (BASE64) se decodifica y se manda de vuelta al cliente.
- **Caso 2 — es query estándar** (`QR = 0` y `OPCODE = 0`): se extrae el hostname (`QNAME`) y se consulta al DNS API si existe un registro (`GET /api/exists`). Según el tipo de registro que devuelva (`single`, `multi`, `weight`, `geo`, `round-trip`) se arma la respuesta localmente. Si no existe el registro, o la IP asociada está `unhealthy`, se cae al mismo camino del Caso 1.

Cada paquete que llega queda logueado con su clasificación, ID, `QR`, `OPCODE` y dominio (si aplica) — fácil de verificar con `docker compose logs -f`.

## Selección GEO y Round Trip

- GEO envía la IP de origen como `client_ip` a `GET /api/exists`. El DNS API la compara con los rangos de `ip_to_country` y devuelve el país; si no hay coincidencia, el interceptor elige aleatoriamente entre los targets saludables.
- Round Trip recibe del DNS API la ubicación estimada del cliente, las ubicaciones configuradas en `health_checker_locations` y las mediciones de `health_checker_measurements` del registro consultado. Selecciona el checker más cercano y luego su target saludable con menor RTT.
- La tabla IP-to-Country actual contiene país y ciudad, no necesariamente coordenadas del cliente. Round Trip usa coordenadas guardadas si existen; de lo contrario estima la ubicación con el centro de los checkers de la misma ciudad y, si no hay, con el centro de los checkers del país. Después elige el checker geográficamente más cercano y su target saludable con menor RTT. Si no hay checker del país o mediciones válidas, cae al DNS público.
- El API trata la consulta de ubicación, ubicaciones de checker y mediciones como datos complementarios: si no están disponibles, el registro DNS sigue respondiendo y Round Trip usa el fallback.


