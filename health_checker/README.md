# Health Checker

## Estructura de archivos

| Archivo | Qué hace |
|---|---|
| `main.c` | Carga la configuración. Se tiene un loop infinito donde trae los targets, lanza un thread por target, espera a que todos terminen, y duerme hasta el siguiente ciclo. |
| `config.c` / `config.h` | Lee la configuración desde variables de entorno y la deja disponible en la estructura `AppConfig`. |
| `db_client.c` / `db_client.h` | Define `HealthTarget` (la estructura de un target a chequear) y las funciones `db_fetch_targets()` / `db_write_result()`. **Ahorita mismo es un stub**: retorna 3 targets fijos de prueba y solo imprime el resultado por consola. |
| `tcp_check.c` / `tcp_check.h` | Ejecuta un intento de conexión TCP con timeout, usando sockets no bloqueantes. Retorna éxito/fallo y el RTT medido. |
| `http_check.c` / `http_check.h` | Ejecuta un intento de petición HTTP, usando `libcurl`. Retorna éxito/fallo y el RTT medido. |
| `health_evaluator.c` / `health_evaluator.h` | Repite el check (TCP o HTTP) hasta la cantidad de tries definidas por el target y decide el estado final (`healthy`/`unhealthy`) por mayoría simple de los intentos. También, se calcula el RTT promedio de los intentos exitosos. |
| `target_task.c` / `target_task.h` | Función que corre dentro de cada thread (`pthread_create`). Llama a `health_evaluator_run()` sobre su target asignado y escribe el resultado con `db_write_result()`. Es lo que permite que los targets se chequeen de forma simultanea. |
| `location.c` / `location.h` | Calcula la distancia entre dos coordenadas geográficas usando la fórmula de Haversine y encuentra cuál checker de una lista está más cerca de un punto dado. Usado para el tipo de registro `round-trip`. **TODO: conectarlo a `main.c`**, la idea es integrarlo junto con la tabla `health_checker_locations` en Supabase. |

## Pendientes

### Múltiples Health Checkers (round-trip)

El chart de Helm actual usa `replicaCount.healthChecker` con un único
`Deployment`, y las variables `CHECKER_ID`/`CHECKER_LATITUDE`/etc. vienen
del mismo ConfigMap para todas las réplicas. Esto significa que **todas
las réplicas tendrían la misma ubicación simulada**, lo cual no sirve
para el tipo de registro "round-trip" porque se necesitan checkers en
ubicaciones distintas.

Como ideas para implementar esta parte, se tenien:
- Definir cuántas ubicaciones simuladas se quieren.
- Crear un `Deployment` + `ConfigMap` separado por ubicación (o
  parametrizar el chart con un `range` sobre una lista de ubicaciones
  en `values.yaml`).

### Implementación con Supabase

Actualmente, se trabaja en varios archivos con datos mock. Por lo que falta la integración final con la base de datos y los demás módulos del proyecto.

## Pruebas locales
Actualmente, las pruebas se pueden correr localmente. En este caso se realizaron en **WSL**. Entonces se tienen los siguientes requisitos:
- Tener WSL
- Tener las dependencias de compilación:
    ```bash
    sudo apt update
    sudo apt install -y gcc make libcurl4-openssl-dev
    ```

### Compilar localmente

```bash
make
```

## Configurar variables de entorno

Se puede copiar el archivo de ejemplo y ajusta los valores del `.env`,`checker-cr.env` y `checker-esp.env`con los valores de pruebas:

```bash
cp .env.example .env
```

Cargar las variables de entorno en la sesión actual:

En una terminal, se puede tener el health checker de Costa Rica (por ejemplo) y en otra terminal el de España (por ejemplo).

- En terminales separadas:

```bash
set -a; source .env; source checker-cr.env; set +a
```

```bash
set -a; source .env; source checker-esp.env; set +a
```

## Probar contra un servidor real

En otra terminal de WSL, se puede levantar el servidor de prueba:

```bash
python3 -m http.server 8080
```

Para detener este servidor:

```bash
Ctrl+C
```

## Ejecutar el binario

Con el servidor ejecutandose, se tiene que ingresar en la terminal principal de WSL:

```bash
./build/health_checker
```

Con esto corriendo, `test-record-1` debería salir `healthy` (porque algo responde en el puerto 8080), y `test-record-2` y `test-record-3` deberían salir `unhealthy` (nada escucha en el puerto 22 para el caso del `test-record-2`). El caso del `test-record-3` es el que está documentado como primer pendiente en el documento para revisar el estado `healthy` ya que se usa el puerto default (80).