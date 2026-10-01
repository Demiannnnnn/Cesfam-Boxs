//! SQL de la tabla `reserva` (RF-03, RF-04, RF-05).

use gestion_box_shared::{Bloque, Fecha, Reserva};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{es_violacion_unique, ErrorRepositorio};

const SQL_LISTAR_RANGO: &str =
    "SELECT id, box_id, fecha, bloque, funcionario, motivo, creada_en FROM reserva
     WHERE fecha BETWEEN ?1 AND ?2
     ORDER BY fecha, bloque, box_id";
const SQL_OBTENER: &str =
    "SELECT id, box_id, fecha, bloque, funcionario, motivo, creada_en FROM reserva WHERE id = ?1";
const SQL_CREAR: &str =
    "INSERT INTO reserva (box_id, fecha, bloque, funcionario, motivo, creada_en)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
const SQL_ELIMINAR: &str = "DELETE FROM reserva WHERE id = ?1";

/// Datos de una reserva ya validados por la capa de servicio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservaValida {
    pub box_id: i64,
    pub fecha: Fecha,
    pub bloque: Bloque,
    pub funcionario: String,
    pub motivo: Option<String>,
    pub creada_en: String,
}

fn fila_a_reserva(fila: &Row<'_>) -> rusqlite::Result<Reserva> {
    let fecha: String = fila.get(2)?;
    let bloque: u8 = fila.get(3)?;
    Ok(Reserva {
        id: fila.get(0)?,
        box_id: fila.get(1)?,
        fecha: fecha
            .parse()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(e)))?,
        bloque: Bloque::new(bloque).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, Type::Integer, Box::new(e))
        })?,
        funcionario: fila.get(4)?,
        motivo: fila.get(5)?,
        creada_en: fila.get(6)?,
    })
}

/// Reservas con fecha entre `desde` y `hasta` (ambas inclusive),
/// ordenadas por fecha, bloque y box.
pub fn listar_por_rango(
    conn: &Connection,
    desde: Fecha,
    hasta: Fecha,
) -> Result<Vec<Reserva>, ErrorRepositorio> {
    let mut sentencia = conn.prepare_cached(SQL_LISTAR_RANGO)?;
    let reservas = sentencia
        .query_map(
            params![desde.to_string(), hasta.to_string()],
            fila_a_reserva,
        )?
        .collect::<Result<_, _>>()?;
    Ok(reservas)
}

/// Obtiene una reserva por id.
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] si no existe.
pub fn obtener(conn: &Connection, id: i64) -> Result<Reserva, ErrorRepositorio> {
    conn.prepare_cached(SQL_OBTENER)?
        .query_row(params![id], fila_a_reserva)
        .optional()?
        .ok_or(ErrorRepositorio::NoEncontrado)
}

/// Inserta una reserva. La exclusividad la garantiza `UNIQUE(box_id, fecha, bloque)`:
/// nunca se consulta antes de insertar, para evitar condiciones de carrera.
///
/// # Errores
/// [`ErrorRepositorio::BloqueOcupado`] si el box ya está reservado en ese bloque (RF-05).
pub fn crear(conn: &Connection, nueva: &ReservaValida) -> Result<Reserva, ErrorRepositorio> {
    conn.prepare_cached(SQL_CREAR)?
        .execute(params![
            nueva.box_id,
            nueva.fecha.to_string(),
            nueva.bloque.valor(),
            nueva.funcionario,
            nueva.motivo,
            nueva.creada_en,
        ])
        .map_err(|error| {
            if es_violacion_unique(&error) {
                ErrorRepositorio::BloqueOcupado
            } else {
                ErrorRepositorio::Sqlite(error)
            }
        })?;
    obtener(conn, conn.last_insert_rowid())
}

/// Elimina una reserva (cancelación).
///
/// # Errores
/// [`ErrorRepositorio::NoEncontrado`] si ya no existe.
pub fn eliminar(conn: &Connection, id: i64) -> Result<(), ErrorRepositorio> {
    let filas = conn.prepare_cached(SQL_ELIMINAR)?.execute(params![id])?;
    if filas == 0 {
        Err(ErrorRepositorio::NoEncontrado)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repositorio::boxes};

    fn reserva(box_id: i64, fecha: &str, bloque: u8) -> ReservaValida {
        ReservaValida {
            box_id,
            fecha: fecha.parse().unwrap(),
            bloque: Bloque::new(bloque).unwrap(),
            funcionario: "Ana".into(),
            motivo: None,
            creada_en: "2026-09-30T10:00:00".into(),
        }
    }

    fn preparar() -> Connection {
        let conn = db::abrir_en_memoria().unwrap();
        boxes::crear(&conn, "Box 1").unwrap();
        boxes::crear(&conn, "Box 2").unwrap();
        conn
    }

    #[test]
    fn crear_y_obtener() {
        let conn = preparar();

        let creada = crear(&conn, &reserva(1, "2026-10-01", 4)).unwrap();

        assert_eq!(obtener(&conn, creada.id).unwrap(), creada);
        assert_eq!(creada.funcionario, "Ana");
    }

    #[test]
    fn mismo_bloque_en_otro_box_esta_permitido() {
        let conn = preparar();
        crear(&conn, &reserva(1, "2026-10-01", 4)).unwrap();

        assert!(crear(&conn, &reserva(2, "2026-10-01", 4)).is_ok());
    }

    #[test]
    fn mismo_box_fecha_y_bloque_es_bloque_ocupado() {
        let conn = preparar();
        crear(&conn, &reserva(1, "2026-10-01", 4)).unwrap();

        let error = crear(&conn, &reserva(1, "2026-10-01", 4)).unwrap_err();

        assert!(matches!(error, ErrorRepositorio::BloqueOcupado));
    }

    #[test]
    fn rango_incluye_extremos_y_ordena() {
        let conn = preparar();
        crear(&conn, &reserva(2, "2026-10-31", 0)).unwrap();
        crear(&conn, &reserva(1, "2026-10-01", 5)).unwrap();
        crear(&conn, &reserva(1, "2026-11-01", 0)).unwrap();

        let lista = listar_por_rango(
            &conn,
            "2026-10-01".parse().unwrap(),
            "2026-10-31".parse().unwrap(),
        )
        .unwrap();

        let fechas: Vec<String> = lista.iter().map(|r| r.fecha.to_string()).collect();
        assert_eq!(fechas, ["2026-10-01", "2026-10-31"]);
    }

    #[test]
    fn eliminar_dos_veces_es_no_encontrado() {
        let conn = preparar();
        let creada = crear(&conn, &reserva(1, "2026-10-01", 4)).unwrap();

        eliminar(&conn, creada.id).unwrap();

        assert!(matches!(
            eliminar(&conn, creada.id),
            Err(ErrorRepositorio::NoEncontrado)
        ));
    }
}
