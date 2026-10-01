//! Handlers HTTP: extraer → llamar al servicio → responder. Sin lógica de negocio.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use gestion_box_shared::{CambiosBox, EstadoSalud, NuevaReserva, NuevoBox};
use serde::Deserialize;

use super::extractores::{JsonApi, PathApi, QueryApi};
use super::ErrorApi;
use crate::estado::AppState;
use crate::servicio::{self, ConsultaReservas};

type Resultado<T> = Result<T, ErrorApi>;

/// `GET /health` (RF-08). Verifica además que la base responda.
pub async fn salud(State(estado): State<AppState>) -> Resultado<Json<EstadoSalud>> {
    estado
        .con_db(|conn| Ok(conn.query_row("SELECT 1", [], |_| Ok(()))?))
        .await?;
    Ok(Json(EstadoSalud {
        estado: "ok".into(),
        version: estado.info.version.clone(),
        equipo: estado.info.equipo.clone(),
    }))
}

/// Parámetros de `GET /boxes`.
#[derive(Debug, Default, Deserialize)]
pub struct ConsultaBoxes {
    #[serde(default)]
    incluir_inactivos: bool,
}

/// `GET /boxes` (RF-01, RF-06).
pub async fn listar_boxes(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaBoxes>,
) -> Resultado<impl IntoResponse> {
    let boxes = estado
        .con_db(move |conn| servicio::listar_boxes(conn, consulta.incluir_inactivos))
        .await?;
    Ok(Json(boxes))
}

/// `POST /boxes` (RF-06).
pub async fn crear_box(
    State(estado): State<AppState>,
    JsonApi(nuevo): JsonApi<NuevoBox>,
) -> Resultado<impl IntoResponse> {
    let creado = estado
        .con_db(move |conn| servicio::crear_box(conn, &nuevo))
        .await?;
    Ok((StatusCode::CREATED, Json(creado)))
}

/// `PATCH /boxes/{id}` (RF-06).
pub async fn actualizar_box(
    State(estado): State<AppState>,
    PathApi(id): PathApi<i64>,
    JsonApi(cambios): JsonApi<CambiosBox>,
) -> Resultado<impl IntoResponse> {
    let actualizado = estado
        .con_db(move |conn| servicio::actualizar_box(conn, id, &cambios))
        .await?;
    Ok(Json(actualizado))
}

/// `GET /reservas?fecha=` o `?desde=&hasta=` (RF-01, RF-12).
pub async fn listar_reservas(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaReservas>,
) -> Resultado<impl IntoResponse> {
    let reservas = estado
        .con_db(move |conn| servicio::listar_reservas(conn, &consulta))
        .await?;
    Ok(Json(reservas))
}

/// `POST /reservas` (RF-03, RF-05).
pub async fn crear_reserva(
    State(estado): State<AppState>,
    JsonApi(nueva): JsonApi<NuevaReserva>,
) -> Resultado<impl IntoResponse> {
    let ahora = chrono::Local::now().naive_local();
    let creada = estado
        .con_db(move |conn| servicio::crear_reserva(conn, &nueva, ahora))
        .await?;
    Ok((StatusCode::CREATED, Json(creada)))
}

/// `DELETE /reservas/{id}` (RF-04).
pub async fn cancelar_reserva(
    State(estado): State<AppState>,
    PathApi(id): PathApi<i64>,
) -> Resultado<StatusCode> {
    estado
        .con_db(move |conn| servicio::cancelar_reserva(conn, id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Cualquier ruta no definida.
pub async fn ruta_inexistente() -> ErrorApi {
    ErrorApi::ruta_no_encontrada()
}
