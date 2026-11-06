mod comun;

use axum::http::StatusCode;
use comun::{crear_etiqueta, pedir, router_de_prueba};
use serde_json::json;

#[tokio::test]
async fn crear_y_listar_etiquetas_en_orden_alfabetico() {
    let router = router_de_prueba();

    let e1 = pedir(
        &router,
        "POST",
        "/etiquetas",
        Some(json!({ "nombre": "  Enfermería  ", "color": "#2563EB" })),
    )
    .await;
    let e2 = pedir(
        &router,
        "POST",
        "/etiquetas",
        Some(json!({ "nombre": "Cirugía", "color": "#dc2626" })),
    )
    .await;

    assert_eq!(e1.estado, StatusCode::CREATED);
    assert_eq!(e1.cuerpo["nombre"], "Enfermería");
    assert_eq!(e1.cuerpo["color"], "#2563eb");
    assert_eq!(e1.cuerpo["activo"], true);
    assert_eq!(e2.estado, StatusCode::CREATED);

    let lista = pedir(&router, "GET", "/etiquetas", None).await;
    assert_eq!(lista.estado, StatusCode::OK);
    let nombres: Vec<&str> = lista
        .cuerpo
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["nombre"].as_str().unwrap())
        .collect();
    assert_eq!(nombres, ["Cirugía", "Enfermería"]);
}

#[tokio::test]
async fn nombre_duplicado_sin_distinguir_mayusculas_devuelve_409() {
    let router = router_de_prueba();
    crear_etiqueta(&router, "Cirugía", "#dc2626").await;

    let r = pedir(
        &router,
        "POST",
        "/etiquetas",
        Some(json!({ "nombre": "cirugía", "color": "#2563eb" })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::CONFLICT);
    assert_eq!(
        r.cuerpo,
        json!({
            "codigo": "nombre_duplicado",
            "mensaje": "Ya existe una etiqueta con ese nombre"
        })
    );
}

#[tokio::test]
async fn color_invalido_o_nombre_vacio_devuelve_400() {
    let router = router_de_prueba();

    for cuerpo in [
        json!({ "nombre": "Cirugía", "color": "rojo" }),
        json!({ "nombre": "Cirugía", "color": "#fff" }),
        json!({ "nombre": "Cirugía", "color": "2563eb" }),
        json!({ "nombre": "   ", "color": "#2563eb" }),
    ] {
        let r = pedir(&router, "POST", "/etiquetas", Some(cuerpo.clone())).await;
        assert_eq!(r.estado, StatusCode::BAD_REQUEST, "{cuerpo}");
        assert_eq!(r.cuerpo["codigo"], "datos_invalidos", "{cuerpo}");
    }
}

#[tokio::test]
async fn actualizar_y_desactivar_etiqueta() {
    let router = router_de_prueba();
    let id = crear_etiqueta(&router, "Cirugía", "#dc2626").await;
    crear_etiqueta(&router, "Enfermería", "#2563eb").await;

    let actualizada = pedir(
        &router,
        "PATCH",
        &format!("/etiquetas/{id}"),
        Some(json!({ "nombre": "Cirugía Menor", "color": "#16A34A", "activo": false })),
    )
    .await;
    assert_eq!(actualizada.estado, StatusCode::OK);
    assert_eq!(actualizada.cuerpo["nombre"], "Cirugía Menor");
    assert_eq!(actualizada.cuerpo["color"], "#16a34a");
    assert_eq!(actualizada.cuerpo["activo"], false);

    let activas = pedir(&router, "GET", "/etiquetas", None).await;
    let todas = pedir(&router, "GET", "/etiquetas?incluir_inactivos=true", None).await;
    assert_eq!(activas.cuerpo.as_array().unwrap().len(), 1);
    assert_eq!(todas.cuerpo.as_array().unwrap().len(), 2);

    let inexistente = pedir(
        &router,
        "PATCH",
        "/etiquetas/999",
        Some(json!({ "activo": false })),
    )
    .await;
    assert_eq!(inexistente.estado, StatusCode::NOT_FOUND);
    assert_eq!(inexistente.cuerpo["mensaje"], "No se encontró la etiqueta");
}
