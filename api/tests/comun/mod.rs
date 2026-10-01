//! Utilidades compartidas por los tests de integración.
#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use gestion_box_api::db;
use gestion_box_api::estado::{AppState, InfoServidor};
use gestion_box_api::http::crear_router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

/// Router con base en memoria vacía.
pub fn router_de_prueba() -> Router {
    let conn = db::abrir_en_memoria().expect("base en memoria");
    let info = InfoServidor {
        version: "0.0.0-test".into(),
        equipo: "PC-PRUEBA".into(),
        puerto_http: 8000,
    };
    crear_router(AppState::new(conn, info))
}

/// Respuesta simplificada: estado y cuerpo JSON (o `Null` si vacío).
pub struct Respuesta {
    pub estado: StatusCode,
    pub cuerpo: Value,
    pub cabeceras: axum::http::HeaderMap,
}

/// Envía una petición al router. `cuerpo` se envía como JSON si existe.
pub async fn pedir(router: &Router, metodo: &str, uri: &str, cuerpo: Option<Value>) -> Respuesta {
    let mut constructor = Request::builder().method(metodo).uri(uri);
    let body = match cuerpo {
        Some(json) => {
            constructor = constructor.header("content-type", "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    enviar(router, constructor.body(body).unwrap()).await
}

/// Envía una petición ya construida.
pub async fn enviar(router: &Router, peticion: Request<Body>) -> Respuesta {
    let respuesta = router.clone().oneshot(peticion).await.unwrap();
    let estado = respuesta.status();
    let cabeceras = respuesta.headers().clone();
    let bytes = respuesta.into_body().collect().await.unwrap().to_bytes();
    let cuerpo = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| panic!("cuerpo no es JSON: {}", String::from_utf8_lossy(&bytes)))
    };
    Respuesta {
        estado,
        cuerpo,
        cabeceras,
    }
}

/// Crea un box y devuelve su id.
pub async fn crear_box(router: &Router, nombre: &str) -> i64 {
    let r = pedir(
        router,
        "POST",
        "/boxes",
        Some(serde_json::json!({ "nombre": nombre })),
    )
    .await;
    assert_eq!(r.estado, StatusCode::CREATED, "{:?}", r.cuerpo);
    r.cuerpo["id"].as_i64().unwrap()
}
