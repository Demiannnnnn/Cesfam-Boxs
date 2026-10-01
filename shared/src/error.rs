//! Cuerpo de error común a todas las respuestas HTTP fallidas.

use serde::{Deserialize, Serialize};

/// Mensaje exacto exigido por RF-05.
pub const MENSAJE_BLOQUE_OCUPADO: &str = "Ese bloque acaba de ser tomado";

/// Código estable que la app usa para decidir cómo reaccionar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodigoError {
    DatosInvalidos,
    NoEncontrado,
    BloqueOcupado,
    NombreDuplicado,
    BoxInactivo,
    ErrorInterno,
}

/// Cuerpo JSON de cualquier error: `{"codigo": "...", "mensaje": "..."}`.
///
/// `mensaje` está en español y puede mostrarse directamente al usuario (RNF-09).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorRespuesta {
    pub codigo: CodigoError,
    pub mensaje: String,
}

impl ErrorRespuesta {
    pub fn new(codigo: CodigoError, mensaje: impl Into<String>) -> Self {
        Self {
            codigo,
            mensaje: mensaje.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloque_ocupado_serializa_en_snake_case() {
        let error = ErrorRespuesta::new(CodigoError::BloqueOcupado, MENSAJE_BLOQUE_OCUPADO);

        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            r#"{"codigo":"bloque_ocupado","mensaje":"Ese bloque acaba de ser tomado"}"#
        );
    }
}
