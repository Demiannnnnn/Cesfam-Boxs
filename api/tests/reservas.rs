mod comun;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use comun::{crear_box, crear_funcionario, enviar, pedir, router_de_prueba};
use serde_json::{json, Value};

fn reserva(box_id: i64, funcionario_id: i64, fecha: &str, bloque: u8) -> Value {
    json!({
        "box_id": box_id,
        "fecha": fecha,
        "bloque": bloque,
        "funcionario_id": funcionario_id
    })
}

#[tokio::test]
async fn crear_y_listar_por_fecha() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    let func_id = crear_funcionario(&router, "María Pérez").await;

    let creada = pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, func_id, "2026-10-01", 4)),
    )
    .await;
    let del_dia = pedir(&router, "GET", "/reservas?fecha=2026-10-01", None).await;
    let otro_dia = pedir(&router, "GET", "/reservas?fecha=2026-10-02", None).await;

    assert_eq!(creada.estado, StatusCode::CREATED);
    assert!(creada.cuerpo["id"].is_i64());
    assert_eq!(creada.cuerpo["funcionario_id"], func_id);
    assert_eq!(creada.cuerpo["funcionario"], "María Pérez");
    assert!(creada.cuerpo["creada_en"].is_string());
    assert_eq!(del_dia.cuerpo.as_array().unwrap().len(), 1);
    assert_eq!(otro_dia.cuerpo, json!([]));
}

#[tokio::test]
async fn listar_por_rango_incluye_extremos() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 1").await;
    let func_id = crear_funcionario(&router, "María Pérez").await;
    for fecha in ["2026-10-01", "2026-10-31", "2026-11-01"] {
        pedir(
            &router,
            "POST",
            "/reservas",
            Some(reserva(box_id, func_id, fecha, 0)),
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
    let func_id = crear_funcionario(&router, "María Pérez").await;
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, func_id, "2026-10-01", 4)),
    )
    .await;

    let r = pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, func_id, "2026-10-01", 4)),
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
    let func_id = crear_funcionario(&router, "María Pérez").await;

    let mut tareas = tokio::task::JoinSet::new();
    for _ in 0..10 {
        let router = router.clone();
        tareas.spawn(async move {
            pedir(
                &router,
                "POST",
                "/reservas",
                Some(reserva(box_id, func_id, "2026-10-01", 7)),
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
    let box_inactivo = crear_box(&router, "Box 2").await;
    let func_id = crear_funcionario(&router, "Ana").await;
    let func_inactivo = crear_funcionario(&router, "Pedro").await;
    pedir(
        &router,
        "PATCH",
        &format!("/boxes/{box_inactivo}"),
        Some(json!({ "activo": false })),
    )
    .await;
    pedir(
        &router,
        "PATCH",
        &format!("/funcionarios/{func_inactivo}"),
        Some(json!({ "activo": false })),
    )
    .await;

    let casos = [
        (
            json!({ "box_id": box_id, "fecha": "2026-10-01", "bloque": 1, "funcionario": "Ana" }),
            StatusCode::BAD_REQUEST,
            "datos_invalidos",
        ),
        (
            json!({ "box_id": box_id, "fecha": "2026-10-01", "bloque": 24, "funcionario_id": func_id }),
            StatusCode::BAD_REQUEST,
            "datos_invalidos",
        ),
        (
            reserva(999, func_id, "2026-10-01", 1),
            StatusCode::NOT_FOUND,
            "no_encontrado",
        ),
        (
            reserva(box_inactivo, func_id, "2026-10-01", 1),
            StatusCode::UNPROCESSABLE_ENTITY,
            "box_inactivo",
        ),
        (
            reserva(box_id, 999, "2026-10-01", 1),
            StatusCode::NOT_FOUND,
            "no_encontrado",
        ),
        (
            reserva(box_id, func_inactivo, "2026-10-01", 1),
            StatusCode::UNPROCESSABLE_ENTITY,
            "funcionario_inactivo",
        ),
    ];
    for (cuerpo, esperado, codigo) in casos {
        let r = pedir(&router, "POST", "/reservas", Some(cuerpo.clone())).await;
        assert_eq!(r.estado, esperado, "{cuerpo}");
        assert_eq!(r.cuerpo["codigo"], codigo, "{cuerpo}");
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
    let func_id = crear_funcionario(&router, "María Pérez").await;
    let creada = pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, func_id, "2026-10-01", 4)),
    )
    .await;
    let uri = format!("/reservas/{}", creada.cuerpo["id"]);

    let primera = pedir(&router, "DELETE", &uri, None).await;
    let segunda = pedir(&router, "DELETE", &uri, None).await;

    assert_eq!(primera.estado, StatusCode::NO_CONTENT);
    assert_eq!(segunda.estado, StatusCode::NOT_FOUND);
    assert_eq!(segunda.cuerpo["codigo"], "no_encontrado");
}

fn lote(
    box_id: i64,
    funcionario_id: i64,
    fechas: &[&str],
    desde: u8,
    hasta: u8,
    omitir: bool,
) -> Value {
    json!({
        "box_id": box_id, "fechas": fechas, "desde": desde, "hasta": hasta,
        "funcionario_id": funcionario_id, "omitir_ocupados": omitir
    })
}

#[tokio::test]
async fn lote_crea_la_serie_y_devuelve_201_con_grupo_y_rechaza_funcionario_texto() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 17").await;
    let func_id = crear_funcionario(&router, "E.U. Olivia").await;

    let con_texto = pedir(
        &router,
        "POST",
        "/reservas/lote",
        Some(json!({
            "box_id": box_id, "fechas": ["2026-10-05"], "desde": 0, "hasta": 1,
            "funcionario": "E.U. Olivia"
        })),
    )
    .await;
    assert_eq!(con_texto.estado, StatusCode::BAD_REQUEST);

    let r = pedir(
        &router,
        "POST",
        "/reservas/lote",
        Some(lote(
            box_id,
            func_id,
            &["2026-10-05", "2026-10-12"],
            0,
            11,
            false,
        )),
    )
    .await;

    assert_eq!(r.estado, StatusCode::CREATED, "{:?}", r.cuerpo);
    let creadas = r.cuerpo["creadas"].as_array().unwrap();
    assert_eq!(creadas.len(), 24);
    assert_eq!(creadas[0]["grupo"], creadas[23]["grupo"]);
    assert_eq!(creadas[0]["funcionario_id"], func_id);
    assert_eq!(creadas[0]["funcionario"], "E.U. Olivia");
    assert_eq!(r.cuerpo["omitidos"], json!([]));
}

#[tokio::test]
async fn lote_con_conflicto_devuelve_409_y_con_omitir_informa_omitidos() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 17").await;
    let func_id = crear_funcionario(&router, "E.U. Olivia").await;
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_id, func_id, "2026-10-12", 3)),
    )
    .await;

    let todo_o_nada = pedir(
        &router,
        "POST",
        "/reservas/lote",
        Some(lote(
            box_id,
            func_id,
            &["2026-10-05", "2026-10-12"],
            0,
            5,
            false,
        )),
    )
    .await;
    let solo_libres = pedir(
        &router,
        "POST",
        "/reservas/lote",
        Some(lote(
            box_id,
            func_id,
            &["2026-10-05", "2026-10-12"],
            0,
            5,
            true,
        )),
    )
    .await;

    assert_eq!(todo_o_nada.estado, StatusCode::CONFLICT);
    assert_eq!(todo_o_nada.cuerpo["codigo"], "bloque_ocupado");
    assert_eq!(solo_libres.estado, StatusCode::CREATED);
    assert_eq!(solo_libres.cuerpo["creadas"].as_array().unwrap().len(), 11);
    assert_eq!(
        solo_libres.cuerpo["omitidos"],
        json!([{ "fecha": "2026-10-12", "bloque": 3 }])
    );
}

#[tokio::test]
async fn cancelar_grupo_desde_fecha_y_luego_404() {
    let router = router_de_prueba();
    let box_id = crear_box(&router, "Box 17").await;
    let func_id = crear_funcionario(&router, "E.U. Olivia").await;
    let creado = pedir(
        &router,
        "POST",
        "/reservas/lote",
        Some(lote(
            box_id,
            func_id,
            &["2026-10-05", "2026-10-12", "2026-10-19"],
            0,
            1,
            false,
        )),
    )
    .await;
    let grupo = creado.cuerpo["creadas"][0]["grupo"].as_i64().unwrap();
    let uri = format!("/reservas/grupo/{grupo}?desde=2026-10-12");

    let primera = pedir(&router, "DELETE", &uri, None).await;
    let segunda = pedir(&router, "DELETE", &uri, None).await;
    let sin_desde = pedir(&router, "DELETE", &format!("/reservas/grupo/{grupo}"), None).await;

    assert_eq!(primera.estado, StatusCode::OK);
    assert_eq!(primera.cuerpo, json!({ "canceladas": 4 }));
    assert_eq!(segunda.estado, StatusCode::NOT_FOUND);
    assert_eq!(sin_desde.estado, StatusCode::BAD_REQUEST);
}
