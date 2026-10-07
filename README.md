# Proyecto 1 — IC7602

## Requisitos

- Docker Desktop
- `kubectl` configurado para un clúster Kubernetes
- Helm 3 o posterior
- `kind` si se usará un clúster local dentro de Docker Desktop

## Arquitectura

```text
Cliente DNS --UDP/53--> dns-interceptor --HTTP/8080--> dns-api --UDP/53--> DNS externo
                                                               └──> Supabase (integración pendiente)
dns-ui --HTTP/80--> navegador --HTTPS--> Supabase
health_checker -----------------------> Supabase (integración pendiente)
```

El DNS API ya implementa las rutas y formatos HTTP consumidos por el Interceptor. Su consulta de registros todavía usa datos de prueba; no se considera una integración completa con Supabase. La UI se conecta directamente a Supabase usando variables `VITE_*` durante el build. El Health Checker compila, pero sigue usando targets mock y no persiste resultados; además, su check HTTP no usa el puerto configurado para el target.

## Estructura

```text
.
├── dns-interceptor/
├── dns-api/
├── dns-ui/
├── health_checker/
├── k8s/ic7602-p1/
├── start.ps1
└── .env.example
```

## Estado actual

**Disponible:** los cuatro componentes tienen Dockerfile; Helm configura UDP/53 para el Interceptor, TCP/8080 para el API y TCP/80 para la UI. El DNS API obtiene el país del cliente desde `ip_to_country` para GEO y entrega al Interceptor ubicaciones de Health Checkers y mediciones RTT para Round Trip. Si IP-to-Country no trae coordenadas, estima la ubicación con los checkers de la misma ciudad o país; el Interceptor elige el checker más cercano y su target saludable con menor RTT. El Health Checker lee targets de Supabase y persiste salud, RTT y ubicación.

**Pendiente:** validar GEO y Round Trip end-to-end con registros de esos tipos, rangos IP-to-Country, ubicaciones de checkers y mediciones reales. Sin esos datos, el resolver usa sus fallbacks; no se afirma que esos flujos estén validados de extremo a extremo.

## Desplegar con `start.ps1`

En PowerShell, desde la raíz:

```powershell
Copy-Item .env.example .env
# Completar localmente los valores necesarios, sin versionar .env.
./start.ps1
```

Si se usa `kind`, crea primero el clúster y selecciona su contexto. El script construye los Dockerfiles utilizables, carga las imágenes a `kind` cuando corresponde, valida Helm y despliega el chart.

## Comando equivalente de Helm

```powershell
helm upgrade --install ic7602-p1 ./k8s/ic7602-p1
```

## Inspeccionar y desinstalar

```powershell
kubectl get pods
kubectl get services
helm uninstall ic7602-p1
```

Para abrir la UI localmente, ejecuta `kubectl port-forward service/dns-ui 8080:80` y visita `http://localhost:8080`.
