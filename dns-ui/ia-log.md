# Registro de uso de IA generativa — DNS UI

Herramienta utilizada: Claude (Anthropic), modo chat.

## Alcance
Se usó IA generativa para todo el componente DNS UI: definición de estructura
del proyecto, formularios de registros DNS, CRUD de IP-to-Country, funciones
RPC de Supabase, Dockerfile y estilos de la interfaz.

## Prompts principales utilizados

| Etapa | Prompt (resumen) |
|---|---|
| Estructura inicial | "Vamos con la estructura" — definir carpetas del proyecto React (api, components, pages, utils) |
| Scaffold para primer PR | "Solo lo básico para el primer pull request" — proyecto mínimo que compile, sin lógica de negocio |
| Formulario de registros | "Con el RecordForm.jsx" — formulario dinámico según tipo de registro (single, multi, weight, geo, round-trip) |
| Esquema de datos | Se compartió el esquema de tablas propuesto por un compañero (Health Checker) y se pidió adaptar el código a ese esquema relacional |
| Guardado atómico | "¿Cómo manejamos el guardado de un registro completo (varias tablas relacionadas)?" — se eligió función RPC de Postgres sobre inserts secuenciales |
| Edición de registros | "¿Cómo manejamos la edición de un registro (cambio de IPs)?" — se eligió diff real (comparar IPs viejas vs nuevas) sobre borrar-y-recrear |
| CRUD de IP-to-Country | "Vamos con CRUD de IP-to-Country" — mismo patrón lista + formulario, sin relaciones |

## Problemas encontrados y cómo se resolvieron

| Problema | Causa | Solución |
|---|---|---|
| `401 Unauthorized` en el RPC | El rol `anon` no tenía permiso para ejecutar la función | `grant execute on function create_dns_record(...) to anon;` |
| `null value in column "check_type" violates not-null constraint` | Un `health_check: null` en JavaScript se serializa como `null` de JSON, que no es lo mismo que `NULL` de SQL — la función igual intentaba insertarlo | Agregar `jsonb_typeof(v_hc) <> 'null'` a la condición, además de `is not null` |
| `invalid input syntax for type inet` | Se ingresó una IP incompleta (ej. `192.168.10`) | Validación de formato de IP en el frontend antes de enviar a Supabase (`utils/validators.js`) |
| Variables de Supabase no llegaban al build de Docker | Vite compila las variables de entorno al momento del `build`, no en runtime | Pasarlas como `--build-arg` en `docker build`, no solo como variables de entorno del contenedor |

## Decisiones de arquitectura tomadas con apoyo de la IA
- Creación y edición de registros usan funciones RPC de Postgres para garantizar
  atomicidad entre `dns_records`, `dns_record_ips` y `health_check_configs`