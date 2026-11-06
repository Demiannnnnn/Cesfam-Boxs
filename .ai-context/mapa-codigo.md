# Mapa del código

Dónde vive cada responsabilidad. Actualizar al crear o mover módulos.

| Responsabilidad | Ubicación | Requisitos |
|---|---|---|
| Tipos del dominio y DTOs | `shared/src/modelo.rs` | — |
| Horario (bloques ↔ hora) | `shared/src/horario.rs` | RF-01 |
| `Fecha`, `Bloque`, `Color` validados | `shared/src/tipos.rs` | RF-03, RF-14 |
| Cuerpo de error y códigos | `shared/src/error.rs` | RF-05, RNF-09 |
| Protocolo UDP (constantes) | `shared/src/descubrimiento.rs` | RF-09 |
| Config API (rutas, puertos) | `api/src/config.rs` | RNF-03, RNF-04 |
| Esquema y migraciones | `api/src/db.rs` | RF-05, RF-13, RF-14, RNF-06 |
| SQL de boxes | `api/src/repositorio/boxes.rs` | RF-06, RF-14, RF-15, RNF-06 |
| SQL de etiquetas | `api/src/repositorio/etiquetas.rs` | RF-14, RNF-06 |
| SQL de funcionarios | `api/src/repositorio/funcionarios.rs` | RF-13, RF-14, RNF-06 |
| SQL de reservas | `api/src/repositorio/reservas.rs` | RF-03–05, RF-13, RF-15, RNF-06 |
| Reglas de negocio (errores, catálogos, reservas, búsqueda) | `api/src/servicio/{mod,catalogos,reservas,busqueda}.rs` | RF-03–06, RF-12–15 |
| Router, CORS | `api/src/http/mod.rs` | RNF-07 |
| Handlers (incluye GET /respaldo) | `api/src/http/handlers.rs` | RF-03–08, RF-13–15, RF-18 |
| Error → respuesta HTTP | `api/src/http/error.rs` | RF-05 |
| Listener UDP | `api/src/descubrimiento.rs` | RF-09, RNF-03 |
| Respaldo, rotación y entrega de copias (GET /respaldo) | `api/src/respaldo.rs` | RF-11, RF-18 |
| Log a archivo | `api/src/log.rs` | RNF-10 |
| Arranque | `api/src/main.rs`, `api/src/lib.rs` | RNF-01, RNF-02 |
| Comandos Tauri | `app/src-tauri/src/comandos.rs` | RF-03–08, RF-12–15 |
| Cliente HTTP | `app/src-tauri/src/cliente_api.rs` | RF-03–08, RF-13–15 |
| Descubrimiento (cadena) | `app/src-tauri/src/conexion.rs` | RF-09, RNF-08 |
| Config local (modo del equipo y dirección) | `app/src-tauri/src/config_local.rs` | RF-10, RF-13, RF-16, RF-17 |
| Instalación local del servicio desde la app | `app/src-tauri/src/servidor_local.rs` | RF-16, RNF-12, RNF-14 |
| Copia diaria y rotación en equipos de escritura | `app/src-tauri/src/copia_diaria/` | RF-18 |
| Opciones avanzadas y promoción a servidor | `app/src-tauri/src/avanzado.rs` | RF-19 |
| Errores → español | `app/src-tauri/src/errores.rs` | RNF-09 |
| Exportar CSV | `app/src-tauri/src/exportar.rs` | RF-12, RF-14 |
| Asistente de primer inicio (elección de modo) | `app/src/componentes/asistente.ts` | RF-16, RF-17 |
| Modal de opciones avanzadas | `app/src/componentes/avanzado.ts` | RF-19 |
| Grilla y leyenda de etiquetas | `app/src/componentes/grilla.ts` | RF-01, RF-02, RF-14, RF-15 |
| Vista semanal (AM/PM, libres, colores) | `app/src/componentes/semana.ts` | RF-01, RF-02, RF-14, RF-15 |
| Agrupar bloques en tramos | `app/src/tramos.ts` | RF-01, RF-15 |
| Colores y contraste WCAG | `app/src/colores.ts` | RF-14 |
| Modal de búsqueda (reservas y boxes libres) | `app/src/componentes/buscar.ts` | RF-15 |
| Modal de reserva (rango, serie, funcionario registrado) | `app/src/componentes/reservar.ts` | RF-03, RF-05, RF-13, RF-14 |
| Modal de funcionarios | `app/src/componentes/funcionarios.ts` | RF-13, RF-14 |
| Modal de etiquetas | `app/src/componentes/etiquetas.ts` | RF-14 |
| Modales (detalle, boxes, conexión, exportar) | `app/src/componentes/modales.ts` | RF-04, RF-06, RF-09, RF-12, RF-14 |
| Piezas de formulario (chips, selectores de etiqueta, color y rango horario) | `app/src/componentes/formulario.ts` | RF-03, RF-13–15 |
| Diálogo base (pausa refresco) | `app/src/componentes/dialogo.ts` | RF-07 |
| Aviso temporal | `app/src/componentes/aviso.ts` | RF-05 |
| Fechas (sin desfase UTC) | `app/src/fechas.ts` | RF-02 |
| Creación segura de DOM | `app/src/dom.ts` | — |
| Envoltorios de invoke | `app/src/api.ts` | — |
| Estado de pantalla y filtros rápidos | `app/src/estado.ts` | RF-13–15 |
| Indicador conexión | `app/src/componentes/conexion.ts` | RF-08 |
| Ciclo de refresco y atajo `Ctrl+F` | `app/src/main.ts` | RF-07, RF-13–15 |
| Textos | `app/src/textos.ts` | RNF-09, RNF-11 |
| Instalador servidor | `instalador/instalar.bat` | RNF-01, RNF-03, RNF-12 |
