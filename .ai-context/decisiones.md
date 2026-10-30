# Registro de decisiones (ADR corto)

Formato: `ADR-NNN · fecha · estado` — contexto → decisión → consecuencias.

## ADR-001 · 2026-09-30 · aceptada — API y app separadas
Contexto: el PC servidor también lo usa un funcionario. → Decisión: la API es un servicio
NSSM independiente; la app es un cliente más. → Consecuencia: cerrar la app no afecta al resto.

## ADR-002 · 2026-09-30 · aceptada — Sin autenticación
Datos no sensibles, solo LAN. → Ningún login. → Menos soporte; cualquiera en la LAN puede
reservar/cancelar (aceptado por el CESFAM).

## ADR-003 · 2026-09-30 · aceptada — Bloques fijos + UNIQUE
→ `UNIQUE(box_id, fecha, bloque)`; la API traduce la violación a 409. → Atomicidad garantizada
por SQLite; no existe "consultar y luego insertar".

## ADR-004 · 2026-09-30 · aceptada — Refresco periódico
3 usuarios. → Polling cada 20 s. → Hasta 20 s de desfase visible; el 409 cubre la carrera.

## ADR-005 · 2026-09-30 · aceptada — Descubrimiento en capas
→ guardada → `/health` → broadcast UDP 8001 → manual. → Sobrevive cambios de IP por DHCP.

## ADR-006 · 2026-10-01 · aceptada — `PATCH /boxes/{id}`
El `.docx` lista solo `GET/POST /boxes`, pero RF-06 exige renombrar y desactivar. → Se agrega
`PATCH`. Boxes nunca se borran (conservan historial).

## ADR-007 · 2026-10-01 · aceptada — Red de la app en Rust
→ Todo HTTP/UDP desde comandos Tauri (`reqwest`), frontend solo `invoke`. → Reutiliza `shared`,
UDP es posible, no depende de CORS/CSP. Se mantiene `CorsLayer` permisivo (RNF-07) por si se
consulta desde la webview o navegador para diagnóstico.

## ADR-008 · 2026-10-01 · aceptada — Frontend sin framework
→ TypeScript + Vite, DOM directo. → Cero dependencias que envejezcan; una sola pantalla no
justifica React/Vue. Contraparte: disciplina con `textContent` y componentes por función.

## ADR-009 · 2026-10-01 · aceptada — Crate `shared` y nombre `BoxAtencion`
→ Tipos en `shared/`. `Box` choca con `std::boxed::Box` → `BoxAtencion`. Tabla SQL sigue `box`.

## ADR-010 · 2026-10-01 · aceptada — Mutex<Connection> en vez de pool
≤ 3 usuarios. → Una conexión + `Mutex` + `spawn_blocking`. → Simplicidad; WAL + `busy_timeout`
por robustez.

## ADR-011 · 2026-10-01 · aceptada — Respaldo con `VACUUM INTO`
→ Al arrancar, antes de aceptar tráfico: `VACUUM INTO 'backups\gestion-box-AAAAMMDD-HHMMSS.db'`,
luego borrar los más antiguos sobre 30. → Copia consistente aun con WAL.

## ADR-012 · 2026-10-01 · aceptada — Log a archivo fijo `api.log`
RNF-10 exige ese nombre. → Appender no rotativo; al arrancar, si `api.log` > 10 MB se renombra a
`api.anterior.log`. → Tamaño acotado (~20 MB) sin perder el nombre exigido.

## ADR-013 · 2026-10-01 · aceptada — Carpeta de datos configurable
→ `GESTIONBOX_DATA_DIR` sobrescribe `C:\ProgramData\GestionBox`; fuera de Windows el defecto
es `./datos-dev`. → Desarrollo y tests en Linux sin tocar rutas de producción.

## ADR-014 · 2026-10-23 · aceptada — Grilla con boxes en filas y tramos fusionados
La planilla que reemplazamos piensa en medio día (BOX AM / BOX PM) y una mañana se anota en
una celda; la grilla anterior repetía el nombre en ~10 celdas. → Boxes en filas, bloques en
columnas, corte mañana/tarde (`HORA_INICIO_TARDE`, pendiente T-0.1) y bloques contiguos con
igual funcionario y motivo agrupados en una barra (`app/src/tramos.ts`). El modelo no cambia:
sigue una fila por bloque y el `UNIQUE` (ADR-003). → Cancelar desde la barra cancela cada
bloque del tramo.

## ADR-015 · 2026-10-23 · aceptada — Lotes de reservas con `grupo`
La planilla asigna un box por medio día y repite el patrón cada semana; reservar bloque por
bloque obligaba a decenas de clics. → `POST /reservas/lote` crea un rango de bloques en varias
fechas dentro de una transacción (todo o nada, u `omitir_ocupados`), y una columna nullable
`grupo` (migración v2) une lo creado junto. `DELETE /reservas/grupo/{grupo}?desde=` cancela
"este día y los siguientes". → Se mantiene una fila por bloque y el `UNIQUE` de ADR-003; no hay
"consultar y luego insertar". La app usa siempre el lote; `POST /reservas` queda por
compatibilidad.

## ADR-016 · 2026-10-23 · aceptada — Vista semanal, libres y buscador de funcionario
La planilla se leía por semana (AM/PM por día), con una hoja aparte de boxes libres, y
organizada por persona. → Selector Día | Semana: boxes × (lun–vie × AM/PM) con fila "Libres";
buscador que resalta un funcionario en ambas vistas; `GET /funcionarios` (nombres recientes)
para sugerir y no duplicar escrituras. → Sigue siendo una sola pantalla con dos modos; no se
agrega una tabla de funcionarios (el dato sigue siendo texto en la reserva).

## ADR-017 · 2026-10-30 · aceptada — Funcionarios registrados, etiquetas con color y búsqueda
Pedido del responsable: (1) que reservar no "cree" un funcionario escribiendo cualquier nombre,
sino que exista un apartado para registrarlos y al reservar se busque entre ellos; (2) etiquetas
con color (p. ej. "Enfermería", "Cirugía") asignables a boxes y a funcionarios; (3) un método de
búsqueda. Reemplaza la parte de ADR-016 que decía "no se agrega una tabla de funcionarios".
→ Migración v3: tablas `etiqueta (id, nombre UNIQUE NOCASE, color '#RRGGBB', activo)` y
`funcionario (id, nombre UNIQUE NOCASE, etiqueta_id NULL → etiqueta, activo)`; `box.etiqueta_id`
y `reserva.funcionario_id` (NULL → funcionario). La migración registra como funcionarios los
nombres distintos ya usados en `reserva.funcionario` y rellena `funcionario_id`. La columna de
texto `reserva.funcionario` se conserva (no se puede quitar el NOT NULL sin recrear la tabla) y se
sigue escribiendo con el nombre al crear; **al leer, el nombre sale del JOIN** con `funcionario`
para reflejar renombres. Las reservas se crean con `funcionario_id` (no con texto); un
funcionario inexistente → 404, inactivo → 422 `funcionario_inactivo`.
Una sola etiqueta por box y por funcionario (nullable): basta para "este box es de enfermería" y
"este funcionario es cirujano"; muchas-a-muchas se evalúa si en terreno hace falta.
Color: newtype `shared::Color` (`#RRGGBB`, parsear-no-validar). La app colorea la barra de la
reserva con la etiqueta del funcionario y el encabezado del box con la etiqueta del box, con
texto negro o blanco según contraste.
Búsqueda: `GET /reservas/buscar` (por funcionario, etiqueta y/o box en un rango de fechas) y
`GET /boxes/libres` (boxes activos libres en todo un rango horario de un día, filtrable por
etiqueta). → Siguen sin roles ni autenticación: cualquiera puede registrar funcionarios.
