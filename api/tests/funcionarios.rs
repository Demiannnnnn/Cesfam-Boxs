mod comun;

use axum::http::StatusCode;
use comun::{crear_box, crear_etiqueta, crear_funcionario, pedir, router_de_prueba};
use serde_json::json;

#[tokio::test]
async fn crear_y_listar_funcionarios_en_orden_alfabetico() {
    let router = router_de_prueba();
    let et_id = crear_etiqueta(&router, "Enfermería", "#2563eb").await;

    let f1 = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "  E.U. Verónica  ", "etiqueta_id": et_id })),
    )
    .await;
    let f2 = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "E.U. Olivia" })),
    )
    .await;

    assert_eq!(f1.estado, StatusCode::CREATED);
    assert_eq!(f1.cuerpo["nombre"], "E.U. Verónica");
    assert_eq!(f1.cuerpo["etiqueta_id"], et_id);
    assert_eq!(f1.cuerpo["activo"], true);
    assert_eq!(f2.estado, StatusCode::CREATED);
    assert!(f2.cuerpo["etiqueta_id"].is_null());

    let lista = pedir(&router, "GET", "/funcionarios", None).await;
    assert_eq!(lista.estado, StatusCode::OK);
    let nombres: Vec<&str> = lista
        .cuerpo
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["nombre"].as_str().unwrap())
        .collect();
    assert_eq!(nombres, ["E.U. Olivia", "E.U. Verónica"]);
}

#[tokio::test]
async fn nombre_duplicado_sin_distinguir_mayusculas_devuelve_409() {
    let router = router_de_prueba();
    crear_funcionario(&router, "María Pérez").await;

    let r = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "maría pérez" })),
    )
    .await;

    assert_eq!(r.estado, StatusCode::CONFLICT);
    assert_eq!(
        r.cuerpo,
        json!({
            "codigo": "nombre_duplicado",
            "mensaje": "Ya existe un funcionario con ese nombre"
        })
    );
}

#[tokio::test]
async fn asignar_y_quitar_etiqueta_con_null() {
    let router = router_de_prueba();
    let et_id = crear_etiqueta(&router, "Cirugía", "#dc2626").await;
    let func_id = crear_funcionario(&router, "Dr. Soto").await;

    let con_etiqueta = pedir(
        &router,
        "PATCH",
        &format!("/funcionarios/{func_id}"),
        Some(json!({ "etiqueta_id": et_id })),
    )
    .await;
    assert_eq!(con_etiqueta.estado, StatusCode::OK);
    assert_eq!(con_etiqueta.cuerpo["etiqueta_id"], et_id);

    let renombrado = pedir(
        &router,
        "PATCH",
        &format!("/funcionarios/{func_id}"),
        Some(json!({ "nombre": "Dr. Juan Soto" })),
    )
    .await;
    assert_eq!(renombrado.cuerpo["etiqueta_id"], et_id);

    let sin_etiqueta = pedir(
        &router,
        "PATCH",
        &format!("/funcionarios/{func_id}"),
        Some(json!({ "etiqueta_id": null })),
    )
    .await;
    assert_eq!(sin_etiqueta.estado, StatusCode::OK);
    assert!(sin_etiqueta.cuerpo["etiqueta_id"].is_null());
}

#[tokio::test]
async fn asignar_etiqueta_inexistente_o_inactiva_falla() {
    let router = router_de_prueba();
    let et_id = crear_etiqueta(&router, "Cirugía", "#dc2626").await;
    pedir(
        &router,
        "PATCH",
        &format!("/etiquetas/{et_id}"),
        Some(json!({ "activo": false })),
    )
    .await;

    let inexistente = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "Ana", "etiqueta_id": 999 })),
    )
    .await;
    let inactiva = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "Ana", "etiqueta_id": et_id })),
    )
    .await;

    assert_eq!(inexistente.estado, StatusCode::NOT_FOUND);
    assert_eq!(inexistente.cuerpo["mensaje"], "No se encontró la etiqueta");
    assert_eq!(inactiva.estado, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        inactiva.cuerpo,
        json!({
            "codigo": "etiqueta_inactiva",
            "mensaje": "Esa etiqueta está desactivada"
        })
    );
}

#[tokio::test]
async fn renombrar_y_desactivar_funcionario_se_refleja_en_reservas_sin_borrarlas() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    let func_id = crear_funcionario(&router, "María").await;

    let creada = pedir(
        &router,
        "POST",
        "/reservas",
        Some(json!({
            "box_id": box_id,
            "fecha": "2026-10-01",
            "bloque": 3,
            "funcionario_id": func_id
        })),
    )
    .await;
    assert_eq!(creada.estado, StatusCode::CREATED);
    assert_eq!(creada.cuerpo["funcionario"], "María");
    assert_eq!(creada.cuerpo["funcionario_id"], func_id);

    let r = pedir(
        &router,
        "PATCH",
        &format!("/funcionarios/{func_id}"),
        Some(json!({ "nombre": "María Pérez", "activo": false })),
    )
    .await;
    assert_eq!(r.estado, StatusCode::OK);

    let activos = pedir(&router, "GET", "/funcionarios", None).await;
    let todos = pedir(&router, "GET", "/funcionarios?incluir_inactivos=true", None).await;
    assert_eq!(activos.cuerpo, json!([]));
    assert_eq!(todos.cuerpo.as_array().unwrap().len(), 1);

    let reservas = pedir(&router, "GET", "/reservas?fecha=2026-10-01", None).await;
    assert_eq!(reservas.cuerpo[0]["funcionario"], "María Pérez");
    assert_eq!(reservas.cuerpo[0]["funcionario_id"], func_id);
}
