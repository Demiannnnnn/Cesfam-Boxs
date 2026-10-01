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
