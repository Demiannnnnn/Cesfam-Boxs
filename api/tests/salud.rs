mod comun;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use comun::{enviar, pedir, router_de_prueba};

#[tokio::test]
async fn health_responde_ok_con_equipo_y_version() {
    let router = router_de_prueba();

    let r = pedir(&router, "GET", "/health", None).await;

    assert_eq!(r.estado, StatusCode::OK);
    assert_eq!(r.cuerpo["estado"], "ok");
    assert_eq!(r.cuerpo["equipo"], "PC-PRUEBA");
    assert_eq!(r.cuerpo["version"], "0.0.0-test");
}

#[tokio::test]
async fn cors_permite_origen_de_tauri() {
    let router = router_de_prueba();
    let peticion = Request::builder()
        .method("OPTIONS")
        .uri("/reservas")
        .header("origin", "tauri://localhost")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let r = enviar(&router, peticion).await;

    assert!(r.cabeceras.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn ruta_inexistente_responde_404_con_cuerpo_estandar() {
    let router = router_de_prueba();

    let r = pedir(&router, "GET", "/no-existe", None).await;

    assert_eq!(r.estado, StatusCode::NOT_FOUND);
    assert_eq!(r.cuerpo["codigo"], "no_encontrado");
}
