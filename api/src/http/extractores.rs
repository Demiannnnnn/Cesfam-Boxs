//! Extractores que responden errores con el cuerpo [`ErrorRespuesta`] estándar
//! en vez del texto plano por defecto de Axum.
//!
//! [`ErrorRespuesta`]: gestion_box_shared::ErrorRespuesta

use axum::extract::FromRequest;
use axum::extract::FromRequestParts;

use super::ErrorApi;

/// `Json<T>` con rechazo `400 datos_invalidos`.
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorApi))]
pub struct JsonApi<T>(pub T);

/// `Query<T>` con rechazo `400 datos_invalidos`.
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ErrorApi))]
pub struct QueryApi<T>(pub T);

/// `Path<T>` con rechazo `400 datos_invalidos`.
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ErrorApi))]
pub struct PathApi<T>(pub T);
