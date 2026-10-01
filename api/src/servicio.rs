//! Reglas de negocio entre HTTP y repositorio (RF-03 – RF-06, RF-12).
//!
//! Funciones síncronas que reciben la conexión: la capa HTTP decide en qué hilo
//! se ejecutan. Aquí se normalizan textos y se validan reglas que el esquema no expresa.

use chrono::NaiveDateTime;
use gestion_box_shared::{BoxAtencion, CambiosBox, Fecha, NuevaReserva, NuevoBox, Reserva};
use rusqlite::Connection;
use serde::Deserialize;

use crate::repositorio::reservas::ReservaValida;
use crate::repositorio::{boxes, reservas, ErrorRepositorio};

/// Largo máximo del nombre de un box.
pub const MAX_NOMBRE_BOX: usize = 40;
/// Largo máximo del nombre del funcionario.
pub const MAX_FUNCIONARIO: usize = 80;
/// Largo máximo del motivo.
pub const MAX_MOTIVO: usize = 200;
/// Máximo de días que abarca una consulta por rango (exportación).
pub const MAX_DIAS_RANGO: i64 = 366;

/// Entidad a la que se refiere un "no encontrado", para un mensaje claro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entidad {
    Box,
    Reserva,
}

/// Error de negocio. La capa HTTP lo traduce a código y mensaje.
#[derive(Debug, thiserror::Error)]
pub enum ErrorServicio {
    #[error("{0}")]
    DatosInvalidos(String),
    #[error("no encontrado: {0:?}")]
    NoEncontrado(Entidad),
    #[error("bloque ocupado")]
    BloqueOcupado,
    #[error("nombre de box duplicado")]
    NombreDuplicado,
    #[error("box inactivo")]
    BoxInactivo,
    #[error("error de base de datos: {0}")]
    Interno(#[from] rusqlite::Error),
    #[error("la operación terminó inesperadamente")]
    TareaFallida,
}

impl ErrorServicio {
    fn desde_repositorio(error: ErrorRepositorio, entidad: Entidad) -> Self {
        match error {
            ErrorRepositorio::BloqueOcupado => Self::BloqueOcupado,
            ErrorRepositorio::NombreDuplicado => Self::NombreDuplicado,
            ErrorRepositorio::NoEncontrado => Self::NoEncontrado(entidad),
            ErrorRepositorio::Sqlite(e) => Self::Interno(e),
        }
    }
}

/// Parámetros de `GET /reservas`: `fecha`, o bien `desde` + `hasta`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultaReservas {
    pub fecha: Option<Fecha>,
    pub desde: Option<Fecha>,
    pub hasta: Option<Fecha>,
}

/// Recorta espacios y verifica largo `1..=max` en caracteres.
fn texto_obligatorio(valor: &str, campo: &str, max: usize) -> Result<String, ErrorServicio> {
    let limpio = valor.trim();
    if limpio.is_empty() {
        return Err(ErrorServicio::DatosInvalidos(format!(
            "El campo {campo} es obligatorio"
        )));
    }
    if limpio.chars().count() > max {
        return Err(ErrorServicio::DatosInvalidos(format!(
            "El campo {campo} admite como máximo {max} caracteres"
        )));
    }
    Ok(limpio.to_owned())
}

/// Recorta espacios; vacío → `None`; verifica largo máximo.
fn texto_opcional(
    valor: Option<&str>,
    campo: &str,
    max: usize,
) -> Result<Option<String>, ErrorServicio> {
    match valor.map(str::trim) {
        None | Some("") => Ok(None),
        Some(limpio) => texto_obligatorio(limpio, campo, max).map(Some),
    }
}

/// Lista boxes (RF-01, RF-06).
pub fn listar_boxes(
    conn: &Connection,
    incluir_inactivos: bool,
) -> Result<Vec<BoxAtencion>, ErrorServicio> {
    boxes::listar(conn, incluir_inactivos)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))
}

/// Crea un box (RF-06).
pub fn crear_box(conn: &Connection, nuevo: &NuevoBox) -> Result<BoxAtencion, ErrorServicio> {
    let nombre = texto_obligatorio(&nuevo.nombre, "nombre", MAX_NOMBRE_BOX)?;
    let creado = boxes::crear(conn, &nombre)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))?;
    tracing::info!(id = creado.id, nombre = %creado.nombre, "box creado");
    Ok(creado)
}

/// Renombra y/o activa/desactiva un box (RF-06). Los boxes nunca se borran (ADR-006).
pub fn actualizar_box(
    conn: &Connection,
    id: i64,
    cambios: &CambiosBox,
) -> Result<BoxAtencion, ErrorServicio> {
    let nombre = cambios
        .nombre
        .as_deref()
        .map(|n| texto_obligatorio(n, "nombre", MAX_NOMBRE_BOX))
        .transpose()?;
    let actualizado = boxes::actualizar(conn, id, nombre.as_deref(), cambios.activo)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))?;
    tracing::info!(
        id,
        nombre = %actualizado.nombre,
        activo = actualizado.activo,
        "box actualizado"
    );
    Ok(actualizado)
}

/// Lista reservas de un día o de un rango (RF-01, RF-12).
pub fn listar_reservas(
    conn: &Connection,
    consulta: &ConsultaReservas,
) -> Result<Vec<Reserva>, ErrorServicio> {
    let (desde, hasta) = match *consulta {
        ConsultaReservas {
            fecha: Some(fecha),
            desde: None,
            hasta: None,
        } => (fecha, fecha),
        ConsultaReservas {
            fecha: None,
            desde: Some(desde),
            hasta: Some(hasta),
        } => {
            let dias = desde.dias_hasta(hasta);
            if dias < 0 {
                return Err(ErrorServicio::DatosInvalidos(
                    "La fecha \"desde\" debe ser anterior o igual a \"hasta\"".into(),
                ));
            }
            if dias >= MAX_DIAS_RANGO {
                return Err(ErrorServicio::DatosInvalidos(format!(
                    "El rango no puede superar {MAX_DIAS_RANGO} días"
                )));
            }
            (desde, hasta)
        }
        _ => {
            return Err(ErrorServicio::DatosInvalidos(
                "Indica \"fecha\", o bien \"desde\" y \"hasta\"".into(),
            ))
        }
    };
    reservas::listar_por_rango(conn, desde, hasta)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Reserva))
}

/// Crea una reserva (RF-03). El box debe existir y estar activo.
///
/// # Errores
/// - [`ErrorServicio::BloqueOcupado`] si otro la tomó antes (RF-05).
/// - [`ErrorServicio::BoxInactivo`], [`ErrorServicio::NoEncontrado`], [`ErrorServicio::DatosInvalidos`].
pub fn crear_reserva(
    conn: &Connection,
    nueva: &NuevaReserva,
    ahora: NaiveDateTime,
) -> Result<Reserva, ErrorServicio> {
    let funcionario = texto_obligatorio(&nueva.funcionario, "funcionario", MAX_FUNCIONARIO)?;
    let motivo = texto_opcional(nueva.motivo.as_deref(), "motivo", MAX_MOTIVO)?;

    let box_destino = boxes::obtener(conn, nueva.box_id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))?;
    if !box_destino.activo {
        return Err(ErrorServicio::BoxInactivo);
    }

    let valida = ReservaValida {
        box_id: nueva.box_id,
        fecha: nueva.fecha,
        bloque: nueva.bloque,
        funcionario,
        motivo,
        creada_en: ahora.format("%Y-%m-%dT%H:%M:%S").to_string(),
    };
    match reservas::crear(conn, &valida) {
        Ok(reserva) => {
            tracing::info!(
                id = reserva.id,
                box_id = reserva.box_id,
                fecha = %reserva.fecha,
                bloque = reserva.bloque.valor(),
                "reserva creada"
            );
            Ok(reserva)
        }
        Err(ErrorRepositorio::BloqueOcupado) => {
            tracing::warn!(
                box_id = valida.box_id,
                fecha = %valida.fecha,
                bloque = valida.bloque.valor(),
                "reserva rechazada: bloque ocupado"
            );
            Err(ErrorServicio::BloqueOcupado)
        }
        Err(e) => Err(ErrorServicio::desde_repositorio(e, Entidad::Reserva)),
    }
}

/// Cancela (elimina) una reserva (RF-04).
pub fn cancelar_reserva(conn: &Connection, id: i64) -> Result<(), ErrorServicio> {
    reservas::eliminar(conn, id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Reserva))?;
    tracing::info!(id, "reserva cancelada");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use gestion_box_shared::Bloque;

    fn ahora() -> NaiveDateTime {
        "2026-09-30T10:00:00".parse().unwrap()
    }

    fn nueva(box_id: i64, funcionario: &str, motivo: Option<&str>) -> NuevaReserva {
        NuevaReserva {
            box_id,
            fecha: "2026-10-01".parse().unwrap(),
            bloque: Bloque::new(2).unwrap(),
            funcionario: funcionario.into(),
            motivo: motivo.map(Into::into),
        }
    }

    fn con_box() -> Connection {
        let conn = db::abrir_en_memoria().unwrap();
        crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 1".into(),
            },
        )
        .unwrap();
        conn
    }

    #[test]
    fn normaliza_funcionario_y_motivo_vacio() {
        let conn = con_box();

        let reserva = crear_reserva(&conn, &nueva(1, "  Ana  ", Some("   ")), ahora()).unwrap();

        assert_eq!(reserva.funcionario, "Ana");
        assert_eq!(reserva.motivo, None);
        assert_eq!(reserva.creada_en, "2026-09-30T10:00:00");
    }

    #[test]
    fn funcionario_vacio_es_dato_invalido() {
        let conn = con_box();

        let error = crear_reserva(&conn, &nueva(1, "   ", None), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::DatosInvalidos(_)));
    }

    #[test]
    fn funcionario_demasiado_largo_es_dato_invalido() {
        let conn = con_box();
        let largo = "ñ".repeat(MAX_FUNCIONARIO + 1);

        let error = crear_reserva(&conn, &nueva(1, &largo, None), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::DatosInvalidos(_)));
    }

    #[test]
    fn box_inactivo_no_se_reserva() {
        let conn = con_box();
        let cambios = CambiosBox {
            activo: Some(false),
            ..Default::default()
        };
        actualizar_box(&conn, 1, &cambios).unwrap();

        let error = crear_reserva(&conn, &nueva(1, "Ana", None), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::BoxInactivo));
    }

    #[test]
    fn box_inexistente_es_no_encontrado() {
        let conn = con_box();

        let error = crear_reserva(&conn, &nueva(9, "Ana", None), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::NoEncontrado(Entidad::Box)));
    }

    #[test]
    fn consulta_sin_parametros_o_mezclada_es_invalida() {
        let conn = con_box();
        let fecha: Fecha = "2026-10-01".parse().unwrap();

        for consulta in [
            ConsultaReservas::default(),
            ConsultaReservas {
                fecha: Some(fecha),
                desde: Some(fecha),
                hasta: None,
            },
            ConsultaReservas {
                fecha: None,
                desde: Some(fecha),
                hasta: None,
            },
        ] {
            assert!(
                listar_reservas(&conn, &consulta).is_err(),
                "debió rechazar {consulta:?}"
            );
        }
    }

    #[test]
    fn rango_invertido_o_demasiado_largo_es_invalido() {
        let conn = con_box();
        let consulta = |desde: &str, hasta: &str| ConsultaReservas {
            fecha: None,
            desde: Some(desde.parse().unwrap()),
            hasta: Some(hasta.parse().unwrap()),
        };

        assert!(listar_reservas(&conn, &consulta("2026-10-02", "2026-10-01")).is_err());
        assert!(listar_reservas(&conn, &consulta("2026-01-01", "2027-01-02")).is_err());
        assert!(listar_reservas(&conn, &consulta("2026-01-01", "2026-12-31")).is_ok());
    }
}
