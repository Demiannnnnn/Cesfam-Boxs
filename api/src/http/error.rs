//! Traducción de errores a respuestas HTTP. Único lugar que decide códigos y mensajes.

use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use gestion_box_shared::error::MENSAJE_BLOQUE_OCUPADO;
use gestion_box_shared::{CodigoError, ErrorRespuesta};

use crate::servicio::{Entidad, ErrorServicio};

/// Error listo para responder: estado HTTP + cuerpo [`ErrorRespuesta`].
#[derive(Debug)]
pub struct ErrorApi {
    estado: StatusCode,
    cuerpo: ErrorRespuesta,
}

impl ErrorApi {
    fn new(estado: StatusCode, codigo: CodigoError, mensaje: impl Into<String>) -> Self {
        Self {
            estado,
            cuerpo: ErrorRespuesta::new(codigo, mensaje),
        }
    }

    /// 400 con un mensaje específico.
    pub fn datos_invalidos(mensaje: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            CodigoError::DatosInvalidos,
            mensaje,
        )
    }

    /// 404 genérico (ruta inexistente).
    pub fn ruta_no_encontrada() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            CodigoError::NoEncontrado,
            "La ruta solicitada no existe",
        )
    }
}

impl From<ErrorServicio> for ErrorApi {
    fn from(error: ErrorServicio) -> Self {
        match error {
            ErrorServicio::DatosInvalidos(mensaje) => Self::datos_invalidos(mensaje),
            ErrorServicio::NoEncontrado(entidad) => Self::new(
                StatusCode::NOT_FOUND,
                CodigoError::NoEncontrado,
                match entidad {
                    Entidad::Box => "No se encontró el box",
                    Entidad::Reserva => "No se encontró la reserva",
                },
            ),
            ErrorServicio::BloqueOcupado => Self::new(
                StatusCode::CONFLICT,
                CodigoError::BloqueOcupado,
                MENSAJE_BLOQUE_OCUPADO,
            ),
            ErrorServicio::NombreDuplicado => Self::new(
                StatusCode::CONFLICT,
                CodigoError::NombreDuplicado,
                "Ya existe un box con ese nombre",
            ),
            ErrorServicio::BoxInactivo => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                CodigoError::BoxInactivo,
                "Ese box está desactivado",
            ),
            error @ (ErrorServicio::Interno(_) | ErrorServicio::TareaFallida) => {
                tracing::error!(%error, "error interno");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    CodigoError::ErrorInterno,
                    "Ocurrió un error interno en el servidor",
                )
            }
        }
    }
}

impl From<JsonRejection> for ErrorApi {
    fn from(rechazo: JsonRejection) -> Self {
        Self::datos_invalidos(format!("Cuerpo JSON inválido: {}", rechazo.body_text()))
    }
}

impl From<QueryRejection> for ErrorApi {
    fn from(rechazo: QueryRejection) -> Self {
        Self::datos_invalidos(format!("Parámetros inválidos: {}", rechazo.body_text()))
    }
}

impl From<PathRejection> for ErrorApi {
    fn from(_: PathRejection) -> Self {
        Self::datos_invalidos("El identificador de la ruta debe ser un número entero")
    }
}

impl IntoResponse for ErrorApi {
    fn into_response(self) -> Response {
        (self.estado, Json(self.cuerpo)).into_response()
    }
}
