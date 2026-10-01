# Mapa del código

Dónde vive cada responsabilidad. Actualizar al crear o mover módulos.

| Responsabilidad | Ubicación | Requisitos |
|---|---|---|
| Tipos del dominio y DTOs | `shared/src/modelo.rs` | — |
| Horario (bloques ↔ hora) | `shared/src/horario.rs` | RF-01 |
| `Fecha`, `Bloque` validados | `shared/src/tipos.rs` | RF-03 |
| Cuerpo de error y códigos | `shared/src/error.rs` | RF-05, RNF-09 |
| Protocolo UDP (constantes) | `shared/src/descubrimiento.rs` | RF-09 |
| Config API (rutas, puertos) | `api/src/config.rs` | RNF-03, RNF-04 |
| Esquema y migraciones | `api/src/db.rs` | RF-05, RNF-06 |
| SQL de boxes | `api/src/repositorio/boxes.rs` | RF-06, RNF-06 |
| SQL de reservas | `api/src/repositorio/reservas.rs` | RF-03–05, RNF-06 |
| Reglas de negocio | `api/src/servicio.rs` | RF-03–06 |
| Router, CORS | `api/src/http/mod.rs` | RNF-07 |
| Handlers | `api/src/http/handlers.rs` | RF-03–08 |
| Error → respuesta HTTP | `api/src/http/error.rs` | RF-05 |
| Listener UDP | `api/src/descubrimiento.rs` | RF-09, RNF-03 |
| Respaldo y rotación | `api/src/respaldo.rs` | RF-11 |
| Log a archivo | `api/src/log.rs` | RNF-10 |
| Arranque | `api/src/main.rs`, `api/src/lib.rs` | RNF-01, RNF-02 |
| Comandos Tauri | `app/src-tauri/src/comandos.rs` | — |
| Cliente HTTP | `app/src-tauri/src/cliente_api.rs` | RF-03–08 |
| Descubrimiento (cadena) | `app/src-tauri/src/conexion.rs` | RF-09, RNF-08 |
| Config local | `app/src-tauri/src/config_local.rs` | RF-10 |
| Errores → español | `app/src-tauri/src/errores.rs` | RNF-09 |
| Exportar CSV | `app/src-tauri/src/exportar.rs` | RF-12 |
| Grilla | `app/src/componentes/grilla.ts` | RF-01, RF-02 |
| Modales | `app/src/componentes/modales.ts` | RF-03, RF-04, RF-06, RF-09, RF-12 |
| Diálogo base (pausa refresco) | `app/src/componentes/dialogo.ts` | RF-07 |
| Aviso temporal | `app/src/componentes/aviso.ts` | RF-05 |
| Fechas (sin desfase UTC) | `app/src/fechas.ts` | RF-02 |
| Creación segura de DOM | `app/src/dom.ts` | — |
| Envoltorios de invoke | `app/src/api.ts` | — |
| Indicador conexión | `app/src/componentes/conexion.ts` | RF-08 |
| Ciclo de refresco | `app/src/main.ts` | RF-07 |
| Textos | `app/src/textos.ts` | RNF-09, RNF-11 |
| Instalador servidor | `instalador/instalar.bat` | RNF-01, RNF-03, RNF-12 |
