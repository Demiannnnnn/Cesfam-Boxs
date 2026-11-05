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
    Write-Host "No se pudo consultar la API en http://localhost:8000. No se cargaron boxes."
    exit 1
}

$existentes = @($respuesta | Where-Object { $null -ne $_ })
if ($existentes.Count -gt 0) {
    Write-Host "Ya existen boxes registrados ($($existentes.Count)). No se crearán boxes iniciales."
    exit 0
}

if (-not (Test-Path -LiteralPath $Archivo -PathType Leaf)) {
    Write-Host "Error: No se encontró el archivo de boxes iniciales: $Archivo"
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
        $null = Invoke-RestMethod -Method Post -Uri 'http://localhost:8000/boxes' -Body $cuerpoBytes -ContentType 'application/json; charset=utf-8'
        Write-Host "Box creado: $nombre"
        $creados++
    }
    catch {
        $errorItem = $_
        $detalle = 'la API rechazó la solicitud o no respondió'
        if ($errorItem.ErrorDetails -and -not [string]::IsNullOrWhiteSpace($errorItem.ErrorDetails.Message)) {
            try {
                $detalle = ($errorItem.ErrorDetails.Message | ConvertFrom-Json).mensaje
            }
            catch {
            }
            if (-not $detalle) {
                $detalle = 'la API rechazó la solicitud o no respondió'
            }
        }
        Write-Host "No se pudo crear el box '$nombre': $detalle"
        $fallidos++
    }
}

Write-Host "Resumen: $creados boxes creados, $fallidos con error."
if ($fallidos -gt 0) {
    exit 1
}
exit 0
