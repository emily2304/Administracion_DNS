$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

foreach ($tool in @('docker', 'kubectl', 'helm')) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Write-Error "$tool es obligatorio, pero no se encontró en PATH."
        exit 1
    }
}

& docker info | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Error 'Docker está instalado, pero el daemon no está disponible.'
    exit $LASTEXITCODE
}

$envValues = @{}
$envFile = Join-Path $root '.env'
if (Test-Path -LiteralPath $envFile -PathType Leaf) {
    foreach ($line in Get-Content -LiteralPath $envFile) {
        $text = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($text) -or $text.StartsWith('#')) { continue }

        $separator = $text.IndexOf('=')
        if ($separator -lt 1) { continue }

        $name = $text.Substring(0, $separator).Trim()
        $value = $text.Substring($separator + 1).Trim()
        if (($value.StartsWith('"') -and $value.EndsWith('"')) -or
            ($value.StartsWith("'") -and $value.EndsWith("'"))) {
            $value = $value.Substring(1, $value.Length - 2)
        }
        $envValues[$name] = $value
    }
}

function Get-ConfigValue {
    param(
        [string]$Name,
        [string]$Default
    )

    $processValue = [Environment]::GetEnvironmentVariable($Name)
    if (-not [string]::IsNullOrWhiteSpace($processValue)) { return $processValue }
    if ($envValues.ContainsKey($Name) -and -not [string]::IsNullOrWhiteSpace($envValues[$Name])) {
        return $envValues[$Name]
    }
    return $Default
}

function Test-UsableDockerfile {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { return $false }
    $lines = @(Get-Content -LiteralPath $Path | Where-Object {
        $line = $_.Trim()
        $line -and -not $line.StartsWith('#')
    })
    return ($lines.Count -gt 0 -and ($lines -match '^\s*FROM\s+').Count -gt 0)
}

$viteSupabaseUrl = Get-ConfigValue 'VITE_SUPABASE_URL' 'https://supabase.example.invalid'
$viteSupabaseAnonKey = Get-ConfigValue 'VITE_SUPABASE_ANON_KEY' 'CLAVE_ANON_SUPABASE_DE_EJEMPLO'
$supabaseUrl = Get-ConfigValue 'SUPABASE_URL' 'https://supabase.example.invalid'
$supabaseKey = Get-ConfigValue 'SUPABASE_KEY' 'CLAVE_SUPABASE_DE_EJEMPLO'

$services = @(
    @{ Source = 'dns-interceptor'; Image = 'ic7602-p1/dns-interceptor:latest' },
    @{ Source = 'dns-api'; Image = 'ic7602-p1/dns-api:latest' },
    @{ Source = 'dns-ui'; Image = 'ic7602-p1/dns-ui:latest' },
    @{ Source = 'health_checker'; Image = 'ic7602-p1/health-checker:latest' }
)
$builtImages = @()

foreach ($service in $services) {
    $directory = Join-Path $root $service.Source
    $dockerfile = Join-Path $directory 'Dockerfile'

    if (-not (Test-UsableDockerfile $dockerfile)) {
        Write-Host "Omitiendo $($service.Source): Dockerfile ausente, vacío o inválido."
        continue
    }

    $arguments = @('build', '--tag', $service.Image)
    if ($service.Source -eq 'dns-ui') {
        $arguments += @('--build-arg', "VITE_SUPABASE_URL=$viteSupabaseUrl")
        $arguments += @('--build-arg', "VITE_SUPABASE_ANON_KEY=$viteSupabaseAnonKey")
    }
    $arguments += $directory

    Write-Host "Construyendo $($service.Source)..."
    & docker @arguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $builtImages += $service.Image
}

$currentContext = (& kubectl config current-context 2>$null).Trim()
if ($LASTEXITCODE -eq 0 -and $currentContext -like 'kind-*') {
    if (-not (Get-Command kind -ErrorAction SilentlyContinue)) {
        Write-Error 'El contexto kind está activo, pero kind no está instalado.'
        exit 1
    }

    $kindCluster = $currentContext.Substring(5)
    foreach ($image in $builtImages) {
        Write-Host "Cargando $image en el clúster kind $kindCluster..."
        & kind load docker-image $image --name $kindCluster
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
}

Write-Host 'Validando chart Helm...'
$helmValuesPath = Join-Path ([System.IO.Path]::GetTempPath()) "ic7602-p1-$([guid]::NewGuid().ToString('N')).json"
$helmOverrides = @{
    config = @{ supabaseUrl = $supabaseUrl }
    secret = @{ supabaseKey = $supabaseKey }
} | ConvertTo-Json -Depth 3

try {
    [System.IO.File]::WriteAllText($helmValuesPath, $helmOverrides, [System.Text.UTF8Encoding]::new($false))

    & helm lint ./k8s/ic7602-p1 --values $helmValuesPath
    if ($LASTEXITCODE -ne 0) { throw 'helm lint falló.' }

    & helm template ic7602-p1 ./k8s/ic7602-p1 --values $helmValuesPath | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'helm template falló.' }
    Write-Host 'helm template pasó; se ocultó su salida para no imprimir la clave.'

    & helm upgrade --install ic7602-p1 ./k8s/ic7602-p1 --values $helmValuesPath
    if ($LASTEXITCODE -ne 0) { throw 'helm upgrade --install falló.' }
}
finally {
    if (Test-Path -LiteralPath $helmValuesPath) {
        Remove-Item -LiteralPath $helmValuesPath -Force -ErrorAction SilentlyContinue
    }
}

$deployments = @('dns-interceptor', 'dns-api', 'dns-ui')
$deploymentNames = & kubectl get deployment -o jsonpath='{.items[*].metadata.name}' 2>$null
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$healthCheckerDeployments = $deploymentNames -split '\s+' | Where-Object { $_ -like '*checker*' -and $_ -ne '' }
if ($healthCheckerDeployments) { $deployments += $healthCheckerDeployments }

foreach ($deployment in $deployments) {
    Write-Host "Reiniciando deployment/$deployment para usar las imágenes locales recién construidas..."
    & kubectl rollout restart "deployment/$deployment"
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

foreach ($deployment in $deployments) {
    & kubectl rollout status "deployment/$deployment" --timeout=120s
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "`nPods:"
& kubectl get pods
Write-Host "`nServicios:"
& kubectl get services
