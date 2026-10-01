//! Acceso a datos. Único lugar con SQL; siempre parametrizado (RNF-06).

pub mod boxes;
pub mod reservas;

use rusqlite::ffi;

/// Error de la capa de almacenamiento, sin conocimiento de HTTP.
#[derive(Debug, thiserror::Error)]
pub enum ErrorRepositorio {
    #[error("el box, fecha y bloque ya están reservados")]
    BloqueOcupado,
    #[error("ya existe un box con ese nombre")]
    NombreDuplicado,
    #[error("registro no encontrado")]
    NoEncontrado,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// `true` si el error es una violación de `UNIQUE`. Se compara el código
/// extendido de SQLite, no el texto del mensaje (que cambia entre versiones).
pub(crate) fn es_violacion_unique(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(e, _) if e.extended_code == ffi::SQLITE_CONSTRAINT_UNIQUE
    )
}
