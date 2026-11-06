# 04 — API

Servidor HTTP JSON en `0.0.0.0:8000` y listener UDP de descubrimiento en `0.0.0.0:8001`.
Todos los cuerpos son `application/json; charset=utf-8`. Sin autenticación.

## Convenciones

- Fechas: texto `AAAA-MM-DD` (ej. `"2026-10-01"`).
- Bloques: entero `0`–`23`. Bloque `n` empieza a las `08:00 + n × 30 min`.
- Colores: texto hexadecimal `#rrggbb` (acepta mayúsculas o minúsculas y lo normaliza a minúsculas).
- Marcas de tiempo (`creada_en`): texto ISO-8601 local `AAAA-MM-DDTHH:MM:SS`.
- Todo error responde con el mismo cuerpo:

```json
{ "codigo": "bloque_ocupado", "mensaje": "Ese bloque acaba de ser tomado" }
```

`codigo` es estable y lo usa la app para decidir; `mensaje` está en español y puede mostrarse
directamente al usuario.

| HTTP | `codigo` | Cuándo |
|---|---|---|
| 400 | `datos_invalidos` | JSON mal formado, campo faltante, fecha/bloque/color inválido, nombre vacío |
| 404 | `no_encontrado` | El box, la etiqueta, el funcionario o la reserva no existe |
| 409 | `bloque_ocupado` | Viola `UNIQUE(box_id, fecha, bloque)` (RF-05) |
| 409 | `nombre_duplicado` | Ya existe un box, funcionario o etiqueta con ese nombre |
| 422 | `box_inactivo` | Se intenta reservar un box desactivado |
| 422 | `funcionario_inactivo` | Se intenta reservar para un funcionario desactivado |
| 422 | `etiqueta_inactiva` | Se intenta asignar una etiqueta desactivada |
| 500 | `error_interno` | Error inesperado (se registra en `api.log`) |

## Endpoints

### `GET /health` — estado del servidor (RF-08)

```json
200 { "estado": "ok", "version": "0.1.0", "equipo": "PC-SOME" }
```

### `GET /boxes` — listar boxes (RF-01, RF-06, RF-14)

Query opcional `incluir_inactivos=true`. Por defecto solo activos, ordenados por `orden`, luego `nombre`.

```json
200 [ { "id": 1, "nombre": "Box 1", "activo": true, "orden": 1, "etiqueta_id": null } ]
```

### `POST /boxes` — crear box (RF-06, RF-14)

```json
// petición
{ "nombre": "Box 7", "etiqueta_id": 2 }
// 201
{ "id": 7, "nombre": "Box 7", "activo": true, "orden": 7, "etiqueta_id": 2 }
```

Errores: `400 datos_invalidos`, `404 no_encontrado` (etiqueta), `409 nombre_duplicado`, `422 etiqueta_inactiva`.

### `PATCH /boxes/{id}` — renombrar, cambiar etiqueta o activar/desactivar (RF-06, RF-14)

> El `.docx` solo menciona `GET/POST /boxes`; renombrar y desactivar (RF-06) requieren este
> endpoint adicional. Los boxes **nunca se borran**: desactivar conserva el historial de reservas.

```json
// petición (cualquier subconjunto; "etiqueta_id": null quita la etiqueta)
{ "nombre": "Box Dental", "activo": false, "etiqueta_id": null }
// 200 → BoxAtencion actualizado
```

Errores: `400`, `404 no_encontrado`, `409 nombre_duplicado`, `422 etiqueta_inactiva`.

### `GET /boxes/libres` — buscar boxes libres en un rango horario (RF-15)

Query: `?fecha=2026-10-06&desde=12&hasta=17&etiqueta_id=1` (`etiqueta_id` opcional).

- `fecha`: fecha `AAAA-MM-DD`.
- `desde` y `hasta`: bloques `0`–`23` con `desde <= hasta`.
- `etiqueta_id`: opcional; filtra por la etiqueta asignada al box.

Devuelve los boxes **activos** que no tienen ninguna reserva en `desde..=hasta` de esa fecha
(una sola consulta con `NOT EXISTS`), ordenados igual que en la grilla (`orden`, `nombre`).

```json
200 [ { "id": 2, "nombre": "Box 2", "activo": true, "orden": 2, "etiqueta_id": 1 } ]
```

Errores: `400 datos_invalidos` (`desde > hasta`, fecha o bloque inválido).

### `GET /etiquetas` — listar etiquetas (RF-14)

Query opcional `incluir_inactivos=true`. Por defecto solo activas, ordenadas alfabéticamente por `nombre`.

```json
200 [ { "id": 1, "nombre": "Enfermería", "color": "#2563eb", "activo": true } ]
```

### `POST /etiquetas` — crear etiqueta (RF-14)

```json
// petición
{ "nombre": "Enfermería", "color": "#2563eb" }
// 201 → Etiqueta creada
```

- `nombre`: obligatorio, 1–40 caracteres tras recortar espacios, único sin distinguir mayúsculas.
- `color`: formato `#RRGGBB` (se guarda en minúsculas).

Errores: `400 datos_invalidos`, `409 nombre_duplicado`.

### `PATCH /etiquetas/{id}` — renombrar, cambiar color o activar/desactivar (RF-14)

```json
// petición (cualquier subconjunto)
{ "nombre": "Cirugía Menor", "color": "#dc2626", "activo": false }
// 200 → Etiqueta actualizada
```

Errores: `400`, `404 no_encontrado`, `409 nombre_duplicado`.

### `GET /funcionarios` — listar funcionarios registrados (RF-13, RF-14)

Query opcional `incluir_inactivos=true`. Por defecto solo activos, ordenados alfabéticamente por `nombre`.

```json
200 [ { "id": 1, "nombre": "E.U. Olivia", "etiqueta_id": 1, "activo": true } ]
```

### `POST /funcionarios` — registrar funcionario (RF-13, RF-14)

```json
// petición
{ "nombre": "E.U. Olivia", "etiqueta_id": 1 }
// 201 → Funcionario creado
```

- `nombre`: obligatorio, 1–80 caracteres tras recortar espacios, único sin distinguir mayúsculas.
- `etiqueta_id`: opcional; debe existir y estar activa.

Errores: `400 datos_invalidos`, `404 no_encontrado` (etiqueta), `409 nombre_duplicado`, `422 etiqueta_inactiva`.

### `PATCH /funcionarios/{id}` — renombrar, cambiar etiqueta o activar/desactivar (RF-13, RF-14)

```json
// petición (cualquier subconjunto; "etiqueta_id": null quita la etiqueta)
{ "nombre": "E.U. Olivia Rojas", "etiqueta_id": null, "activo": false }
// 200 → Funcionario actualizado
```

Renombrar un funcionario actualiza el nombre mostrado en sus reservas existentes (`LEFT JOIN funcionario`).
Desactivar un funcionario o etiqueta **no** borra ni modifica las reservas ya hechas.

Errores: `400`, `404 no_encontrado`, `409 nombre_duplicado`, `422 etiqueta_inactiva`.

### `GET /reservas` — listar reservas (RF-01, RF-12)

Una de dos formas:

- `?fecha=2026-10-01` → reservas de ese día.
- `?desde=2026-10-01&hasta=2026-10-31` → rango inclusivo (máx. 366 días), para exportar.

```json
200 [
  { "id": 15, "box_id": 1, "fecha": "2026-10-01", "bloque": 4,
    "funcionario_id": 3, "funcionario": "María Pérez", "motivo": "Control sano",
    "creada_en": "2026-09-30T16:12:03", "grupo": null }
]
```

### `GET /reservas/buscar` — buscar reservas (RF-15)

Query: `?desde=2026-10-01&hasta=2026-10-31&funcionario_id=1&etiqueta_id=2&box_id=3`.

- `desde` y `hasta`: obligatorios (`desde <= hasta`, máx. 366 días).
- `funcionario_id`, `etiqueta_id`, `box_id`: opcionales y combinables (`AND`); **al menos uno de los tres es obligatorio** (`400 datos_invalidos` con mensaje `"Indica al menos un funcionario, etiqueta o box"`).
- `etiqueta_id`: filtra las reservas donde la etiqueta pertenezca **al funcionario o al box**.

Devuelve `[Reserva]` ordenado por `(fecha, box_id, bloque)`, hasta un máximo de 2000 filas.

Errores: `400 datos_invalidos`.

### `POST /reservas` — crear reserva (RF-03, RF-05, RF-13)

```json
// petición
{ "box_id": 1, "fecha": "2026-10-01", "bloque": 4,
  "funcionario_id": 3, "motivo": "Control sano" }
// 201 → Reserva creada
```

- `funcionario_id`: identificador de un funcionario registrado y activo.
- `motivo`: opcional, máx. 200 caracteres; vacío se guarda como `null`.

Errores: `400 datos_invalidos`, `404 no_encontrado` (box o funcionario), `422 box_inactivo`,
`422 funcionario_inactivo`, **`409 bloque_ocupado` → "Ese bloque acaba de ser tomado"**.

### `POST /reservas/lote` — rango de bloques y series semanales (RF-03, RF-05, RF-13)

Crea los bloques `desde..=hasta` de un box en cada una de las `fechas` (máx. 46), en **una
sola transacción**. Todas las reservas creadas comparten `grupo` (el id de la primera), para
poder cancelar la serie de una vez. La exclusividad sigue en manos del `UNIQUE` (ADR-003).

```json
// petición: toda la mañana, tres lunes seguidos
{ "box_id": 17, "fechas": ["2026-10-05", "2026-10-12", "2026-10-19"],
  "desde": 0, "hasta": 11, "funcionario_id": 1, "motivo": null,
  "omitir_ocupados": false }
// 201
{ "creadas": [ { "id": 40, "grupo": 40, "...": "..." } ], "omitidos": [] }
```

- `omitir_ocupados: false` (por defecto): si un bloque ya está tomado **no se crea nada** →
  `409 bloque_ocupado`.
- `omitir_ocupados: true`: se crean los libres y los ocupados vuelven en
  `omitidos: [{ "fecha": "2026-10-12", "bloque": 3 }]`. Si no quedó ninguno libre → `409`.

Errores: `400 datos_invalidos` (`desde > hasta`, sin fechas o más de 46), `404` (box o funcionario),
`422 box_inactivo`, `422 funcionario_inactivo`, `409 bloque_ocupado`.

### `DELETE /reservas/grupo/{grupo}?desde=AAAA-MM-DD` — cancelar una serie (RF-04)

Elimina las reservas del grupo con fecha `>= desde` ("este día y los siguientes"); las
anteriores se conservan. `200 { "canceladas": 4 }`. `404 no_encontrado` si no quedaba
ninguna. `desde` es obligatorio.

### `DELETE /reservas/{id}` — cancelar (RF-04)

`204` sin cuerpo. `404 no_encontrado` si ya no existe (por ejemplo, otro PC la canceló antes).

## Esquema SQLite

```sql
CREATE TABLE IF NOT EXISTS etiqueta (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre  TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    color   TEXT    NOT NULL CHECK (length(color) = 7 AND substr(color, 1, 1) = '#'),
    activo  INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0, 1))
);

CREATE TABLE IF NOT EXISTS funcionario (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre      TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    etiqueta_id INTEGER REFERENCES etiqueta(id),
    activo      INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0, 1))
);

CREATE TABLE IF NOT EXISTS box (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre      TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    activo      INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0, 1)),
    orden       INTEGER NOT NULL DEFAULT 0,
    etiqueta_id INTEGER REFERENCES etiqueta(id)              -- v3: ADR-017
);

CREATE TABLE IF NOT EXISTS reserva (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    box_id         INTEGER NOT NULL REFERENCES box(id),
    fecha          TEXT    NOT NULL,                         -- AAAA-MM-DD
    bloque         INTEGER NOT NULL CHECK (bloque BETWEEN 0 AND 23),
    funcionario    TEXT    NOT NULL,
    motivo         TEXT,
    creada_en      TEXT    NOT NULL,
    grupo          INTEGER,                                  -- v2: rango o serie (ADR-015)
    funcionario_id INTEGER REFERENCES funcionario(id),       -- v3: ADR-017
    UNIQUE (box_id, fecha, bloque)                           -- RF-05
);

CREATE INDEX IF NOT EXISTS idx_reserva_fecha ON reserva (fecha);
CREATE INDEX IF NOT EXISTS idx_reserva_grupo ON reserva (grupo);               -- v2
CREATE INDEX IF NOT EXISTS idx_reserva_funcionario ON reserva (funcionario_id); -- v3
```

`PRAGMA`s al abrir: `foreign_keys = ON`, `journal_mode = WAL`, `busy_timeout = 5000`.
La versión del esquema se guarda en `PRAGMA user_version` para migraciones futuras.
La migración v3 puebla `funcionario` con los nombres distintos existentes en `reserva` y
enlaza `reserva.funcionario_id`.

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
curl -s -X POST localhost:8000/funcionarios -H 'content-type: application/json' -d '{"nombre":"María Pérez"}'
curl -s "localhost:8000/reservas?fecha=2026-10-01"
curl -s -X POST localhost:8000/reservas -H 'content-type: application/json' \
     -d '{"box_id":1,"fecha":"2026-10-01","bloque":4,"funcionario_id":1}'
curl -s -i -X DELETE localhost:8000/reservas/1
```
