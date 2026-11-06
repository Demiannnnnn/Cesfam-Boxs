//! Reglas de negocio de búsqueda de reservas y de boxes libres (RF-15).

use gestion_box_shared::{BoxAtencion, ConsultaBoxesLibres, ConsultaBuscarReservas, Reserva};
use rusqlite::Connection;

use super::{validar_rango_fechas, Entidad, ErrorServicio, MAX_RESULTADOS_BUSQUEDA};
use crate::repositorio::{boxes, reservas};

/// Busca reservas en `desde..=hasta` filtrando por `funcionario_id`, `etiqueta_id`
/// (del funcionario o del box) y/o `box_id` (combinables con `AND`), ordenadas por
/// `(fecha, box_id, bloque)` hasta [`MAX_RESULTADOS_BUSQUEDA`] filas.
///
/// # Errores
/// [`ErrorServicio::DatosInvalidos`] si `desde > hasta`, si el rango supera 366 días
/// o si no se indica al menos un filtro (`funcionario_id`, `etiqueta_id` o `box_id`).
///
/// Cubre RF-15.
pub fn buscar_reservas(
    conn: &Connection,
    consulta: &ConsultaBuscarReservas,
) -> Result<Vec<Reserva>, ErrorServicio> {
    validar_rango_fechas(consulta.desde, consulta.hasta)?;
    if consulta.funcionario_id.is_none()
        && consulta.etiqueta_id.is_none()
        && consulta.box_id.is_none()
    {
        return Err(ErrorServicio::DatosInvalidos(
            "Indica al menos un funcionario, etiqueta o box".into(),
        ));
    }
    reservas::buscar(conn, consulta, MAX_RESULTADOS_BUSQUEDA)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Reserva))
}

/// Lista los boxes activos que no tienen ninguna reserva en `desde..=hasta` de `fecha`,
/// filtrados por `etiqueta_id` del box si se indica, en el orden de la grilla.
///
/// # Errores
/// [`ErrorServicio::DatosInvalidos`] si `desde > hasta`.
///
/// Cubre RF-15.
pub fn boxes_libres(
    conn: &Connection,
    consulta: &ConsultaBoxesLibres,
) -> Result<Vec<BoxAtencion>, ErrorServicio> {
    if consulta.desde > consulta.hasta {
        return Err(ErrorServicio::DatosInvalidos(
            "El bloque \"desde\" debe ser anterior o igual a \"hasta\"".into(),
        ));
    }
    boxes::listar_libres(
        conn,
        consulta.fecha,
        consulta.desde,
        consulta.hasta,
        consulta.etiqueta_id,
    )
    .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::servicio::{
        actualizar_box, crear_box, crear_etiqueta, crear_funcionario, crear_reserva,
    };
    use gestion_box_shared::{
        Bloque, CambiosBox, NuevaEtiqueta, NuevaReserva, NuevoBox, NuevoFuncionario,
    };

    fn ahora() -> chrono::NaiveDateTime {
        "2026-09-30T10:00:00".parse().unwrap()
    }

    #[test]
    fn buscar_reservas_exige_al_menos_un_filtro() {
        let conn = db::abrir_en_memoria().unwrap();
        let consulta = ConsultaBuscarReservas {
            desde: "2026-10-01".parse().unwrap(),
            hasta: "2026-10-31".parse().unwrap(),
            funcionario_id: None,
            etiqueta_id: None,
            box_id: None,
        };

        let err = buscar_reservas(&conn, &consulta).unwrap_err();

        assert!(matches!(
            err,
            ErrorServicio::DatosInvalidos(ref m) if m == "Indica al menos un funcionario, etiqueta o box"
        ));
    }

    #[test]
    fn buscar_reservas_filtra_por_etiqueta_de_funcionario_o_de_box() {
        let conn = db::abrir_en_memoria().unwrap();
        let et_enfermeria = crear_etiqueta(
            &conn,
            &NuevaEtiqueta {
                nombre: "Enfermería".into(),
                color: "#2563eb".parse().unwrap(),
            },
        )
        .unwrap();
        let et_dental = crear_etiqueta(
            &conn,
            &NuevaEtiqueta {
                nombre: "Dental".into(),
                color: "#16a34a".parse().unwrap(),
            },
        )
        .unwrap();
        let box_enf = crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 1".into(),
                etiqueta_id: Some(et_enfermeria.id),
            },
        )
        .unwrap();
        let box_sin = crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 2".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        let func_enf = crear_funcionario(
            &conn,
            &NuevoFuncionario {
                nombre: "Olivia".into(),
                etiqueta_id: Some(et_enfermeria.id),
            },
        )
        .unwrap();
        let func_dental = crear_funcionario(
            &conn,
            &NuevoFuncionario {
                nombre: "Carlos".into(),
                etiqueta_id: Some(et_dental.id),
            },
        )
        .unwrap();

        // Carlos (Dental) atiende en Box 1 (Enfermería) → coincide con ambas etiquetas.
        crear_reserva(
            &conn,
            &NuevaReserva {
                box_id: box_enf.id,
                fecha: "2026-10-02".parse().unwrap(),
                bloque: Bloque::new(4).unwrap(),
                funcionario_id: func_dental.id,
                motivo: None,
            },
            ahora(),
        )
        .unwrap();
        // Olivia (Enfermería) atiende en Box 2 (sin etiqueta) → coincide con Enfermería.
        crear_reserva(
            &conn,
            &NuevaReserva {
                box_id: box_sin.id,
                fecha: "2026-10-01".parse().unwrap(),
                bloque: Bloque::new(2).unwrap(),
                funcionario_id: func_enf.id,
                motivo: None,
            },
            ahora(),
        )
        .unwrap();

        let por_enf = buscar_reservas(
            &conn,
            &ConsultaBuscarReservas {
                desde: "2026-10-01".parse().unwrap(),
                hasta: "2026-10-10".parse().unwrap(),
                funcionario_id: None,
                etiqueta_id: Some(et_enfermeria.id),
                box_id: None,
            },
        )
        .unwrap();
        assert_eq!(por_enf.len(), 2);
        assert_eq!(por_enf[0].fecha.to_string(), "2026-10-01");
        assert_eq!(por_enf[1].fecha.to_string(), "2026-10-02");

        // Combinando etiqueta Enfermería + funcionario Carlos → solo la del 2026-10-02.
        let combinada = buscar_reservas(
            &conn,
            &ConsultaBuscarReservas {
                desde: "2026-10-01".parse().unwrap(),
                hasta: "2026-10-10".parse().unwrap(),
                funcionario_id: Some(func_dental.id),
                etiqueta_id: Some(et_enfermeria.id),
                box_id: None,
            },
        )
        .unwrap();
        assert_eq!(combinada.len(), 1);
        assert_eq!(combinada[0].box_id, box_enf.id);
    }

    #[test]
    fn boxes_libres_excluye_inactivos_y_parcialmente_ocupados() {
        let conn = db::abrir_en_memoria().unwrap();
        let b1 = crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 1".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        let b2 = crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 2".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        let b3 = crear_box(
            &conn,
            &NuevoBox {
                nombre: "Box 3".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        actualizar_box(
            &conn,
            b3.id,
            &CambiosBox {
                activo: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        let func = crear_funcionario(
            &conn,
            &NuevoFuncionario {
                nombre: "Ana".into(),
                etiqueta_id: None,
            },
        )
        .unwrap();
        crear_reserva(
            &conn,
            &NuevaReserva {
                box_id: b1.id,
                fecha: "2026-10-01".parse().unwrap(),
                bloque: Bloque::new(3).unwrap(),
                funcionario_id: func.id,
                motivo: None,
            },
            ahora(),
        )
        .unwrap();

        let libres = boxes_libres(
            &conn,
            &ConsultaBoxesLibres {
                fecha: "2026-10-01".parse().unwrap(),
                desde: Bloque::new(0).unwrap(),
                hasta: Bloque::new(5).unwrap(),
                etiqueta_id: None,
            },
        )
        .unwrap();

        assert_eq!(libres.len(), 1);
        assert_eq!(libres[0].id, b2.id);
    }
}
