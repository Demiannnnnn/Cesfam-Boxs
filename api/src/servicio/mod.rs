//! Reglas de negocio entre HTTP y repositorio (RF-03 – RF-06, RF-12 – RF-14).
//!
//! Funciones síncronas que reciben la conexión: la capa HTTP decide en qué hilo
//! se ejecutan. Aquí se normalizan textos y se validan reglas que el esquema no expresa.

mod catalogos;
mod reservas;

pub use catalogos::{
    actualizar_box, actualizar_etiqueta, actualizar_funcionario, crear_box, crear_etiqueta,
    crear_funcionario, listar_boxes, listar_etiquetas, listar_funcionarios,
};
pub use reservas::{
    cancelar_grupo, cancelar_reserva, crear_lote, crear_reserva, listar_reservas, ConsultaReservas,
};

use crate::repositorio::ErrorRepositorio;

/// Largo máximo del nombre de un box.
pub const MAX_NOMBRE_BOX: usize = 40;
/// Largo máximo del nombre de una etiqueta.
pub const MAX_NOMBRE_ETIQUETA: usize = 40;
/// Largo máximo del nombre del funcionario.
pub const MAX_FUNCIONARIO: usize = 80;
/// Largo máximo del motivo.
pub const MAX_MOTIVO: usize = 200;
/// Máximo de días que abarca una consulta por rango (exportación).
pub const MAX_DIAS_RANGO: i64 = 366;
/// Máximo de fechas en un lote: alcanza para todos los días hábiles de dos meses.
pub const MAX_FECHAS_LOTE: usize = 46;

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
