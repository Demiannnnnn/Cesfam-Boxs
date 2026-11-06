//! Reglas de negocio entre HTTP y repositorio (RF-03 – RF-06, RF-12 – RF-15).
//!
//! Funciones síncronas que reciben la conexión: la capa HTTP decide en qué hilo
//! se ejecutan. Aquí se normalizan textos y se validan reglas que el esquema no expresa.

mod busqueda;
mod catalogos;
mod reservas;

pub use busqueda::{boxes_libres, buscar_reservas};
pub use catalogos::{
    actualizar_box, actualizar_etiqueta, actualizar_funcionario, crear_box, crear_etiqueta,
    crear_funcionario, listar_boxes, listar_etiquetas, listar_funcionarios,
};
pub use reservas::{
    cancelar_grupo, cancelar_reserva, crear_lote, crear_reserva, listar_reservas, ConsultaReservas,
};

use gestion_box_shared::Fecha;

use crate::repositorio::ErrorRepositorio;

/// Largo máximo del nombre de un box.
pub const MAX_NOMBRE_BOX: usize = 40;
/// Largo máximo del nombre de una etiqueta.
pub const MAX_NOMBRE_ETIQUETA: usize = 40;
/// Largo máximo del nombre del funcionario.
pub const MAX_FUNCIONARIO: usize = 80;
/// Largo máximo del motivo.
pub const MAX_MOTIVO: usize = 200;
/// Máximo de días que abarca una consulta por rango (exportación y búsqueda).
pub const MAX_DIAS_RANGO: i64 = 366;
/// Máximo de fechas en un lote: alcanza para todos los días hábiles de dos meses.
pub const MAX_FECHAS_LOTE: usize = 46;
/// Máximo de filas devueltas por `GET /reservas/buscar` (RF-15).
pub const MAX_RESULTADOS_BUSQUEDA: usize = 2000;

/// Entidad a la que se refiere un error de negocio, para un mensaje claro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entidad {
    Box,
    Etiqueta,
    Funcionario,
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
    #[error("nombre duplicado: {0:?}")]
    NombreDuplicado(Entidad),
    #[error("box inactivo")]
    BoxInactivo,
    #[error("funcionario inactivo")]
    FuncionarioInactivo,
    #[error("etiqueta inactiva")]
    EtiquetaInactiva,
    #[error("error de base de datos: {0}")]
    Interno(#[from] rusqlite::Error),
    #[error("la operación terminó inesperadamente")]
    TareaFallida,
}

impl ErrorServicio {
    pub(crate) fn desde_repositorio(error: ErrorRepositorio, entidad: Entidad) -> Self {
        match error {
            ErrorRepositorio::BloqueOcupado => Self::BloqueOcupado,
            ErrorRepositorio::NombreDuplicado => Self::NombreDuplicado(entidad),
            ErrorRepositorio::NoEncontrado => Self::NoEncontrado(entidad),
            ErrorRepositorio::Sqlite(e) => Self::Interno(e),
        }
    }
}

/// Recorta espacios y verifica largo `1..=max` en caracteres.
pub(crate) fn texto_obligatorio(
    valor: &str,
    campo: &str,
    max: usize,
) -> Result<String, ErrorServicio> {
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
pub(crate) fn texto_opcional(
    valor: Option<&str>,
    campo: &str,
    max: usize,
) -> Result<Option<String>, ErrorServicio> {
    match valor.map(str::trim) {
        None | Some("") => Ok(None),
        Some(limpio) => texto_obligatorio(limpio, campo, max).map(Some),
    }
}

/// Verifica que `desde <= hasta` y que el rango no supere [`MAX_DIAS_RANGO`] días.
pub(crate) fn validar_rango_fechas(desde: Fecha, hasta: Fecha) -> Result<(), ErrorServicio> {
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
    Ok(())
}
