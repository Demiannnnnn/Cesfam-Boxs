//! SQL de la tabla `etiqueta` (RF-14, RNF-06).

use gestion_box_shared::{Color, Etiqueta};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{es_violacion_unique, ErrorRepositorio};

const SQL_LISTAR: &str = "SELECT id, nombre, color, activo FROM etiqueta
     WHERE (?1 OR activo = 1)
     ORDER BY nombre COLLATE NOCASE, id";
const SQL_OBTENER: &str = "SELECT id, nombre, color, activo FROM etiqueta WHERE id = ?1";
const SQL_CREAR: &str = "INSERT INTO etiqueta (nombre, color) VALUES (?1, ?2)";
const SQL_ACTUALIZAR: &str = "UPDATE etiqueta
     SET nombre = COALESCE(?2, nombre),
         color = COALESCE(?3, color),
         activo = COALESCE(?4, activo)
     WHERE id = ?1";

fn fila_a_etiqueta(fila: &Row<'_>) -> rusqlite::Result<Etiqueta> {
    let color: String = fila.get(2)?;
    Ok(Etiqueta {
        id: fila.get(0)?,
        nombre: fila.get(1)?,
        color: color
            .parse()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(e)))?,
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

/// Lista las etiquetas ordenadas alfabéticamente por nombre. Solo activas salvo que se pidan todas.
///
/// Cubre RF-14.
pub fn listar(
    conn: &Connection,
    incluir_inactivos: bool,
) -> Result<Vec<Etiqueta>, ErrorRepositorio> {
    let mut sentencia = conn.prepare_cached(SQL_LISTAR)?;
    let etiquetas = sentencia
        .query_map(params![incluir_inactivos], fila_a_etiqueta)?
        .collect::<Result<_, _>>()?;
    Ok(etiquetas)
}

/// Obtiene una etiqueta por id.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] si no existe.
///
/// Cubre RF-14.
pub fn obtener(conn: &Connection, id: i64) -> Result<Etiqueta, ErrorRepositorio> {
    conn.prepare_cached(SQL_OBTENER)?
        .query_row(params![id], fila_a_etiqueta)
        .optional()?
        .ok_or(ErrorRepositorio::NoEncontrado)
}

/// Crea una etiqueta activa.
///
/// # Errores
/// [`ErrorRepositorio::NombreDuplicado`] si el nombre ya existe (sin distinguir mayúsculas).
///
/// Cubre RF-14.
pub fn crear(conn: &Connection, nombre: &str, color: &Color) -> Result<Etiqueta, ErrorRepositorio> {
    conn.prepare_cached(SQL_CREAR)?
        .execute(params![nombre, color.as_str()])
        .map_err(mapear_unique)?;
    obtener(conn, conn.last_insert_rowid())
}

/// Actualiza nombre, color y/o estado activo de una etiqueta. `None` deja el campo como está.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] o [`ErrorRepositorio::NombreDuplicado`].
///
/// Cubre RF-14.
pub fn actualizar(
    conn: &Connection,
    id: i64,
    nombre: Option<&str>,
    color: Option<&Color>,
    activo: Option<bool>,
) -> Result<Etiqueta, ErrorRepositorio> {
    let filas = conn
        .prepare_cached(SQL_ACTUALIZAR)?
        .execute(params![id, nombre, color.map(Color::as_str), activo])
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

    fn color(hex: &str) -> Color {
        hex.parse().unwrap()
    }

    #[test]
    fn crear_y_listar_en_orden_alfabetico() {
        let conn = db::abrir_en_memoria().unwrap();

        crear(&conn, "Enfermería", &color("#2563eb")).unwrap();
        crear(&conn, "Cirugía", &color("#dc2626")).unwrap();

        let lista = listar(&conn, false).unwrap();
        let nombres: Vec<&str> = lista.iter().map(|e| e.nombre.as_str()).collect();
        assert_eq!(nombres, ["Cirugía", "Enfermería"]);
        assert!(lista[0].activo);
    }

    #[test]
    fn nombre_duplicado_sin_distinguir_mayusculas() {
        let conn = db::abrir_en_memoria().unwrap();
        crear(&conn, "Cirugía", &color("#dc2626")).unwrap();

        let error = crear(&conn, "cirugía", &color("#2563eb")).unwrap_err();

        assert!(matches!(error, ErrorRepositorio::NombreDuplicado));
    }

    #[test]
    fn listar_oculta_inactivas_salvo_que_se_pidan() {
        let conn = db::abrir_en_memoria().unwrap();
        let a = crear(&conn, "Cirugía", &color("#dc2626")).unwrap();
        crear(&conn, "Enfermería", &color("#2563eb")).unwrap();
        actualizar(&conn, a.id, None, None, Some(false)).unwrap();

        assert_eq!(listar(&conn, false).unwrap().len(), 1);
        assert_eq!(listar(&conn, true).unwrap().len(), 2);
    }

    #[test]
    fn actualizar_parcial_conserva_campos_omitidos() {
        let conn = db::abrir_en_memoria().unwrap();
        let a = crear(&conn, "Cirugía", &color("#dc2626")).unwrap();

        let cambiada = actualizar(&conn, a.id, None, Some(&color("#16a34a")), None).unwrap();

        assert_eq!(cambiada.nombre, "Cirugía");
        assert_eq!(cambiada.color, color("#16a34a"));
        assert!(cambiada.activo);
    }
}
