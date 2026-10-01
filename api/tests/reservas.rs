mod comun;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use comun::{crear_box, enviar, pedir, router_de_prueba};
use serde_json::{json, Value};

fn reserva(box_id: i64, fecha: &str, bloque: u8) -> Value {
    json!({ "box_id": box_id, "fecha": fecha, "bloque": bloque, "funcionario": "María Pérez" })
}

#[tokio::test]
async fn crear_y_listar_por_fecha() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;

    let creada = pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, "2026-10-01", 4)),
    )
    .await;
    let del_dia = pedir(&router, "GET", "/reservas?fecha=2026-10-01", None).await;
    let otro_dia = pedir(&router, "GET", "/reservas?fecha=2026-10-02", None).await;

    assert_eq!(creada.estado, StatusCode::CREATED);
    assert!(creada.cuerpo["id"].is_i64());
    assert!(creada.cuerpo["creada_en"].is_string());
    assert_eq!(del_dia.cuerpo.as_array().unwrap().len(), 1);
    assert_eq!(otro_dia.cuerpo, json!([]));
}

#[tokio::test]
async fn listar_por_rango_incluye_extremos() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    for fecha in ["2026-10-01", "2026-10-31", "2026-11-01"] {
        pedir(
            &router,
            "POST",
            "/reservas",
            Some(reserva(box_id, fecha, 0)),
        )
        .await;
    }

    let r = pedir(
        &router,
        "GET",
        "/reservas?desde=2026-10-01&hasta=2026-10-31",
        None,
    )
    .await;

    assert_eq!(r.estado, StatusCode::OK);
    assert_eq!(r.cuerpo.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn listar_sin_parametros_o_con_fecha_invalida_devuelve_400() {
    let router = router_de_prueba();

    for uri in [
        "/reservas",
        "/reservas?fecha=2026-02-30",
        "/reservas?fecha=ayer",
    ] {
        let r = pedir(&router, "GET", uri, None).await;
        assert_eq!(r.estado, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(r.cuerpo["codigo"], "datos_invalidos", "{uri}");
    }
}

#[tokio::test]
async fn bloque_ocupado_devuelve_409_con_mensaje_exacto() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, "2026-10-01", 4)),
    )
    .await;

    let r = pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, "2026-10-01", 4)),
    )
    .await;

    assert_eq!(r.estado, StatusCode::CONFLICT);
    assert_eq!(
        r.cuerpo,
        json!({ "codigo": "bloque_ocupado", "mensaje": "Ese bloque acaba de ser tomado" })
    );
}

#[tokio::test]
async fn reservas_simultaneas_solo_una_gana() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;

    let mut tareas = tokio::task::JoinSet::new();
    for _ in 0..10 {
        let router = router.clone();
        tareas.spawn(async move {
            pedir(
                &router,
                "POST",
                "/reservas",
                Some(reserva(box_id, "2026-10-01", 7)),
            )
            .await
            .estado
        });
    }
    let estados = tareas.join_all().await;

    let creadas = estados
        .iter()
        .filter(|e| **e == StatusCode::CREATED)
        .count();
    let conflictos = estados
        .iter()
        .filter(|e| **e == StatusCode::CONFLICT)
        .count();
    assert_eq!((creadas, conflictos), (1, 9));
    let lista = pedir(&router, "GET", "/reservas?fecha=2026-10-01", None).await;
    assert_eq!(lista.cuerpo.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn validaciones_de_creacion() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    let inactivo = crear_box(&router, "Box 2").await;
    pedir(
        &router,
        "PATCH",
        &format!("/boxes/{inactivo}"),
        Some(json!({ "activo": false })),
    )
    .await;

    let casos = [
        (
            json!({ "box_id": box_id, "fecha": "2026-10-01", "bloque": 1, "funcionario": "  " }),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({ "box_id": box_id, "fecha": "2026-10-01", "bloque": 24, "funcionario": "Ana" }),
            StatusCode::BAD_REQUEST,
        ),
        (reserva(999, "2026-10-01", 1), StatusCode::NOT_FOUND),
        (
            reserva(inactivo, "2026-10-01", 1),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ];
    for (cuerpo, esperado) in casos {
        let r = pedir(&router, "POST", "/reservas", Some(cuerpo.clone())).await;
        assert_eq!(r.estado, esperado, "{cuerpo}");
        assert!(r.cuerpo["mensaje"].is_string());
    }
}

#[tokio::test]
async fn json_malformado_devuelve_400_con_cuerpo_estandar() {
    let router = router_de_prueba();
    let peticion = Request::builder()
        .method("POST")
        .uri("/reservas")
        .header("content-type", "application/json")
        .body(Body::from("{ esto no es json"))
        .unwrap();

    let r = enviar(&router, peticion).await;

    assert_eq!(r.estado, StatusCode::BAD_REQUEST);
    assert_eq!(r.cuerpo["codigo"], "datos_invalidos");
}

#[tokio::test]
async fn cancelar_dos_veces_devuelve_204_y_luego_404() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    let creada = pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, "2026-10-01", 4)),
    )
    .await;
    let uri = format!("/reservas/{}", creada.cuerpo["id"]);

    let primera = pedir(&router, "DELETE", &uri, None).await;
    let segunda = pedir(&router, "DELETE", &uri, None).await;

    assert_eq!(primera.estado, StatusCode::NO_CONTENT);
    assert_eq!(segunda.estado, StatusCode::NOT_FOUND);
    assert_eq!(segunda.cuerpo["codigo"], "no_encontrado");
}
