# DNS UI

Componente de interfaz (React) del Proyecto 1 — Redes (IC7602), TEC.

Permite crear, listar, editar y borrar registros DNS (`single`, `multi`, `weight`, `geo`, `round-trip`) junto con sus health checks asociados, y gestionar la tabla de IP-to-Country. Se conecta directamente a Supabase.

## Requisitos

- Docker
- Acceso al proyecto de Supabase compartido del equipo (URL + anon key)

## Correr con Docker

Configura `VITE_SUPABASE_URL` y `VITE_SUPABASE_ANON_KEY` localmente y pásalas como argumentos de build. No uses una clave privada/service-role como clave del frontend ni guardes credenciales reales en este README.

```powershell
docker build -t dns-ui `
  --build-arg VITE_SUPABASE_URL=https://supabase.example.invalid `
  --build-arg VITE_SUPABASE_ANON_KEY=CLAVE_ANON_SUPABASE_DE_EJEMPLO `
  .

docker run -p 8080:80 dns-ui
```

Abrir `http://localhost:8080`.

> Importante: Vite compila las variables de entorno **al momento del build**, no en runtime. Por eso van como `--build-arg`, no como variables normales del contenedor (`-e`). Si se te olvida pasarlas, el build igual va a funcionar pero la app va a fallar al conectar a Supabase.

## Base de datos (Supabase)

El proyecto usa un Supabase compartido por todo el equipo (plan gratuito). 

| Archivo | Contenido |
|---|---|
| `0001_create_tables.sql` | Tablas: `dns_records`, `dns_record_ips`, `health_check_configs`, `ip_to_country`, `health_checker_locations` |
| `0002_create_dns_record_function.sql` | Función RPC `create_dns_record`: inserta un registro completo (IPs + health checks) en una sola transacción |
| `0003_update_dns_record_function.sql` | Función RPC `update_dns_record`: actualiza un registro existente, comparando IPs viejas vs. nuevas |

Para aplicar las migraciones contra una instancia nueva de Supabase:

```bash
export DATABASE_URL="postgresql://postgres:<password>@db.<proyecto>.supabase.co:5432/postgres"
sh supabase/apply-migrations.sh
```

Permisos necesarios después de crear las funciones:

```sql
grant execute on function create_dns_record(text, text, jsonb) to anon;
grant execute on function update_dns_record(uuid, text, text, jsonb) to anon;
```

Se obtiene en Supabase → Project Settings → Database → Connection string.

## Arquitectura

DNS UI (React) ──► Supabase (Postgres, en la nube)
├─ CRUD directo sobre las tablas (lectura/edición)
└─ RPC create_dns_record / update_dns_record (escritura atómica)


El DNS Interceptor, DNS API y Health Checker leen/escriben las mismas tablas de Supabase, según el esquema compartido del equipo.

## Estructura del proyecto

dns-ui/
├── Dockerfile
├── nginx.conf
├── .dockerignore
├── .env.example
├── docs/
│ └── ia-log.md
├── supabase/
│ ├── apply-migrations.sh
│ └── migrations/
│ ├── 0001_create_tables.sql
│ ├── 0002_create_dns_record_function.sql
│ └── 0003_update_dns_record_function.sql
└── src/
├── api/
│ ├── supabaseClient.js
│ ├── records.js
│ └── geoIp.js
├── components/
│ ├── records/
│ │ ├── RecordForm.jsx
│ │ ├── RecordList.jsx
│ │ └── IpListEditor.jsx
│ └── geoIp/
│ ├── GeoIpForm.jsx
│ └── GeoIpList.jsx
├── styles/
│ └── global.css
└── App.jsx


## Estado actual

- [x] Scaffold del proyecto (Vite + React)
- [x] Esquema de base de datos y funciones RPC (crear/actualizar)
- [x] CRUD de registros DNS (`single`, `multi`, `weight`, `geo`, `round-trip`)
- [x] Health check opcional por IP (TCP/HTTP)
- [x] CRUD de IP-to-Country
- [x] Dockerfile (build multi-stage con nginx)
- [x] Diseño de interfaz
- [x] Registro de uso de IA generativa (`docs/ia-log.md`)

## Uso de IA generativa

Este componente se desarrolló con asistencia de Claude (Anthropic). Prompts y problemas encontrados documentados en `docs/ia-log.md`.
