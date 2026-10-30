//! Reglas de negocio de boxes, etiquetas y funcionarios (RF-06, RF-13, RF-14).

use gestion_box_shared::{
    BoxAtencion, CambiosBox, CambiosEtiqueta, CambiosFuncionario, Etiqueta, Funcionario,
    NuevaEtiqueta, NuevoBox, NuevoFuncionario,
};
use rusqlite::Connection;

use super::{
    texto_obligatorio, Entidad, ErrorServicio, MAX_FUNCIONARIO, MAX_NOMBRE_BOX, MAX_NOMBRE_ETIQUETA,
};
use crate::repositorio::{boxes, etiquetas, funcionarios};

/// Verifica que `etiqueta_id` exista y esté activa antes de asignarla.
fn exigir_etiqueta_activa(conn: &Connection, etiqueta_id: i64) -> Result<(), ErrorServicio> {
    let etiqueta = etiquetas::obtener(conn, etiqueta_id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Etiqueta))?;
    if etiqueta.activo {
        Ok(())
    } else {
        Err(ErrorServicio::EtiquetaInactiva)
    }
}

/// Verifica que `box_id` exista y esté activo antes de reservarlo.
pub(crate) fn exigir_box_activo(conn: &Connection, box_id: i64) -> Result<(), ErrorServicio> {
    let box_destino = boxes::obtener(conn, box_id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))?;
    if box_destino.activo {
        Ok(())
    } else {
        Err(ErrorServicio::BoxInactivo)
    }
}

/// Verifica que `funcionario_id` exista y esté activo antes de asignarle una reserva.
pub(crate) fn exigir_funcionario_activo(
    conn: &Connection,
    funcionario_id: i64,
) -> Result<Funcionario, ErrorServicio> {
    let funcionario = funcionarios::obtener(conn, funcionario_id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Funcionario))?;
    if funcionario.activo {
        Ok(funcionario)
    } else {
        Err(ErrorServicio::FuncionarioInactivo)
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

/// Crea un box (RF-06, RF-14).
///
/// # Errores
/// - [`ErrorServicio::DatosInvalidos`] si el nombre es vacío o supera [`MAX_NOMBRE_BOX`].
/// - [`ErrorServicio::NombreDuplicado`] si ya existe un box con ese nombre.
/// - [`ErrorServicio::NoEncontrado`] o [`ErrorServicio::EtiquetaInactiva`] si `etiqueta_id` no es válida.
pub fn crear_box(conn: &Connection, nuevo: &NuevoBox) -> Result<BoxAtencion, ErrorServicio> {
    let nombre = texto_obligatorio(&nuevo.nombre, "nombre", MAX_NOMBRE_BOX)?;
    if let Some(etiqueta_id) = nuevo.etiqueta_id {
        exigir_etiqueta_activa(conn, etiqueta_id)?;
    }
    let creado = boxes::crear(conn, &nombre, nuevo.etiqueta_id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))?;
    tracing::info!(id = creado.id, nombre = %creado.nombre, "box creado");
    Ok(creado)
}

/// Renombra, cambia la etiqueta y/o activa/desactiva un box (RF-06, RF-14).
/// Los boxes nunca se borran (ADR-006).
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
    if let Some(Some(etiqueta_id)) = cambios.etiqueta_id {
        exigir_etiqueta_activa(conn, etiqueta_id)?;
    }
    let actualizado = boxes::actualizar(
        conn,
        id,
        nombre.as_deref(),
        cambios.activo,
        cambios.etiqueta_id,
    )
    .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Box))?;
    tracing::info!(
        id,
        nombre = %actualizado.nombre,
        activo = actualizado.activo,
        "box actualizado"
    );
    Ok(actualizado)
}

/// Lista etiquetas en orden alfabético (RF-14).
pub fn listar_etiquetas(
    conn: &Connection,
    incluir_inactivos: bool,
) -> Result<Vec<Etiqueta>, ErrorServicio> {
    etiquetas::listar(conn, incluir_inactivos)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Etiqueta))
}

/// Crea una etiqueta con nombre y color (RF-14).
///
/// # Errores
/// - [`ErrorServicio::DatosInvalidos`] si el nombre es vacío o supera [`MAX_NOMBRE_ETIQUETA`].
/// - [`ErrorServicio::NombreDuplicado`] si ya existe una etiqueta con ese nombre.
pub fn crear_etiqueta(conn: &Connection, nueva: &NuevaEtiqueta) -> Result<Etiqueta, ErrorServicio> {
    let nombre = texto_obligatorio(&nueva.nombre, "nombre", MAX_NOMBRE_ETIQUETA)?;
    let creada = etiquetas::crear(conn, &nombre, &nueva.color)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Etiqueta))?;
    tracing::info!(
        id = creada.id,
        nombre = %creada.nombre,
        color = %creada.color,
        "etiqueta creada"
    );
    Ok(creada)
}

/// Actualiza nombre, color y/o estado activo de una etiqueta (RF-14).
/// Desactivar una etiqueta no borra ni modifica reservas existentes.
pub fn actualizar_etiqueta(
    conn: &Connection,
    id: i64,
    cambios: &CambiosEtiqueta,
) -> Result<Etiqueta, ErrorServicio> {
    let nombre = cambios
        .nombre
        .as_deref()
        .map(|n| texto_obligatorio(n, "nombre", MAX_NOMBRE_ETIQUETA))
        .transpose()?;
    let actualizada = etiquetas::actualizar(
        conn,
        id,
        nombre.as_deref(),
        cambios.color.as_ref(),
        cambios.activo,
    )
    .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Etiqueta))?;
    tracing::info!(
        id,
        nombre = %actualizada.nombre,
        activo = actualizada.activo,
        "etiqueta actualizada"
    );
    Ok(actualizada)
}

/// Lista funcionarios registrados en orden alfabético (RF-13).
pub fn listar_funcionarios(
    conn: &Connection,
    incluir_inactivos: bool,
) -> Result<Vec<Funcionario>, ErrorServicio> {
    funcionarios::listar(conn, incluir_inactivos)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Funcionario))
}

/// Registra un funcionario con etiqueta opcional (RF-13, RF-14).
///
/// # Errores
/// - [`ErrorServicio::DatosInvalidos`] si el nombre es vacío o supera [`MAX_FUNCIONARIO`].
/// - [`ErrorServicio::NombreDuplicado`] si ya existe un funcionario con ese nombre.
/// - [`ErrorServicio::NoEncontrado`] o [`ErrorServicio::EtiquetaInactiva`] si `etiqueta_id` no es válida.
pub fn crear_funcionario(
    conn: &Connection,
    nuevo: &NuevoFuncionario,
) -> Result<Funcionario, ErrorServicio> {
    let nombre = texto_obligatorio(&nuevo.nombre, "nombre", MAX_FUNCIONARIO)?;
    if let Some(etiqueta_id) = nuevo.etiqueta_id {
        exigir_etiqueta_activa(conn, etiqueta_id)?;
    }
    let creado = funcionarios::crear(conn, &nombre, nuevo.etiqueta_id)
        .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Funcionario))?;
    tracing::info!(id = creado.id, nombre = %creado.nombre, "funcionario creado");
    Ok(creado)
}

/// Renombra, cambia la etiqueta (`Some(None)` la quita) y/o activa/desactiva un funcionario (RF-13, RF-14).
/// Desactivar un funcionario no borra ni modifica reservas existentes.
pub fn actualizar_funcionario(
    conn: &Connection,
    id: i64,
    cambios: &CambiosFuncionario,
) -> Result<Funcionario, ErrorServicio> {
    let nombre = cambios
        .nombre
        .as_deref()
        .map(|n| texto_obligatorio(n, "nombre", MAX_FUNCIONARIO))
        .transpose()?;
    if let Some(Some(etiqueta_id)) = cambios.etiqueta_id {
        exigir_etiqueta_activa(conn, etiqueta_id)?;
    }
    let actualizado = funcionarios::actualizar(
        conn,
        id,
        nombre.as_deref(),
        cambios.etiqueta_id,
        cambios.activo,
    )
    .map_err(|e| ErrorServicio::desde_repositorio(e, Entidad::Funcionario))?;
    tracing::info!(
        id,
        nombre = %actualizado.nombre,
        activo = actualizado.activo,
        "funcionario actualizado"
    );
    Ok(actualizado)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use gestion_box_shared::Color;

    #[test]
    fn valida_largo_de_nombre_en_etiqueta_y_funcionario() {
        let conn = db::abrir_en_memoria().unwrap();
        let color: Color = "#2563eb".parse().unwrap();

        assert!(matches!(
            crear_etiqueta(
                &conn,
                &NuevaEtiqueta {
                    nombre: "   ".into(),
                    color: color.clone(),
                }
            ),
            Err(ErrorServicio::DatosInvalidos(_))
        ));
        assert!(matches!(
            crear_etiqueta(
                &conn,
                &NuevaEtiqueta {
                    nombre: "a".repeat(MAX_NOMBRE_ETIQUETA + 1),
                    color,
                }
            ),
            Err(ErrorServicio::DatosInvalidos(_))
        ));
        assert!(matches!(
            crear_funcionario(
                &conn,
                &NuevoFuncionario {
                    nombre: "ñ".repeat(MAX_FUNCIONARIO + 1),
                    etiqueta_id: None,
                }
            ),
            Err(ErrorServicio::DatosInvalidos(_))
        ));
    }

    #[test]
    fn rechaza_asignar_etiqueta_inexistente_o_inactiva() {
        let conn = db::abrir_en_memoria().unwrap();
        let et = crear_etiqueta(
            &conn,
            &NuevaEtiqueta {
                nombre: "Dental".into(),
                color: "#16a34a".parse().unwrap(),
            },
        )
        .unwrap();
        actualizar_etiqueta(
            &conn,
            et.id,
            &CambiosEtiqueta {
                activo: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

        let err_inexistente = crear_funcionario(
            &conn,
            &NuevoFuncionario {
                nombre: "Ana".into(),
                etiqueta_id: Some(999),
            },
        )
        .unwrap_err();
        let err_inactiva = crear_funcionario(
            &conn,
            &NuevoFuncionario {
                nombre: "Ana".into(),
                etiqueta_id: Some(et.id),
            },
        )
        .unwrap_err();

        assert!(matches!(
            err_inexistente,
            ErrorServicio::NoEncontrado(Entidad::Etiqueta)
        ));
        assert!(matches!(err_inactiva, ErrorServicio::EtiquetaInactiva));
    }
}
