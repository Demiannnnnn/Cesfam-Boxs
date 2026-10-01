//! SQL de la tabla `box` (RF-06).

use gestion_box_shared::BoxAtencion;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{es_violacion_unique, ErrorRepositorio};

const SQL_LISTAR: &str = "SELECT id, nombre, activo, orden FROM box
     WHERE (?1 OR activo = 1)
     ORDER BY orden, nombre COLLATE NOCASE";
const SQL_OBTENER: &str = "SELECT id, nombre, activo, orden FROM box WHERE id = ?1";
const SQL_CREAR: &str = "INSERT INTO box (nombre, orden)
     VALUES (?1, (SELECT COALESCE(MAX(orden), 0) + 1 FROM box))";
const SQL_ACTUALIZAR: &str = "UPDATE box
     SET nombre = COALESCE(?2, nombre), activo = COALESCE(?3, activo)
     WHERE id = ?1";

fn fila_a_box(fila: &Row<'_>) -> rusqlite::Result<BoxAtencion> {
    Ok(BoxAtencion {
        id: fila.get(0)?,
        nombre: fila.get(1)?,
        activo: fila.get(2)?,
        orden: fila.get(3)?,
    })
}

fn mapear_unique(error: rusqlite::Error) -> ErrorRepositorio {
    if es_violacion_unique(&error) {
        ErrorRepositorio::NombreDuplicado
    } else {
        ErrorRepositorio::Sqlite(error)
    }
}

/// Lista los boxes ordenados por `orden` y nombre. Solo activos salvo que se pidan todos.
pub fn listar(
    conn: &Connection,
    incluir_inactivos: bool,
) -> Result<Vec<BoxAtencion>, ErrorRepositorio> {
    let mut sentencia = conn.prepare_cached(SQL_LISTAR)?;
    let boxes = sentencia
        .query_map(params![incluir_inactivos], fila_a_box)?
        .collect::<Result<_, _>>()?;
    Ok(boxes)
}

/// Obtiene un box por id.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] si no existe.
pub fn obtener(conn: &Connection, id: i64) -> Result<BoxAtencion, ErrorRepositorio> {
    conn.prepare_cached(SQL_OBTENER)?
        .query_row(params![id], fila_a_box)
        .optional()?
        .ok_or(ErrorRepositorio::NoEncontrado)
}

/// Crea un box activo al final del orden.
///
/// # Errores
/// [`ErrorRepositorio::NombreDuplicado`] si el nombre ya existe (sin distinguir mayúsculas).
pub fn crear(conn: &Connection, nombre: &str) -> Result<BoxAtencion, ErrorRepositorio> {
    conn.prepare_cached(SQL_CREAR)?
        .execute(params![nombre])
        .map_err(mapear_unique)?;
    obtener(conn, conn.last_insert_rowid())
}

/// Actualiza nombre y/o estado activo. `None` deja el campo como está.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] o [`ErrorRepositorio::NombreDuplicado`].
pub fn actualizar(
    conn: &Connection,
    id: i64,
    nombre: Option<&str>,
    activo: Option<bool>,
) -> Result<BoxAtencion, ErrorRepositorio> {
    let filas = conn
        .prepare_cached(SQL_ACTUALIZAR)?
        .execute(params![id, nombre, activo])
        .map_err(mapear_unique)?;
    if filas == 0 {
        return Err(ErrorRepositorio::NoEncontrado);
    }
    obtener(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn crear_asigna_orden_incremental() {
        let conn = db::abrir_en_memoria().unwrap();

        let a = crear(&conn, "Box 1").unwrap();
        let b = crear(&conn, "Box 2").unwrap();

        assert_eq!((a.orden, b.orden), (1, 2));
        assert!(a.activo);
    }

    #[test]
    fn nombre_duplicado_sin_distinguir_mayusculas() {
        let conn = db::abrir_en_memoria().unwrap();
        crear(&conn, "Box 1").unwrap();

        let error = crear(&conn, "box 1").unwrap_err();

        assert!(matches!(error, ErrorRepositorio::NombreDuplicado));
    }

    #[test]
    fn listar_oculta_inactivos_salvo_que_se_pidan() {
        let conn = db::abrir_en_memoria().unwrap();
        let a = crear(&conn, "Box 1").unwrap();
        crear(&conn, "Box 2").unwrap();
        actualizar(&conn, a.id, None, Some(false)).unwrap();

        assert_eq!(listar(&conn, false).unwrap().len(), 1);
        assert_eq!(listar(&conn, true).unwrap().len(), 2);
    }

    #[test]
    fn actualizar_parcial_conserva_campos_omitidos() {
        let conn = db::abrir_en_memoria().unwrap();
        let a = crear(&conn, "Box 1").unwrap();

        let cambiado = actualizar(&conn, a.id, Some("Box Dental"), None).unwrap();

        assert_eq!(cambiado.nombre, "Box Dental");
        assert!(cambiado.activo);
    }

    #[test]
    fn actualizar_inexistente_es_no_encontrado() {
        let conn = db::abrir_en_memoria().unwrap();

        let error = actualizar(&conn, 99, None, Some(false)).unwrap_err();

        assert!(matches!(error, ErrorRepositorio::NoEncontrado));
    }
}
