//! SQL de la tabla `funcionario` (RF-13, RF-14, RNF-06).

use gestion_box_shared::Funcionario;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{es_violacion_unique, ErrorRepositorio};

const SQL_LISTAR: &str = "SELECT id, nombre, etiqueta_id, activo FROM funcionario
     WHERE (?1 OR activo = 1)
     ORDER BY nombre COLLATE NOCASE, id";
const SQL_OBTENER: &str = "SELECT id, nombre, etiqueta_id, activo FROM funcionario WHERE id = ?1";
const SQL_CREAR: &str = "INSERT INTO funcionario (nombre, etiqueta_id) VALUES (?1, ?2)";
const SQL_ACTUALIZAR: &str = "UPDATE funcionario
     SET nombre = COALESCE(?2, nombre),
         etiqueta_id = CASE WHEN ?3 THEN ?4 ELSE etiqueta_id END,
         activo = COALESCE(?5, activo)
     WHERE id = ?1";

fn fila_a_funcionario(fila: &Row<'_>) -> rusqlite::Result<Funcionario> {
    Ok(Funcionario {
        id: fila.get(0)?,
        nombre: fila.get(1)?,
        etiqueta_id: fila.get(2)?,
        activo: fila.get(3)?,
    })
}

fn mapear_unique(error: rusqlite::Error) -> ErrorRepositorio {
    if es_violacion_unique(&error) {
        ErrorRepositorio::NombreDuplicado
    } else {
        ErrorRepositorio::Sqlite(error)
    }
}

/// Lista los funcionarios ordenados alfabéticamente por nombre. Solo activos salvo que se pidan todos.
///
/// Cubre RF-13.
pub fn listar(
    conn: &Connection,
    incluir_inactivos: bool,
) -> Result<Vec<Funcionario>, ErrorRepositorio> {
    let mut sentencia = conn.prepare_cached(SQL_LISTAR)?;
    let funcionarios = sentencia
        .query_map(params![incluir_inactivos], fila_a_funcionario)?
        .collect::<Result<_, _>>()?;
    Ok(funcionarios)
}

/// Obtiene un funcionario por id.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] si no existe.
///
/// Cubre RF-13.
pub fn obtener(conn: &Connection, id: i64) -> Result<Funcionario, ErrorRepositorio> {
    conn.prepare_cached(SQL_OBTENER)?
        .query_row(params![id], fila_a_funcionario)
        .optional()?
        .ok_or(ErrorRepositorio::NoEncontrado)
}

/// Crea un funcionario activo con etiqueta opcional.
///
/// # Errores
/// [`ErrorRepositorio::NombreDuplicado`] si el nombre ya existe (sin distinguir mayúsculas).
///
/// Cubre RF-13 y RF-14.
pub fn crear(
    conn: &Connection,
    nombre: &str,
    etiqueta_id: Option<i64>,
) -> Result<Funcionario, ErrorRepositorio> {
    conn.prepare_cached(SQL_CREAR)?
        .execute(params![nombre, etiqueta_id])
        .map_err(mapear_unique)?;
    obtener(conn, conn.last_insert_rowid())
}

/// Actualiza nombre, etiqueta (`Some(None)` la quita, `None` no la toca) y/o estado activo.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] o [`ErrorRepositorio::NombreDuplicado`].
///
/// Cubre RF-13 y RF-14.
pub fn actualizar(
    conn: &Connection,
    id: i64,
    nombre: Option<&str>,
    etiqueta_id: Option<Option<i64>>,
    activo: Option<bool>,
) -> Result<Funcionario, ErrorRepositorio> {
    let cambiar_etiqueta = etiqueta_id.is_some();
    let nueva_etiqueta = etiqueta_id.flatten();
    let filas = conn
        .prepare_cached(SQL_ACTUALIZAR)?
        .execute(params![
            id,
            nombre,
            cambiar_etiqueta,
            nueva_etiqueta,
            activo
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
    fn crear_y_listar_en_orden_alfabetico() {
        let conn = db::abrir_en_memoria().unwrap();
        crear(&conn, "Verónica", None).unwrap();
        crear(&conn, "Ana", None).unwrap();

        let lista = listar(&conn, false).unwrap();
        let nombres: Vec<&str> = lista.iter().map(|f| f.nombre.as_str()).collect();
        assert_eq!(nombres, ["Ana", "Verónica"]);
    }

    #[test]
    fn asignar_y_quitar_etiqueta() {
        let conn = db::abrir_en_memoria().unwrap();
        let et = etiquetas::crear(&conn, "Enfermería", &"#2563eb".parse().unwrap()).unwrap();
        let f = crear(&conn, "Ana", Some(et.id)).unwrap();
        assert_eq!(f.etiqueta_id, Some(et.id));

        let renombrada = actualizar(&conn, f.id, Some("Ana Pérez"), None, None).unwrap();
        assert_eq!(renombrada.etiqueta_id, Some(et.id));

        let sin_etiqueta = actualizar(&conn, f.id, None, Some(None), None).unwrap();
        assert_eq!(sin_etiqueta.etiqueta_id, None);
    }

    #[test]
    fn nombre_duplicado_sin_distinguir_mayusculas() {
        let conn = db::abrir_en_memoria().unwrap();
        crear(&conn, "Ana", None).unwrap();

        let error = crear(&conn, "ANA", None).unwrap_err();

        assert!(matches!(error, ErrorRepositorio::NombreDuplicado));
    }
}
