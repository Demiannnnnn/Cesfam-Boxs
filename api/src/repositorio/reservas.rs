//! SQL de la tabla `reserva` (RF-03, RF-04, RF-05).

use gestion_box_shared::{Bloque, Fecha, Reserva};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{es_violacion_unique, ErrorRepositorio};

const SQL_LISTAR_RANGO: &str =
    "SELECT id, box_id, fecha, bloque, funcionario, motivo, creada_en, grupo FROM reserva
     WHERE fecha BETWEEN ?1 AND ?2
     ORDER BY fecha, bloque, box_id";
const SQL_OBTENER: &str = "SELECT id, box_id, fecha, bloque, funcionario, motivo, creada_en, grupo
     FROM reserva WHERE id = ?1";
const SQL_CREAR: &str =
    "INSERT INTO reserva (box_id, fecha, bloque, funcionario, motivo, creada_en, grupo)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";
const SQL_FUNCIONARIOS: &str = "SELECT funcionario FROM reserva WHERE fecha >= ?1
     GROUP BY funcionario COLLATE NOCASE
     ORDER BY MAX(fecha) DESC, COUNT(*) DESC
     LIMIT ?2";
const SQL_FIJAR_GRUPO: &str = "UPDATE reserva SET grupo = ?2 WHERE id = ?1";
const SQL_ELIMINAR: &str = "DELETE FROM reserva WHERE id = ?1";
const SQL_ELIMINAR_GRUPO: &str = "DELETE FROM reserva WHERE grupo = ?1 AND fecha >= ?2";

/// Datos de una reserva ya validados por la capa de servicio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservaValida {
    pub box_id: i64,
    pub fecha: Fecha,
    pub bloque: Bloque,
    pub funcionario: String,
    pub motivo: Option<String>,
    pub creada_en: String,
    pub grupo: Option<i64>,
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
        grupo: fila.get(7)?,
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
            nueva.grupo,
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

/// Nombres de funcionarios con reservas desde `desde`, sin repetir (sin distinguir
/// mayúsculas), los más recientes primero. Alimenta el autocompletado.
pub fn funcionarios_recientes(
    conn: &Connection,
    desde: Fecha,
    limite: usize,
) -> Result<Vec<String>, ErrorRepositorio> {
    let limite = i64::try_from(limite).unwrap_or(i64::MAX);
    let mut sentencia = conn.prepare_cached(SQL_FUNCIONARIOS)?;
    let nombres = sentencia
        .query_map(params![desde.to_string(), limite], |fila| fila.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(nombres)
}

/// Asigna `grupo` a una reserva ya creada (la primera de un lote da su id al grupo).
pub fn fijar_grupo(conn: &Connection, id: i64, grupo: i64) -> Result<(), ErrorRepositorio> {
    conn.prepare_cached(SQL_FIJAR_GRUPO)?
        .execute(params![id, grupo])?;
    Ok(())
}

/// Elimina las reservas de `grupo` con fecha desde `desde` (inclusive).
/// Devuelve cuántas eliminó (0 si el grupo ya no tenía reservas en ese rango).
pub fn eliminar_grupo(
    conn: &Connection,
    grupo: i64,
    desde: Fecha,
) -> Result<usize, ErrorRepositorio> {
    Ok(conn
        .prepare_cached(SQL_ELIMINAR_GRUPO)?
        .execute(params![grupo, desde.to_string()])?)
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
            grupo: None,
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
    fn eliminar_grupo_respeta_la_fecha_desde() {
        let conn = preparar();
        for fecha in ["2026-10-01", "2026-10-08", "2026-10-15"] {
            crear(
                &conn,
                &ReservaValida {
                    grupo: Some(7),
                    ..reserva(1, fecha, 0)
                },
            )
            .unwrap();
        }
        crear(&conn, &reserva(2, "2026-10-08", 0)).unwrap();

        let eliminadas = eliminar_grupo(&conn, 7, "2026-10-08".parse().unwrap()).unwrap();

        assert_eq!(eliminadas, 2);
        let quedan = listar_por_rango(
            &conn,
            "2026-10-01".parse().unwrap(),
            "2026-10-31".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(quedan.len(), 2, "quedan la del 01 del grupo y la suelta");
    }

    #[test]
    fn funcionarios_sin_repetir_y_mas_recientes_primero() {
        let conn = preparar();
        let con_nombre = |nombre: &str, fecha: &str, bloque: u8| ReservaValida {
            funcionario: nombre.into(),
            ..reserva(1, fecha, bloque)
        };
        crear(&conn, &con_nombre("Ana", "2026-10-01", 0)).unwrap();
        crear(&conn, &con_nombre("ana", "2026-10-02", 0)).unwrap();
        crear(&conn, &con_nombre("Berta", "2026-10-05", 0)).unwrap();
        crear(&conn, &con_nombre("Vieja", "2026-01-01", 0)).unwrap();

        let nombres = funcionarios_recientes(&conn, "2026-09-01".parse().unwrap(), 10).unwrap();

        assert_eq!(nombres.len(), 2);
        assert_eq!(nombres[0], "Berta");
        assert!(nombres[1].eq_ignore_ascii_case("ana"));
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
