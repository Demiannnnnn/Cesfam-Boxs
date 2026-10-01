# Convenciones de código

Complementa `AGENTS.md` §6 con reglas concretas y ejemplos.

## Rust

### Errores por capa

```rust
// repositorio: errores de almacenamiento, sin HTTP
#[derive(Debug, thiserror::Error)]
pub enum ErrorRepositorio {
    #[error("el bloque ya está reservado")]
    BloqueOcupado,
    #[error("nombre duplicado")]
    NombreDuplicado,
    #[error("no encontrado")]
    NoEncontrado,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

// http: único lugar que decide status + cuerpo
impl IntoResponse for ErrorApi { /* match → (StatusCode, Json(ErrorRespuesta)) */ }
```

Detectar `UNIQUE` con el código extendido de SQLite, no con el texto del mensaje:

```rust
Err(rusqlite::Error::SqliteFailure(e, _))
    if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE => ErrorRepositorio::BloqueOcupado
```

### SQL

- Constantes `const SQL_INSERTAR_RESERVA: &str = "INSERT ... VALUES (?1, ?2, ?3, ?4, ?5, ?6)";`
- `params![...]` siempre. Nunca `format!`/concatenación con datos.
- Columnas explícitas en `SELECT` (nunca `SELECT *`), mapeo en una función `fila_a_reserva(&Row)`.

### Concurrencia

- `AppState { db: Arc<Mutex<Connection>>, info: Arc<InfoServidor> }`, `Clone`.
- Acceso a DB dentro de `tokio::task::spawn_blocking`, mediante un helper
  `con_db(&estado, |conn| ...)` para no repetir el boilerplate.
- No sostener el `MutexGuard` a través de un `.await`.

### Logging

- `tracing::info!` para eventos de negocio (reserva creada/cancelada, box cambiado, arranque, respaldo).
- `tracing::warn!` para entradas rechazadas relevantes (409), `error!` para fallas internas.
- Campos estructurados: `info!(box_id, %fecha, bloque, "reserva creada")`.
- Nunca loguear el cuerpo completo de peticiones.

### Documentación

```rust
/// Crea una reserva.
///
/// # Errores
/// - [`ErrorServicio::BloqueOcupado`] si ya existe una reserva para ese box, fecha y bloque.
///
/// Cubre RF-03 y RF-05.
pub fn crear_reserva(...)
```

### Tests

- Nombres descriptivos en español: `fn rechaza_reserva_duplicada_con_409()`.
- Esquema AAA (preparar / actuar / verificar) con líneas en blanco entre fases.
- API: `api/tests/*.rs` usan `crear_router(estado_de_prueba())` + `tower::ServiceExt::oneshot`.
- DB de prueba: `Connection::open_in_memory()` + `db::migrar`.

## TypeScript

- `tsconfig` `strict`, `noUncheckedIndexedAccess`.
- Un componente = una función `renderizarX(contenedor, datos, acciones)` que reconstruye su DOM.
  Sin estado oculto en el DOM: la fuente es `estado.ts`.
- Crear nodos con `document.createElement` + `textContent`. **Nunca** `innerHTML` con datos
  (evita inyección con nombres de funcionarios).
- Toda llamada al backend pasa por `api.ts`; errores llegan como `{ codigo, mensaje }`.
- Textos visibles: solo desde `textos.ts`.

## Commits

`feat(api): implementar POST /reservas con 409 en conflicto` + pie `Tareas: T-2.4, T-2.5`.
