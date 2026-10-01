//! Capa HTTP: router, CORS y extractores.

mod error;
mod extractores;
mod handlers;

pub use error::ErrorApi;

use axum::routing::{get, patch};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::estado::AppState;

/// Construye el router completo. Se usa igual en producción y en tests.
pub fn crear_router(estado: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::salud))
        .route(
            "/boxes",
            get(handlers::listar_boxes).post(handlers::crear_box),
        )
        .route("/boxes/{id}", patch(handlers::actualizar_box))
        .route(
            "/reservas",
            get(handlers::listar_reservas).post(handlers::crear_reserva),
        )
        .route(
            "/reservas/{id}",
            axum::routing::delete(handlers::cancelar_reserva),
        )
        .fallback(handlers::ruta_inexistente)
        // CORS permisivo (RNF-07): la API solo es accesible en la red local, no hay
        // autenticación y los datos no son sensibles (ADR-002, ADR-007).
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(estado)
}
