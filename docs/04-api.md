# 04 — API

Servidor HTTP JSON en `0.0.0.0:8000` y listener UDP de descubrimiento en `0.0.0.0:8001`.
Todos los cuerpos son `application/json; charset=utf-8`. Sin autenticación.

## Convenciones

- Fechas: texto `AAAA-MM-DD` (ej. `"2026-10-01"`).
- Bloques: entero `0`–`23`. Bloque `n` empieza a las `08:00 + n × 30 min`.
- Marcas de tiempo (`creada_en`): texto ISO-8601 local `AAAA-MM-DDTHH:MM:SS`.
- Todo error responde con el mismo cuerpo:

```json
{ "codigo": "bloque_ocupado", "mensaje": "Ese bloque acaba de ser tomado" }
```

`codigo` es estable y lo usa la app para decidir; `mensaje` está en español y puede mostrarse
directamente al usuario.

| HTTP | `codigo` | Cuándo |
|---|---|---|
| 400 | `datos_invalidos` | JSON mal formado, campo faltante, fecha o bloque fuera de rango, nombre vacío |
| 404 | `no_encontrado` | El box o la reserva no existe |
| 409 | `bloque_ocupado` | Viola `UNIQUE(box_id, fecha, bloque)` (RF-05) |
| 409 | `nombre_duplicado` | Ya existe un box con ese nombre |
| 422 | `box_inactivo` | Se intenta reservar un box desactivado |
| 500 | `error_interno` | Error inesperado (se registra en `api.log`) |

## Endpoints

### `GET /health` — estado del servidor (RF-08)

```json
200 { "estado": "ok", "version": "0.1.0", "equipo": "PC-SOME" }
```

### `GET /boxes` — listar boxes (RF-01, RF-06)

Query opcional `incluir_inactivos=true`. Por defecto solo activos, ordenados por `orden`, luego `nombre`.

```json
200 [ { "id": 1, "nombre": "Box 1", "activo": true, "orden": 1 } ]
```

### `POST /boxes` — crear box (RF-06)

```json
// petición
{ "nombre": "Box 7" }
// 201
{ "id": 7, "nombre": "Box 7", "activo": true, "orden": 7 }
```

Errores: `400 datos_invalidos`, `409 nombre_duplicado`.

### `PATCH /boxes/{id}` — renombrar o activar/desactivar (RF-06)

> El `.docx` solo menciona `GET/POST /boxes`; renombrar y desactivar (RF-06) requieren este
> endpoint adicional. Los boxes **nunca se borran**: desactivar conserva el historial de reservas.

```json
// petición (cualquier subconjunto)
{ "nombre": "Box Dental", "activo": false }
// 200 → BoxAtencion actualizado
```

Errores: `400`, `404 no_encontrado`, `409 nombre_duplicado`.

### `GET /reservas` — listar reservas (RF-01, RF-12)

Una de dos formas:

- `?fecha=2026-10-01` → reservas de ese día.
- `?desde=2026-10-01&hasta=2026-10-31` → rango inclusivo (máx. 366 días), para exportar.

```json
200 [
  { "id": 15, "box_id": 1, "fecha": "2026-10-01", "bloque": 4,
    "funcionario": "María Pérez", "motivo": "Control sano",
    "creada_en": "2026-09-30T16:12:03", "grupo": null }
]
```

### `GET /funcionarios` — nombres para autocompletar

Nombres de funcionarios con reservas en los últimos 180 días, sin repetir (sin distinguir
mayúsculas), los más recientes primero, máximo 300. La app los sugiere al reservar y en el
buscador, para que el mismo funcionario no quede escrito de tres formas.

```json
200 [ "E.U. Olivia", "E.U. Verónica", "Nutri. Javiera" ]
```

### `POST /reservas` — crear reserva (RF-03, RF-05)

```json
// petición
{ "box_id": 1, "fecha": "2026-10-01", "bloque": 4,
  "funcionario": "María Pérez", "motivo": "Control sano" }
// 201 → Reserva creada
```

- `funcionario`: obligatorio, 1–80 caracteres tras recortar espacios.
- `motivo`: opcional, máx. 200 caracteres; vacío se guarda como `null`.

Errores: `400 datos_invalidos`, `404 no_encontrado` (box), `422 box_inactivo`,
**`409 bloque_ocupado` → "Ese bloque acaba de ser tomado"**.

### `POST /reservas/lote` — rango de bloques y series semanales (RF-03, RF-05)

Crea los bloques `desde..=hasta` de un box en cada una de las `fechas` (máx. 46), en **una
sola transacción**. Todas las reservas creadas comparten `grupo` (el id de la primera), para
poder cancelar la serie de una vez. La exclusividad sigue en manos del `UNIQUE` (ADR-003).

```json
// petición: toda la mañana, tres lunes seguidos
{ "box_id": 17, "fechas": ["2026-10-05", "2026-10-12", "2026-10-19"],
  "desde": 0, "hasta": 11, "funcionario": "E.U. Olivia", "motivo": null,
  "omitir_ocupados": false }
// 201
{ "creadas": [ { "id": 40, "grupo": 40, "...": "..." } ], "omitidos": [] }
```

- `omitir_ocupados: false` (por defecto): si un bloque ya está tomado **no se crea nada** →
  `409 bloque_ocupado`.
- `omitir_ocupados: true`: se crean los libres y los ocupados vuelven en
  `omitidos: [{ "fecha": "2026-10-12", "bloque": 3 }]`. Si no quedó ninguno libre → `409`.

Errores: `400 datos_invalidos` (`desde > hasta`, sin fechas o más de 46), `404` (box),
`422 box_inactivo`, `409 bloque_ocupado`.

### `DELETE /reservas/grupo/{grupo}?desde=AAAA-MM-DD` — cancelar una serie (RF-04)

Elimina las reservas del grupo con fecha `>= desde` ("este día y los siguientes"); las
anteriores se conservan. `200 { "canceladas": 4 }`. `404 no_encontrado` si no quedaba
ninguna. `desde` es obligatorio.

### `DELETE /reservas/{id}` — cancelar (RF-04)

`204` sin cuerpo. `404 no_encontrado` si ya no existe (por ejemplo, otro PC la canceló antes).

## Esquema SQLite

```sql
CREATE TABLE IF NOT EXISTS box (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre  TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    activo  INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0, 1)),
    orden   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS reserva (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    box_id      INTEGER NOT NULL REFERENCES box(id),
    fecha       TEXT    NOT NULL,                       -- AAAA-MM-DD
    bloque      INTEGER NOT NULL CHECK (bloque BETWEEN 0 AND 23),
    funcionario TEXT    NOT NULL,
    motivo      TEXT,
    creada_en   TEXT    NOT NULL,
    grupo       INTEGER,                                -- v2: rango o serie (ADR-015)
    UNIQUE (box_id, fecha, bloque)                      -- RF-05
);

CREATE INDEX IF NOT EXISTS idx_reserva_fecha ON reserva (fecha);
CREATE INDEX IF NOT EXISTS idx_reserva_grupo ON reserva (grupo);   -- v2
```

`PRAGMA`s al abrir: `foreign_keys = ON`, `journal_mode = WAL`, `busy_timeout = 5000`.
La versión del esquema se guarda en `PRAGMA user_version` para migraciones futuras.

## Protocolo de descubrimiento UDP (RF-09)

1. El cliente envía el datagrama ASCII `GESTIONBOX_BUSCAR` a `255.255.255.255:8001`.
2. La API responde **al origen** con JSON:

```json
{ "servicio": "gestionbox", "version": "0.1.0", "puerto_http": 8000, "equipo": "PC-SOME" }
```

3. El cliente usa la IP de origen de la respuesta + `puerto_http` y confirma con `GET /health`.

Cualquier otro contenido recibido se ignora (no se responde), para no amplificar tráfico.

## Archivos en el servidor (RNF-04)

```
C:\ProgramData\GestionBox\
├── gestion-box-api.exe
├── nssm.exe
├── gestion-box.db          base de datos
├── backups\                un respaldo por arranque, máximo 30
│   └── gestion-box-20261001-080102.db
├── api.log                 eventos y errores
└── api.anterior.log        log previo (se rota al arrancar si api.log supera 10 MB)
```

## Probar a mano

```bash
curl -s localhost:8000/health
curl -s localhost:8000/boxes
curl -s -X POST localhost:8000/boxes -H 'content-type: application/json' -d '{"nombre":"Box 1"}'
curl -s "localhost:8000/reservas?fecha=2026-10-01"
curl -s -X POST localhost:8000/reservas -H 'content-type: application/json' \
     -d '{"box_id":1,"fecha":"2026-10-01","bloque":4,"funcionario":"María Pérez"}'
curl -s -i -X DELETE localhost:8000/reservas/1
```
