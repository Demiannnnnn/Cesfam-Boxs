# Tareas

Cada tarea es un archivo con: requisitos que cubre, dependencias, objetivo, pasos de
ejecución, patrones/buenas prácticas y **validaciones** que deben pasar para considerarla
cumplida. Plantilla: [`_plantilla.md`](_plantilla.md).

## Cómo ejecutar una tarea

1. Elige la primera tarea `pendiente` cuyas dependencias estén `completada`.
2. Crea la rama: `git switch -c dev/<fase>-<tema> develop` (nunca trabajes en `main`/`develop`).
3. Lee el archivo de la tarea y los enlaces de su sección *Contexto*.
4. Sigue los *Pasos de ejecución* en orden.
5. Ejecuta las *Validaciones* (todas) y los comandos de calidad:
   ```bash
   cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
   ```
6. Actualiza el estado en la tabla de abajo y en `.ai-context/estado.md`.
7. Commit con `Tareas: T-x.y` en el pie.

## Estados

`pendiente` · `en curso` · `completada` · `bloqueada` (requiere acción humana o en terreno)

## Definición de terminado (aplica a toda tarea de código)

- [ ] Todas las validaciones de la tarea pasan.
- [ ] `cargo fmt`, `clippy -D warnings` y `cargo test` pasan en el workspace.
- [ ] Ítems públicos documentados con `///` indicando el requisito que cubren.
- [ ] Sin `unwrap()`/`expect()` fuera de tests y arranque.
- [ ] Sin SQL concatenado.
- [ ] Textos visibles en español.
- [ ] Documentación afectada actualizada (`docs/`, `.ai-context/`).

## Tablero

### Fase 0 — Validación en terreno

| ID | Tarea | Cubre | Depende de | Estado |
|---|---|---|---|---|
| [T-0.1](fase-0/T-0.1-confirmar-horario.md) | Confirmar horario de atención y duración de bloques | RF-01 | — | bloqueada |
| [T-0.2](fase-0/T-0.2-lista-boxes-y-planilla.md) | Obtener lista de boxes y formato de la planilla Excel | RF-01, RF-06 | — | bloqueada |
| [T-0.3](fase-0/T-0.3-identificar-pc-servidor.md) | Identificar el PC servidor y su nombre de equipo | RNF-08 | — | bloqueada |
| [T-0.4](fase-0/T-0.4-probar-ping-por-nombre.md) | Probar `ping <nombre-pc>` desde otro equipo | RF-09 | T-0.3 | bloqueada |
| [T-0.5](fase-0/T-0.5-confirmar-subred.md) | Confirmar que todos los PCs están en la misma subred | RF-09 | — | bloqueada |
| [T-0.6](fase-0/T-0.6-reserva-dhcp.md) | Pedir a TI una reserva DHCP (opcional) | RNF-08 | T-0.3 | bloqueada |
| [T-0.7](fase-0/T-0.7-version-windows.md) | Revisar la versión de Windows de cada PC | RNF-13 | — | bloqueada |

### Fase 1 — Proyecto base

| ID | Tarea | Cubre | Depende de | Estado |
|---|---|---|---|---|
| [T-1.1](fase-1/T-1.1-workspace-cargo.md) | Crear el workspace de Cargo con `api/`, `app/` y `shared/` | RNF-05 | — | completada |
| [T-1.2](fase-1/T-1.2-tipos-compartidos.md) | Definir en `shared` los structs Box y Reserva con serde | — | T-1.1 | completada |
| [T-1.3](fase-1/T-1.3-repositorio-y-readme.md) | Repositorio y README con instrucciones de compilación | — | T-1.1 | completada |

### Fase 2 — API (Axum + rusqlite)

| ID | Tarea | Cubre | Depende de | Estado |
|---|---|---|---|---|
| [T-2.1](fase-2/T-2.1-esquema-sqlite.md) | Esquema SQLite con `UNIQUE(box_id, fecha, bloque)` | RF-05 | T-1.2 | completada |
| [T-2.2](fase-2/T-2.2-rutas-programdata.md) | Resolver rutas en `C:\ProgramData\GestionBox\` | RNF-04 | T-1.1 | completada |
| [T-2.3](fase-2/T-2.3-endpoints-boxes.md) | `GET/POST /boxes` (+ `PATCH /boxes/{id}`) | RF-06 | T-2.1 | completada |
| [T-2.4](fase-2/T-2.4-endpoints-reservas.md) | `GET/POST /reservas`, `DELETE /reservas/{id}` | RF-03, RF-04 | T-2.1 | completada |
| [T-2.5](fase-2/T-2.5-conflicto-409.md) | 409 Conflict al violar el UNIQUE | RF-05 | T-2.4 | completada |
| [T-2.6](fase-2/T-2.6-health.md) | `GET /health` | RF-08 | T-1.2 | completada |
| [T-2.7](fase-2/T-2.7-cors.md) | `CorsLayer` permisivo | RNF-07 | T-2.6 | completada |
| [T-2.8](fase-2/T-2.8-listener-udp.md) | Listener UDP 8001 en hilo aparte | RF-09, RNF-03 | T-1.2 | completada |
| [T-2.9](fase-2/T-2.9-respaldo-rotacion.md) | Respaldo al iniciar y rotación a 30 | RF-11 | T-2.1, T-2.2 | completada |
| [T-2.10](fase-2/T-2.10-tracing-log.md) | `tracing` hacia `api.log` | RNF-10 | T-2.2 | completada |
| [T-2.11](fase-2/T-2.11-probar-endpoints.md) | Probar todos los endpoints (tests + curl) | — | T-2.3 – T-2.10 | completada |

### Fase 3 — App Tauri (cliente)

| ID | Tarea | Cubre | Depende de | Estado |
|---|---|---|---|---|
| [T-3.1](fase-3/T-3.1-logica-conexion.md) | Lógica de conexión en capas | RF-09, RF-10 | T-2.8 | pendiente |
| [T-3.2](fase-3/T-3.2-pantalla-conexion.md) | Pantalla de primera configuración / conexión manual | RF-09 | T-3.1 | pendiente |
| [T-3.3](fase-3/T-3.3-grilla-navegacion.md) | Grilla de boxes por bloques con navegación de fechas | RF-01, RF-02 | T-3.1 | pendiente |
| [T-3.4](fase-3/T-3.4-reservar-y-cancelar.md) | Formulario de reserva y detalle con cancelación | RF-03, RF-04 | T-3.3 | pendiente |
| [T-3.5](fase-3/T-3.5-manejo-409.md) | Manejo del 409 con su mensaje | RF-05 | T-3.4 | pendiente |
| [T-3.6](fase-3/T-3.6-modal-boxes.md) | Modal de configuración de boxes | RF-06 | T-3.3 | pendiente |
| [T-3.7](fase-3/T-3.7-refresco-automatico.md) | Refresco automático cada 15–30 s | RF-07 | T-3.3 | pendiente |
| [T-3.8](fase-3/T-3.8-indicador-conexion.md) | Indicador de estado de conexión | RF-08 | T-3.1 | pendiente |
| [T-3.9](fase-3/T-3.9-mensajes-error.md) | Mensajes de error en español por caso de red | RNF-09 | T-3.1 | pendiente |
| [T-3.10](fase-3/T-3.10-exportar.md) | Exportar a CSV (opcional) | RF-12 | T-3.3 | pendiente |

### Fase 4 — Instalación

| ID | Tarea | Cubre | Depende de | Estado |
|---|---|---|---|---|
| [T-4.1](fase-4/T-4.1-release-sin-consola.md) | API release con `windows_subsystem = "windows"` | RNF-02 | T-2.11 | pendiente |
| [T-4.2](fase-4/T-4.2-instalar-bat.md) | `instalar.bat`: carpeta, NSSM, firewall | RNF-01, RNF-03, RNF-12 | T-4.1 | pendiente |
| [T-4.3](fase-4/T-4.3-desinstalar-bat.md) | `desinstalar.bat` (opcional) | — | T-4.2 | pendiente |
| [T-4.4](fase-4/T-4.4-instalador-app.md) | Instalador de la app Tauri (.msi) | RNF-13 | T-3.9 | pendiente |

### Fase 5 — Pruebas y entrega

| ID | Tarea | Cubre | Depende de | Estado |
|---|---|---|---|---|
| [T-5.1](fase-5/T-5.1-pruebas-aceptacion.md) | Ejecutar las pruebas de aceptación | Todos | T-4.2, T-4.4 | bloqueada |
| [T-5.2](fase-5/T-5.2-manual-funcionario.md) | Manual de una página para funcionarios | RNF-11 | T-3.9 | pendiente |
| [T-5.3](fase-5/T-5.3-guia-instalacion.md) | Guía de instalación y solución de problemas | RNF-12 | T-4.2 | pendiente |
| [T-5.4](fase-5/T-5.4-instalacion-demo.md) | Instalar en el CESFAM y demostración | — | T-5.1 | bloqueada |
| [T-5.5](fase-5/T-5.5-acta-entrega.md) | Firmar el acta de entrega | — | T-5.4 | bloqueada |
