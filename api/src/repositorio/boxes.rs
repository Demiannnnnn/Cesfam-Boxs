//! SQL de la tabla `box` (RF-06, RF-14).

use gestion_box_shared::BoxAtencion;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{es_violacion_unique, ErrorRepositorio};

const SQL_LISTAR: &str = "SELECT id, nombre, activo, orden, etiqueta_id FROM box
     WHERE (?1 OR activo = 1)
     ORDER BY orden, nombre COLLATE NOCASE";
const SQL_OBTENER: &str = "SELECT id, nombre, activo, orden, etiqueta_id FROM box WHERE id = ?1";
const SQL_CREAR: &str = "INSERT INTO box (nombre, orden, etiqueta_id)
     VALUES (?1, (SELECT COALESCE(MAX(orden), 0) + 1 FROM box), ?2)";
const SQL_ACTUALIZAR: &str = "UPDATE box
     SET nombre = COALESCE(?2, nombre),
         activo = COALESCE(?3, activo),
         etiqueta_id = CASE WHEN ?4 THEN ?5 ELSE etiqueta_id END
     WHERE id = ?1";

fn fila_a_box(fila: &Row<'_>) -> rusqlite::Result<BoxAtencion> {
    Ok(BoxAtencion {
        id: fila.get(0)?,
        nombre: fila.get(1)?,
        activo: fila.get(2)?,
        orden: fila.get(3)?,
        etiqueta_id: fila.get(4)?,
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

/// Crea un box activo al final del orden, con etiqueta opcional (RF-06, RF-14).
///
/// # Errores
/// [`ErrorRepositorio::NombreDuplicado`] si el nombre ya existe (sin distinguir mayúsculas).
pub fn crear(
    conn: &Connection,
    nombre: &str,
    etiqueta_id: Option<i64>,
) -> Result<BoxAtencion, ErrorRepositorio> {
    conn.prepare_cached(SQL_CREAR)?
        .execute(params![nombre, etiqueta_id])
        .map_err(mapear_unique)?;
    obtener(conn, conn.last_insert_rowid())
}

/// Actualiza nombre, estado activo y/o etiqueta (`Some(None)` la quita, `None` no la toca).
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] o [`ErrorRepositorio::NombreDuplicado`].
pub fn actualizar(
    conn: &Connection,
    id: i64,
    nombre: Option<&str>,
    activo: Option<bool>,
    etiqueta_id: Option<Option<i64>>,
) -> Result<BoxAtencion, ErrorRepositorio> {
    let cambiar_etiqueta = etiqueta_id.is_some();
    let nueva_etiqueta = etiqueta_id.flatten();
    let filas = conn
        .prepare_cached(SQL_ACTUALIZAR)?
        .execute(params![
            id,
            nombre,
            activo,
            cambiar_etiqueta,
            nueva_etiqueta
        ])
        .map_err(mapear_unique)?;
    if filas == 0 {
        return Err(ErrorRepositorio::NoEncontrado);
    }
    obtener(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repositorio::etiquetas};

    #[test]
    fn crear_asigna_orden_incremental() {
        let conn = db::abrir_en_memoria().unwrap();

        let a = crear(&conn, "Box 1", None).unwrap();
        let b = crear(&conn, "Box 2", None).unwrap();

        assert_eq!((a.orden, b.orden), (1, 2));
        assert!(a.activo);
        assert_eq!(a.etiqueta_id, None);
    }

    #[test]
    fn nombre_duplicado_sin_distinguir_mayusculas() {
        let conn = db::abrir_en_memoria().unwrap();
        crear(&conn, "Box 1", None).unwrap();

        let error = crear(&conn, "box 1", None).unwrap_err();

        assert!(matches!(error, ErrorRepositorio::NombreDuplicado));
    }

    #[test]
    fn listar_oculta_inactivos_salvo_que_se_pidan() {
        let conn = db::abrir_en_memoria().unwrap();
        let a = crear(&conn, "Box 1", None).unwrap();
        crear(&conn, "Box 2", None).unwrap();
        actualizar(&conn, a.id, None, Some(false), None).unwrap();

        assert_eq!(listar(&conn, false).unwrap().len(), 1);
        assert_eq!(listar(&conn, true).unwrap().len(), 2);
    }

    #[test]
    fn actualizar_parcial_conserva_campos_omitidos_y_permite_quitar_etiqueta() {
        let conn = db::abrir_en_memoria().unwrap();
        let et = etiquetas::crear(&conn, "Dental", &"#16a34a".parse().unwrap()).unwrap();
        let a = crear(&conn, "Box 1", Some(et.id)).unwrap();

        let cambiado = actualizar(&conn, a.id, Some("Box Dental"), None, None).unwrap();
        assert_eq!(cambiado.nombre, "Box Dental");
        assert!(cambiado.activo);
        assert_eq!(cambiado.etiqueta_id, Some(et.id));

        let sin_etiqueta = actualizar(&conn, a.id, None, None, Some(None)).unwrap();
        assert_eq!(sin_etiqueta.etiqueta_id, None);
    }

    #[test]
    fn actualizar_inexistente_es_no_encontrado() {
        let conn = db::abrir_en_memoria().unwrap();

        let error = actualizar(&conn, 99, None, Some(false), None).unwrap_err();

        assert!(matches!(error, ErrorRepositorio::NoEncontrado));
    }
}
