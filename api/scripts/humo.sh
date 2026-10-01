#!/usr/bin/env bash
# Prueba de humo de la API (T-2.11): levanta la API en una carpeta temporal y
# recorre health → crear box → reservar → 409 → listar → cancelar → 404.
# Uso: bash api/scripts/humo.sh   (desde la raíz del repositorio)
set -euo pipefail

PUERTO_HTTP="${PUERTO_HTTP:-18000}"
PUERTO_UDP="${PUERTO_UDP:-18001}"
BASE="http://127.0.0.1:${PUERTO_HTTP}"
CARPETA="$(mktemp -d)"
FECHA="2026-10-01"

cargo build -q -p gestion-box-api
GESTIONBOX_DATA_DIR="$CARPETA" GESTIONBOX_HTTP_PORT="$PUERTO_HTTP" GESTIONBOX_UDP_PORT="$PUERTO_UDP" \
    ./target/debug/gestion-box-api 2>/dev/null &
PID=$!
trap 'kill $PID 2>/dev/null || true; rm -rf "$CARPETA"' EXIT

for _ in $(seq 1 50); do
    curl -sf "$BASE/health" >/dev/null && break
    sleep 0.1
done

fallas=0
verificar() { # verificar <descripción> <código esperado> <método> <ruta> [cuerpo]
    local descripcion=$1 esperado=$2 metodo=$3 ruta=$4 cuerpo=${5:-}
    local args=(-s -o /tmp/humo-cuerpo.$$ -w '%{http_code}' -X "$metodo" "$BASE$ruta")
    [[ -n $cuerpo ]] && args+=(-H 'content-type: application/json' -d "$cuerpo")
    local obtenido
    obtenido=$(curl "${args[@]}")
    if [[ $obtenido == "$esperado" ]]; then
        echo "  ok   $descripcion ($obtenido)"
    else
        echo "  FALLA $descripcion: esperado $esperado, obtenido $obtenido — $(cat /tmp/humo-cuerpo.$$)"
        fallas=$((fallas + 1))
    fi
}

echo "Prueba de humo contra $BASE (datos en $CARPETA)"
verificar "health"                 200 GET    /health
verificar "crear box"              201 POST   /boxes '{"nombre":"Box 1"}'
verificar "box duplicado"          409 POST   /boxes '{"nombre":"box 1"}'
verificar "listar boxes"           200 GET    /boxes
verificar "reservar"               201 POST   /reservas "{\"box_id\":1,\"fecha\":\"$FECHA\",\"bloque\":4,\"funcionario\":\"María Pérez\"}"
verificar "reservar mismo bloque"  409 POST   /reservas "{\"box_id\":1,\"fecha\":\"$FECHA\",\"bloque\":4,\"funcionario\":\"Juan Soto\"}"
verificar "listar reservas"        200 GET    "/reservas?fecha=$FECHA"
verificar "cancelar"               204 DELETE /reservas/1
verificar "cancelar de nuevo"      404 DELETE /reservas/1
verificar "desactivar box"         200 PATCH  /boxes/1 '{"activo":false}'

respuesta_udp=$(python3 - "$PUERTO_UDP" <<'PY' 2>/dev/null || true
import socket, sys
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.settimeout(2)
s.sendto(b"GESTIONBOX_BUSCAR", ("127.0.0.1", int(sys.argv[1])))
print(s.recvfrom(512)[0].decode())
PY
)
if [[ $respuesta_udp == *'"servicio":"gestionbox"'* ]]; then
    echo "  ok   descubrimiento UDP"
else
    echo "  FALLA descubrimiento UDP sin respuesta"
    fallas=$((fallas + 1))
fi

[[ -f "$CARPETA/api.log" ]] && grep -q "reserva creada" "$CARPETA/api.log" \
    && echo "  ok   api.log registra eventos" || { echo "  FALLA api.log"; fallas=$((fallas + 1)); }
ls "$CARPETA/backups"/gestion-box-*.db >/dev/null 2>&1 \
    && echo "  ok   respaldo al iniciar" || { echo "  FALLA respaldo"; fallas=$((fallas + 1)); }

rm -f /tmp/humo-cuerpo.$$
if ((fallas > 0)); then
    echo "Prueba de humo FALLÓ ($fallas fallas)"
    exit 1
fi
echo "Prueba de humo OK"
