param(
    [Parameter(Mandatory)]
    [string]$Archivo
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

try {
    $respuesta = Invoke-RestMethod -Method Get -Uri 'http://localhost:8000/boxes?incluir_inactivos=true'
}
catch {
    Write-Host "Error al consultar los boxes existentes en la API: $_"
    exit 1
}

$existentes = @($respuesta | Where-Object { $null -ne $_ })
if ($existentes.Count -gt 0) {
    Write-Host "Ya existen boxes registrados ($($existentes.Count)). No se crearan boxes iniciales."
    exit 0
}

if (-not (Test-Path -LiteralPath $Archivo -PathType Leaf)) {
    Write-Host "Error: No se encontro el archivo de boxes iniciales: $Archivo"
    exit 1
}

$lineas = Get-Content -LiteralPath $Archivo -Encoding UTF8
$nombres = @(
    foreach ($linea in $lineas) {
        $limpio = $linea.Trim()
        if (-not [string]::IsNullOrWhiteSpace($limpio) -and -not $limpio.StartsWith('#')) {
            $limpio
        }
    }
)

if ($nombres.Count -eq 0) {
    Write-Host "No hay nombres de boxes para procesar en $Archivo."
    exit 0
}

$creados = 0
$fallidos = 0

foreach ($nombre in $nombres) {
    $cuerpoJson = [PSCustomObject]@{ nombre = $nombre } | ConvertTo-Json -Compress
    $cuerpoBytes = [System.Text.Encoding]::UTF8.GetBytes($cuerpoJson)
    try {
        $nuevo = Invoke-RestMethod -Method Post -Uri 'http://localhost:8000/boxes' -Body $cuerpoBytes -ContentType 'application/json; charset=utf-8'
        Write-Host "Box creado: $($nuevo.nombre) (ID: $($nuevo.id))"
        $creados++
    }
    catch {
        Write-Host "Error al crear el box '$nombre': $_"
        $fallidos++
    }
}

Write-Host "Resumen: $creados boxes creados, $fallidos con error."
if ($fallidos -gt 0) {
    exit 1
}
exit 0
