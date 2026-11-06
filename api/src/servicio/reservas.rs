//! Reglas de negocio de reservas y lotes (RF-01, RF-03 – RF-05, RF-12, RF-13).

use chrono::NaiveDateTime;
use gestion_box_shared::{
    Bloque, BloqueOmitido, Fecha, NuevaReserva, NuevoLote, Reserva, ResultadoCancelacion,
    ResultadoLote,
};
use rusqlite::Connection;
use serde::Deserialize;

use super::catalogos::{exigir_box_activo, exigir_funcionario_activo};
use super::{
    texto_opcional, validar_rango_fechas, Entidad, ErrorServicio, MAX_FECHAS_LOTE, MAX_MOTIVO,
};
use crate::repositorio::reservas::ReservaValida;
use crate::repositorio::{reservas, ErrorRepositorio};

/// Parámetros de `GET /reservas`: `fecha`, o bien `desde` + `hasta`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultaReservas {
    pub fecha: Option<Fecha>,
    pub desde: Option<Fecha>,
    pub hasta: Option<Fecha>,
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
            validar_rango_fechas(desde, hasta)?;
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

/// Crea una reserva (RF-03, RF-13). El box y el funcionario deben existir y estar activos.
///
/// # Errores
/// - [`ErrorServicio::BloqueOcupado`] si otro la tomó antes (RF-05).
/// - [`ErrorServicio::BoxInactivo`], [`ErrorServicio::FuncionarioInactivo`],
///   [`ErrorServicio::NoEncontrado`], [`ErrorServicio::DatosInvalidos`].
pub fn crear_reserva(
    conn: &Connection,
    nueva: &NuevaReserva,
    ahora: NaiveDateTime,
) -> Result<Reserva, ErrorServicio> {
    let motivo = texto_opcional(nueva.motivo.as_deref(), "motivo", MAX_MOTIVO)?;
    exigir_box_activo(conn, nueva.box_id)?;
    let funcionario = exigir_funcionario_activo(conn, nueva.funcionario_id)?;

    let valida = ReservaValida {
        box_id: nueva.box_id,
        fecha: nueva.fecha,
        bloque: nueva.bloque,
        funcionario_id: funcionario.id,
        funcionario: funcionario.nombre,
        motivo,
        creada_en: marca_de_tiempo(ahora),
        grupo: None,
    };
    let reserva = insertar(conn, &valida)?;
    tracing::info!(
        id = reserva.id,
        box_id = reserva.box_id,
        funcionario_id = funcionario.id,
        fecha = %reserva.fecha,
        bloque = reserva.bloque.valor(),
        "reserva creada"
    );
    Ok(reserva)
}

/// Crea en una sola transacción los bloques `desde..=hasta` del box en cada fecha
/// del lote, todos con el mismo grupo (el id de la primera reserva creada).
///
/// Con `omitir_ocupados = false` es todo o nada: si un bloque está tomado no se crea
/// ninguno. Con `true` se crean los libres y se devuelven los ocupados en `omitidos`.
/// La exclusividad sigue en manos del `UNIQUE` (ADR-003): nunca se consulta antes.
///
/// # Errores
/// - [`ErrorServicio::BloqueOcupado`] si hay un bloque tomado (todo o nada) o si no
///   quedó ningún bloque libre.
/// - [`ErrorServicio::BoxInactivo`], [`ErrorServicio::FuncionarioInactivo`],
///   [`ErrorServicio::NoEncontrado`], [`ErrorServicio::DatosInvalidos`].
///
/// Cubre RF-03, RF-05 y RF-13.
pub fn crear_lote(
    conn: &Connection,
    lote: &NuevoLote,
    ahora: NaiveDateTime,
) -> Result<ResultadoLote, ErrorServicio> {
    let motivo = texto_opcional(lote.motivo.as_deref(), "motivo", MAX_MOTIVO)?;
    if lote.desde > lote.hasta {
        return Err(ErrorServicio::DatosInvalidos(
            "El bloque \"desde\" debe ser anterior o igual a \"hasta\"".into(),
        ));
    }
    let mut fechas = lote.fechas.clone();
    fechas.sort();
    fechas.dedup();
    if fechas.is_empty() || fechas.len() > MAX_FECHAS_LOTE {
        return Err(ErrorServicio::DatosInvalidos(format!(
            "Indica entre 1 y {MAX_FECHAS_LOTE} fechas"
        )));
    }
    exigir_box_activo(conn, lote.box_id)?;
    let funcionario = exigir_funcionario_activo(conn, lote.funcionario_id)?;

    // `unchecked_transaction` porque la conexión llega como `&Connection`; el `Mutex`
    // de `AppState` ya garantiza que nadie más la usa mientras dura la transacción.
    let tx = conn.unchecked_transaction()?;
    let creada_en = marca_de_tiempo(ahora);
    let mut creadas: Vec<Reserva> = Vec::new();
    let mut omitidos = Vec::new();
    for &fecha in &fechas {
        for valor in lote.desde.valor()..=lote.hasta.valor() {
            let bloque =
                Bloque::new(valor).map_err(|e| ErrorServicio::DatosInvalidos(e.to_string()))?;
            let valida = ReservaValida {
                box_id: lote.box_id,
                fecha,
                bloque,
                funcionario_id: funcionario.id,
                funcionario: funcionario.nombre.clone(),
                motivo: motivo.clone(),
                creada_en: creada_en.clone(),
                grupo: creadas.first().map(|r| r.id),
            };
            match insertar(&tx, &valida) {
                Ok(reserva) => creadas.push(reserva),
                Err(ErrorServicio::BloqueOcupado) if lote.omitir_ocupados => {
                    omitidos.push(BloqueOmitido { fecha, bloque });
                }
                // Al salir sin `commit` la transacción se revierte: no queda nada a medias.
                Err(error) => return Err(error),
            }
        }
    }
    let Some(primera) = creadas.first_mut() else {
        return Err(ErrorServicio::BloqueOcupado);
    };
    let grupo = primera.id;
    reservas::fijar_grupo(&tx, grupo, grupo)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Reserva))?;
    primera.grupo = Some(grupo);
    tx.commit()?;

    tracing::info!(
        grupo,
        box_id = lote.box_id,
        funcionario_id = funcionario.id,
        creadas = creadas.len(),
        omitidos = omitidos.len(),
        "lote de reservas creado"
    );
    Ok(ResultadoLote { creadas, omitidos })
}

fn marca_de_tiempo(ahora: NaiveDateTime) -> String {
    ahora.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Inserta y registra el rechazo por bloque ocupado (RF-05).
fn insertar(conn: &Connection, valida: &ReservaValida) -> Result<Reserva, ErrorServicio> {
    reservas::crear(conn, valida).map_err(|error| match error {
        ErrorRepositorio::BloqueOcupado => {
            tracing::warn!(
                box_id = valida.box_id,
                fecha = %valida.fecha,
                bloque = valida.bloque.valor(),
                "reserva rechazada: bloque ocupado"
            );
            ErrorServicio::BloqueOcupado
        }
        otro => ErrorServicio::desde_repositorio(otro, Entidad::Reserva),
    })
}

/// Cancela las reservas de un grupo desde `desde` (inclusive), p. ej. "este día y
/// las semanas siguientes" de una serie.
///
/// # Errores
/// [`ErrorServicio::NoEncontrado`] si no quedaba ninguna (otro PC ya las canceló).
///
/// Cubre RF-04.
pub fn cancelar_grupo(
    conn: &Connection,
    grupo: i64,
    desde: Fecha,
) -> Result<ResultadoCancelacion, ErrorServicio> {
    let canceladas = reservas::eliminar_grupo(conn, grupo, desde)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Reserva))?;
    if canceladas == 0 {
        return Err(ErrorServicio::NoEncontrado(Entidad::Reserva));
    }
    tracing::info!(grupo, %desde, canceladas, "grupo de reservas cancelado");
    Ok(ResultadoCancelacion { canceladas })
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
    use crate::servicio::{actualizar_box, actualizar_funcionario, crear_box, crear_funcionario};
    use gestion_box_shared::{CambiosBox, CambiosFuncionario, NuevoBox, NuevoFuncionario};

    fn ahora() -> NaiveDateTime {
        "2026-09-30T10:00:00".parse().unwrap()
    }

    fn nueva(box_id: i64, funcionario_id: i64, motivo: Option<&str>) -> NuevaReserva {
        NuevaReserva {
            box_id,
            fecha: "2026-10-01".parse().unwrap(),
            bloque: Bloque::new(2).unwrap(),
            funcionario_id,
            motivo: motivo.map(Into::into),
        }
    }

    fn con_box_y_funcionario() -> Connection {
        let conn = db::abrir_en_memoria().unwrap();
        crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 1".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        crear_funcionario(
            &conn,
            &NuevoFuncionario {
                nombre: "  Ana  ".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        conn
    }

    #[test]
    fn normaliza_funcionario_y_motivo_vacio() {
        let conn = con_box_y_funcionario();

        let reserva = crear_reserva(&conn, &nueva(1, 1, Some("   ")), ahora()).unwrap();

        assert_eq!(reserva.funcionario, "Ana");
        assert_eq!(reserva.funcionario_id, Some(1));
        assert_eq!(reserva.motivo, None);
        assert_eq!(reserva.creada_en, "2026-09-30T10:00:00");
    }

    #[test]
    fn funcionario_inexistente_o_inactivo_no_reserva() {
        let conn = con_box_y_funcionario();
        assert!(matches!(
            crear_reserva(&conn, &nueva(1, 99, None), ahora()),
            Err(ErrorServicio::NoEncontrado(Entidad::Funcionario))
        ));

        actualizar_funcionario(
            &conn,
            1,
            &CambiosFuncionario {
                activo: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(matches!(
            crear_reserva(&conn, &nueva(1, 1, None), ahora()),
            Err(ErrorServicio::FuncionarioInactivo)
        ));
    }

    #[test]
    fn box_inactivo_no_se_reserva() {
        let conn = con_box_y_funcionario();
        actualizar_box(
            &conn,
            1,
            &CambiosBox {
                activo: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

        let error = crear_reserva(&conn, &nueva(1, 1, None), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::BoxInactivo));
    }

    #[test]
    fn box_inexistente_es_no_encontrado() {
        let conn = con_box_y_funcionario();

        let error = crear_reserva(&conn, &nueva(9, 1, None), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::NoEncontrado(Entidad::Box)));
    }

    fn lote(fechas: &[&str], desde: u8, hasta: u8, omitir: bool) -> NuevoLote {
        NuevoLote {
            box_id: 1,
            fechas: fechas.iter().map(|f| f.parse().unwrap()).collect(),
            desde: Bloque::new(desde).unwrap(),
            hasta: Bloque::new(hasta).unwrap(),
            funcionario_id: 1,
            motivo: None,
            omitir_ocupados: omitir,
        }
    }

    fn total(conn: &Connection) -> usize {
        let consulta = ConsultaReservas {
            fecha: None,
            desde: Some("2026-01-01".parse().unwrap()),
            hasta: Some("2026-12-31".parse().unwrap()),
        };
        listar_reservas(conn, &consulta).unwrap().len()
    }

    #[test]
    fn lote_crea_cada_bloque_de_cada_fecha_con_un_mismo_grupo() {
        let conn = con_box_y_funcionario();

        let resultado = crear_lote(
            &conn,
            &lote(&["2026-10-08", "2026-10-01"], 0, 3, false),
            ahora(),
        )
        .unwrap();

        assert_eq!(resultado.creadas.len(), 8);
        assert!(resultado.omitidos.is_empty());
        let grupo = resultado.creadas[0].id;
        assert!(resultado.creadas.iter().all(|r| r.grupo == Some(grupo)));
        assert_eq!(
            resultado.creadas[0].fecha.to_string(),
            "2026-10-01",
            "fechas ordenadas"
        );
    }

    #[test]
    fn lote_con_un_bloque_ocupado_no_crea_nada() {
        let conn = con_box_y_funcionario();
        crear_reserva(&conn, &nueva(1, 1, None), ahora()).unwrap();

        let error = crear_lote(&conn, &lote(&["2026-10-01"], 0, 3, false), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::BloqueOcupado));
        assert_eq!(total(&conn), 1, "la transacción se revirtió");
    }

    #[test]
    fn lote_omitiendo_ocupados_crea_los_libres_e_informa_el_resto() {
        let conn = con_box_y_funcionario();
        crear_reserva(&conn, &nueva(1, 1, None), ahora()).unwrap();

        let resultado = crear_lote(&conn, &lote(&["2026-10-01"], 0, 3, true), ahora()).unwrap();

        assert_eq!(resultado.creadas.len(), 3);
        assert_eq!(resultado.omitidos.len(), 1);
        assert_eq!(resultado.omitidos[0].bloque.valor(), 2);
    }

    #[test]
    fn lote_sin_ningun_bloque_libre_es_bloque_ocupado() {
        let conn = con_box_y_funcionario();
        crear_reserva(&conn, &nueva(1, 1, None), ahora()).unwrap();

        let error = crear_lote(&conn, &lote(&["2026-10-01"], 2, 2, true), ahora()).unwrap_err();

        assert!(matches!(error, ErrorServicio::BloqueOcupado));
    }

    #[test]
    fn lote_invertido_vacio_o_demasiado_largo_es_invalido() {
        let conn = con_box_y_funcionario();
        let muchas: Vec<String> = (1..=MAX_FECHAS_LOTE + 1)
            .map(|d| format!("2026-{:02}-{:02}", 1 + (d - 1) / 28, 1 + (d - 1) % 28))
            .collect();
        let muchas: Vec<&str> = muchas.iter().map(String::as_str).collect();

        for invalido in [
            lote(&["2026-10-01"], 4, 3, false),
            lote(&[], 0, 0, false),
            lote(&muchas, 0, 0, false),
        ] {
            assert!(matches!(
                crear_lote(&conn, &invalido, ahora()),
                Err(ErrorServicio::DatosInvalidos(_))
            ));
        }
    }

    #[test]
    fn cancelar_grupo_desde_una_fecha_conserva_las_anteriores() {
        let conn = con_box_y_funcionario();
        let creado = crear_lote(
            &conn,
            &lote(&["2026-10-01", "2026-10-08", "2026-10-15"], 0, 1, false),
            ahora(),
        )
        .unwrap();
        let grupo = creado.creadas[0].id;

        let resultado = cancelar_grupo(&conn, grupo, "2026-10-08".parse().unwrap()).unwrap();

        assert_eq!(resultado.canceladas, 4);
        assert_eq!(total(&conn), 2);
        assert!(matches!(
            cancelar_grupo(&conn, grupo, "2026-10-08".parse().unwrap()),
            Err(ErrorServicio::NoEncontrado(Entidad::Reserva))
        ));
    }

    #[test]
    fn consulta_sin_parametros_o_mezclada_es_invalida() {
        let conn = con_box_y_funcionario();
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
        let conn = con_box_y_funcionario();
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
