mod comun;

use axum::http::StatusCode;
use comun::{crear_box, crear_etiqueta, pedir, router_de_prueba};
use serde_json::json;

#[tokio::test]
async fn crear_box_devuelve_201_activo() {
    let router = router_de_prueba();

    let r = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "Box 1" })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::CREATED);
    assert_eq!(r.cuerpo["nombre"], "Box 1");
    assert_eq!(r.cuerpo["activo"], true);
}

#[tokio::test]
async fn nombre_duplicado_devuelve_409() {
    let router = router_de_prueba();
    crear_box(&router, "Box 1").await;

    let r = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "box 1" })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::CONFLICT);
    assert_eq!(r.cuerpo["codigo"], "nombre_duplicado");
}

#[tokio::test]
async fn nombre_vacio_devuelve_400() {
    let router = router_de_prueba();

    let r = pedir(&router, "POST", "/boxes", Some(json!({ "nombre": "   " }))).await;

    assert_eq!(r.estado, StatusCode::BAD_REQUEST);
    assert_eq!(r.cuerpo["codigo"], "datos_invalidos");
}

#[tokio::test]
async fn desactivar_oculta_el_box_salvo_que_se_pidan_todos() {
    let router = router_de_prueba();
    let id = crear_box(&router, "Box 1").await;
    crear_box(&router, "Box 2").await;

    let r = pedir(
        &router,
        "PATCH",
        &format!("/boxes/{id}"),
        Some(json!({ "activo": false })),
    )
    .await;
    assert_eq!(r.estado, StatusCode::OK);
    assert_eq!(r.cuerpo["activo"], false);

    let activos = pedir(&router, "GET", "/boxes", None).await;
    let todos = pedir(&router, "GET", "/boxes?incluir_inactivos=true", None).await;
    assert_eq!(activos.cuerpo.as_array().unwrap().len(), 1);
    assert_eq!(todos.cuerpo.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn renombrar_box() {
    let router = router_de_prueba();
    let id = crear_box(&router, "Box 1").await;

    let r = pedir(
        &router,
        "PATCH",
        &format!("/boxes/{id}"),
        Some(json!({ "nombre": "Box Dental" })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::OK);
    assert_eq!(r.cuerpo["nombre"], "Box Dental");
}

#[tokio::test]
async fn actualizar_box_inexistente_devuelve_404() {
    let router = router_de_prueba();

    let r = pedir(
        &router,
        "PATCH",
        "/boxes/999",
        Some(json!({ "activo": false })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn id_no_numerico_devuelve_400() {
    let router = router_de_prueba();

    let r = pedir(
        &router,
        "PATCH",
        "/boxes/abc",
        Some(json!({ "activo": false })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::BAD_REQUEST);
    assert_eq!(r.cuerpo["codigo"], "datos_invalidos");
}

#[tokio::test]
async fn asignar_quitar_y_validar_etiqueta_en_box() {
    let router = router_de_prueba();
    let et_id = crear_etiqueta(&router, "Dental", "#16a34a").await;

    let creado = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "Box 1", "etiqueta_id": et_id })),
    )
    .await;
    assert_eq!(creado.estado, StatusCode::CREATED);
    assert_eq!(creado.cuerpo["etiqueta_id"], et_id);
    let box_id = creado.cuerpo["id"].as_i64().unwrap();

    let sin_etiqueta = pedir(
        &router,
        "PATCH",
        &format!("/boxes/{box_id}"),
        Some(json!({ "etiqueta_id": null })),
    )
    .await;
    assert_eq!(sin_etiqueta.estado, StatusCode::OK);
    assert!(sin_etiqueta.cuerpo["etiqueta_id"].is_null());

    pedir(
        &router,
        "PATCH",
        &format!("/etiquetas/{et_id}"),
        Some(json!({ "activo": false })),
    )
    .await;
    let inactiva = pedir(
        &router,
        "PATCH",
        &format!("/boxes/{box_id}"),
        Some(json!({ "etiqueta_id": et_id })),
    )
    .await;
    assert_eq!(inactiva.estado, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(inactiva.cuerpo["codigo"], "etiqueta_inactiva");

    let inexistente = pedir(
        &router,
        "PATCH",
        &format!("/boxes/{box_id}"),
        Some(json!({ "etiqueta_id": 999 })),
    )
    .await;
    assert_eq!(inexistente.estado, StatusCode::NOT_FOUND);
    assert_eq!(inexistente.cuerpo["mensaje"], "No se encontró la etiqueta");
}
