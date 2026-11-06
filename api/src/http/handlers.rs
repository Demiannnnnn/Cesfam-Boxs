//! Handlers HTTP: extraer → llamar al servicio → responder. Sin lógica de negocio.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use gestion_box_shared::{
    CambiosBox, CambiosEtiqueta, CambiosFuncionario, ConsultaBoxesLibres, ConsultaBuscarReservas,
    EstadoSalud, Fecha, NuevaEtiqueta, NuevaReserva, NuevoBox, NuevoFuncionario, NuevoLote,
};
use serde::Deserialize;

use super::extractores::{JsonApi, PathApi, QueryApi};
use super::ErrorApi;
use crate::estado::AppState;
use crate::respaldo;
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

/// `GET /respaldo` (RF-18, ADR-020, ADR-021).
///
/// Entrega una copia consistente de la base SQLite para la copia diaria en los equipos
/// de escritura. El archivo temporal se lee en memoria y se elimina antes de responder.
pub async fn descargar_respaldo(
    State(estado): State<AppState>,
    partes: Parts,
) -> Resultado<Response> {
    let ip = partes
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "desconocida".to_owned());

    tracing::info!(ip = %ip, "entregando copia de respaldo");

    let db = estado.conexion();
    let carpeta = estado.carpeta_respaldos.clone();

    let bytes = tokio::task::spawn_blocking(move || {
        let ruta_temporal = {
            let conn = db
                .lock()
                .unwrap_or_else(|envenenado| envenenado.into_inner());
            respaldo::copia_para_enviar(&conn, &carpeta)
        };

        let ruta = match ruta_temporal {
            Ok(r) => r,
            Err(error) => {
                tracing::error!(%error, "no se pudo crear la copia temporal para el respaldo");
                return Err(ErrorApi::error_interno(
                    "Ocurrió un error al generar la copia de respaldo",
                ));
            }
        };

        let resultado_lectura = std::fs::read(&ruta);
        // Borrar el temporal SIEMPRE, incluso si la lectura falló.
        if let Err(error) = std::fs::remove_file(&ruta) {
            tracing::warn!(%error, ruta = %ruta.display(), "no se pudo borrar el archivo temporal de respaldo");
        }

        match resultado_lectura {
            Ok(contenido) => Ok(contenido),
            Err(error) => {
                tracing::error!(%error, ruta = %ruta.display(), "no se pudo leer el archivo temporal de respaldo");
                Err(ErrorApi::error_interno(
                    "Ocurrió un error al leer la copia de respaldo",
                ))
            }
        }
    })
    .await
    .unwrap_or_else(|error| {
        tracing::error!(%error, "la tarea de respaldo terminó con pánico");
        Err(ErrorApi::error_interno(
            "Ocurrió un error interno al generar la copia de respaldo",
        ))
    })?;

    let ahora = chrono::Local::now();
    let nombre_archivo = ahora.format("gestion-box-%Y%m%d-%H%M%S.db");
    let valor_disposition = format!("attachment; filename=\"{nombre_archivo}\"");

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_DISPOSITION, valor_disposition)
        .body(Body::from(bytes))
        .map_err(|error| {
            tracing::error!(%error, "error al construir la respuesta del respaldo");
            ErrorApi::error_interno("Ocurrió un error al preparar la respuesta")
        })
}

/// Parámetros de listados con filtro de estado (`GET /boxes`, `GET /etiquetas`, `GET /funcionarios`).
#[derive(Debug, Default, Deserialize)]
pub struct ConsultaIncluirInactivos {
    #[serde(default)]
    incluir_inactivos: bool,
}

/// `GET /boxes` (RF-01, RF-06).
pub async fn listar_boxes(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaIncluirInactivos>,
) -> Resultado<impl IntoResponse> {
    let boxes = estado
        .con_db(move |conn| servicio::listar_boxes(conn, consulta.incluir_inactivos))
        .await?;
    Ok(Json(boxes))
}

/// `POST /boxes` (RF-06, RF-14).
pub async fn crear_box(
    State(estado): State<AppState>,
    JsonApi(nuevo): JsonApi<NuevoBox>,
) -> Resultado<impl IntoResponse> {
    let creado = estado
        .con_db(move |conn| servicio::crear_box(conn, &nuevo))
        .await?;
    Ok((StatusCode::CREATED, Json(creado)))
}

/// `PATCH /boxes/{id}` (RF-06, RF-14).
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

/// `GET /boxes/libres?fecha=&desde=&hasta=&etiqueta_id=` (RF-15).
pub async fn boxes_libres(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaBoxesLibres>,
) -> Resultado<impl IntoResponse> {
    let boxes = estado
        .con_db(move |conn| servicio::boxes_libres(conn, &consulta))
        .await?;
    Ok(Json(boxes))
}

/// `GET /etiquetas` (RF-14).
pub async fn listar_etiquetas(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaIncluirInactivos>,
) -> Resultado<impl IntoResponse> {
    let etiquetas = estado
        .con_db(move |conn| servicio::listar_etiquetas(conn, consulta.incluir_inactivos))
        .await?;
    Ok(Json(etiquetas))
}

/// `POST /etiquetas` (RF-14).
pub async fn crear_etiqueta(
    State(estado): State<AppState>,
    JsonApi(nueva): JsonApi<NuevaEtiqueta>,
) -> Resultado<impl IntoResponse> {
    let creada = estado
        .con_db(move |conn| servicio::crear_etiqueta(conn, &nueva))
        .await?;
    Ok((StatusCode::CREATED, Json(creada)))
}

/// `PATCH /etiquetas/{id}` (RF-14).
pub async fn actualizar_etiqueta(
    State(estado): State<AppState>,
    PathApi(id): PathApi<i64>,
    JsonApi(cambios): JsonApi<CambiosEtiqueta>,
) -> Resultado<impl IntoResponse> {
    let actualizada = estado
        .con_db(move |conn| servicio::actualizar_etiqueta(conn, id, &cambios))
        .await?;
    Ok(Json(actualizada))
}

/// `GET /funcionarios` (RF-13).
pub async fn listar_funcionarios(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaIncluirInactivos>,
) -> Resultado<impl IntoResponse> {
    let funcionarios = estado
        .con_db(move |conn| servicio::listar_funcionarios(conn, consulta.incluir_inactivos))
        .await?;
    Ok(Json(funcionarios))
}

/// `POST /funcionarios` (RF-13, RF-14).
pub async fn crear_funcionario(
    State(estado): State<AppState>,
    JsonApi(nuevo): JsonApi<NuevoFuncionario>,
) -> Resultado<impl IntoResponse> {
    let creado = estado
        .con_db(move |conn| servicio::crear_funcionario(conn, &nuevo))
        .await?;
    Ok((StatusCode::CREATED, Json(creado)))
}

/// `PATCH /funcionarios/{id}` (RF-13, RF-14).
pub async fn actualizar_funcionario(
    State(estado): State<AppState>,
    PathApi(id): PathApi<i64>,
    JsonApi(cambios): JsonApi<CambiosFuncionario>,
) -> Resultado<impl IntoResponse> {
    let actualizado = estado
        .con_db(move |conn| servicio::actualizar_funcionario(conn, id, &cambios))
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

/// `GET /reservas/buscar?desde=&hasta=&funcionario_id=&etiqueta_id=&box_id=` (RF-15).
pub async fn buscar_reservas(
    State(estado): State<AppState>,
    QueryApi(consulta): QueryApi<ConsultaBuscarReservas>,
) -> Resultado<impl IntoResponse> {
    let reservas = estado
        .con_db(move |conn| servicio::buscar_reservas(conn, &consulta))
        .await?;
    Ok(Json(reservas))
}

/// `POST /reservas` (RF-03, RF-05, RF-13).
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

/// `POST /reservas/lote`: rango de bloques en una o varias fechas (RF-03, RF-05, RF-13).
pub async fn crear_lote(
    State(estado): State<AppState>,
    JsonApi(lote): JsonApi<NuevoLote>,
) -> Resultado<impl IntoResponse> {
    let ahora = chrono::Local::now().naive_local();
    let resultado = estado
        .con_db(move |conn| servicio::crear_lote(conn, &lote, ahora))
        .await?;
    Ok((StatusCode::CREATED, Json(resultado)))
}

/// Parámetros de `DELETE /reservas/grupo/{grupo}`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultaCancelarGrupo {
    /// Primera fecha a cancelar; las anteriores del grupo se conservan.
    desde: Fecha,
}

/// `DELETE /reservas/grupo/{grupo}?desde=` (RF-04).
pub async fn cancelar_grupo(
    State(estado): State<AppState>,
    PathApi(grupo): PathApi<i64>,
    QueryApi(consulta): QueryApi<ConsultaCancelarGrupo>,
) -> Resultado<impl IntoResponse> {
    let resultado = estado
        .con_db(move |conn| servicio::cancelar_grupo(conn, grupo, consulta.desde))
        .await?;
    Ok(Json(resultado))
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
