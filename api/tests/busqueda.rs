mod comun;

use axum::http::StatusCode;
use comun::{crear_box, crear_etiqueta, crear_funcionario, pedir, router_de_prueba};
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
async fn buscar_reservas_por_funcionario_etiqueta_box_y_combinaciones() {
    let router = router_de_prueba();
    let et_enf = crear_etiqueta(&router, "Enfermería", "#2563eb").await;
    let et_cir = crear_etiqueta(&router, "Cirugía", "#dc2626").await;

    let box_enf = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "Box 1", "etiqueta_id": et_enf })),
    )
    .await
    .cuerpo["id"]
        .as_i64()
        .unwrap();
    let box_sin = crear_box(&router, "Box 2").await;

    let func_enf = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "E.U. Olivia", "etiqueta_id": et_enf })),
    )
    .await
    .cuerpo["id"]
        .as_i64()
        .unwrap();
    let func_cir = pedir(
        &router,
        "POST",
        "/funcionarios",
        Some(json!({ "nombre": "Dr. Soto", "etiqueta_id": et_cir })),
    )
    .await
    .cuerpo["id"]
        .as_i64()
        .unwrap();

    // Olivia (Enfermería) en Box 2 (sin etiqueta) el 2026-10-05 bloques 2 y 0.
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_sin, func_enf, "2026-10-05", 2)),
    )
    .await;
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_sin, func_enf, "2026-10-05", 0)),
    )
    .await;
    // Dr. Soto (Cirugía) en Box 1 (Enfermería) el 2026-10-05 bloque 1.
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_enf, func_cir, "2026-10-05", 1)),
    )
    .await;
    // Fuera del rango consultado.
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_sin, func_enf, "2026-11-10", 0)),
    )
    .await;

    // 1. Por funcionario: solo las 2 de Olivia en el rango, ordenadas por (fecha, box_id, bloque).
    let por_func = pedir(
        &router,
        "GET",
        &format!("/reservas/buscar?desde=2026-10-01&hasta=2026-10-31&funcionario_id={func_enf}"),
        None,
    )
    .await;
    assert_eq!(por_func.estado, StatusCode::OK);
    let arr = por_func.cuerpo.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["bloque"], 0);
    assert_eq!(arr[1]["bloque"], 2);

    // 2. Por etiqueta Enfermería: coincide con Olivia (por etiqueta del funcionario) y con
    //    Dr. Soto en Box 1 (por etiqueta del box). Orden: (fecha, box_id, bloque).
    let por_et = pedir(
        &router,
        "GET",
        &format!("/reservas/buscar?desde=2026-10-01&hasta=2026-10-31&etiqueta_id={et_enf}"),
        None,
    )
    .await;
    assert_eq!(por_et.estado, StatusCode::OK);
    let arr_et = por_et.cuerpo.as_array().unwrap();
    assert_eq!(arr_et.len(), 3);
    assert_eq!(arr_et[0]["box_id"], box_enf);
    assert_eq!(arr_et[1]["box_id"], box_sin);
    assert_eq!(arr_et[1]["bloque"], 0);
    assert_eq!(arr_et[2]["bloque"], 2);

    // 3. Por box: solo Box 1.
    let por_box = pedir(
        &router,
        "GET",
        &format!("/reservas/buscar?desde=2026-10-01&hasta=2026-10-31&box_id={box_enf}"),
        None,
    )
    .await;
    assert_eq!(por_box.estado, StatusCode::OK);
    assert_eq!(por_box.cuerpo.as_array().unwrap().len(), 1);

    // 4. Combinación AND (funcionario Dr. Soto + etiqueta Enfermería → coincide porque Box 1 es de Enfermería).
    let comb_ok = pedir(
        &router,
        "GET",
        &format!(
            "/reservas/buscar?desde=2026-10-01&hasta=2026-10-31&funcionario_id={func_cir}&etiqueta_id={et_enf}"
        ),
        None,
    )
    .await;
    assert_eq!(comb_ok.estado, StatusCode::OK);
    assert_eq!(comb_ok.cuerpo.as_array().unwrap().len(), 1);

    // 5. Combinación AND sin coincidencias (funcionario Olivia + Box 1).
    let comb_vacia = pedir(
        &router,
        "GET",
        &format!(
            "/reservas/buscar?desde=2026-10-01&hasta=2026-10-31&funcionario_id={func_enf}&box_id={box_enf}"
        ),
        None,
    )
    .await;
    assert_eq!(comb_vacia.estado, StatusCode::OK);
    assert_eq!(comb_vacia.cuerpo, json!([]));
}

#[tokio::test]
async fn buscar_reservas_validaciones_400() {
    let router = router_de_prueba();

    // Sin ningún filtro opcional → 400 con mensaje exacto.
    let sin_filtros = pedir(
        &router,
        "GET",
        "/reservas/buscar?desde=2026-10-01&hasta=2026-10-31",
        None,
    )
    .await;
    assert_eq!(sin_filtros.estado, StatusCode::BAD_REQUEST);
    assert_eq!(sin_filtros.cuerpo["codigo"], "datos_invalidos");
    assert_eq!(
        sin_filtros.cuerpo["mensaje"],
        "Indica al menos un funcionario, etiqueta o box"
    );

    // Sin fechas, rango invertido o > 366 días.
    for uri in [
        "/reservas/buscar?funcionario_id=1",
        "/reservas/buscar?desde=2026-10-10&hasta=2026-10-01&funcionario_id=1",
        "/reservas/buscar?desde=2026-01-01&hasta=2027-01-05&funcionario_id=1",
        "/reservas/buscar?desde=fecha-mala&hasta=2026-10-10&funcionario_id=1",
    ] {
        let r = pedir(&router, "GET", uri, None).await;
        assert_eq!(r.estado, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(r.cuerpo["codigo"], "datos_invalidos", "{uri}");
    }
}

#[tokio::test]
async fn boxes_libres_filtra_ocupados_inactivos_y_por_etiqueta() {
    let router = router_de_prueba();
    let et_enf = crear_etiqueta(&router, "Enfermería", "#2563eb").await;
    let et_cir = crear_etiqueta(&router, "Cirugía", "#dc2626").await;

    let box_1 = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "Box 1", "etiqueta_id": et_enf })),
    )
    .await
    .cuerpo["id"]
        .as_i64()
        .unwrap();
    let box_2 = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "Box 2", "etiqueta_id": et_enf })),
    )
    .await
    .cuerpo["id"]
        .as_i64()
        .unwrap();
    let box_3 = pedir(
        &router,
        "POST",
        "/boxes",
        Some(json!({ "nombre": "Box 3", "etiqueta_id": et_cir })),
    )
    .await
    .cuerpo["id"]
        .as_i64()
        .unwrap();
    let box_inactivo = crear_box(&router, "Box 4").await;
    pedir(
        &router,
        "PATCH",
        &format!("/boxes/{box_inactivo}"),
        Some(json!({ "activo": false })),
    )
    .await;

    let func = crear_funcionario(&router, "Ana").await;
    // Box 1 tiene ocupado solo el bloque 5 el 2026-10-06 (rango 0..=11 parcialmente ocupado).
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_1, func, "2026-10-06", 5)),
    )
    .await;
    // Box 2 tiene ocupado el bloque 15 (tarde), así que en la mañana 0..=11 está libre.
    pedir(
        &router,
        "POST",
        "/reservas",
        Some(reserva(box_2, func, "2026-10-06", 15)),
    )
    .await;

    // En 0..=11 sin filtro de etiqueta → libres Box 2 y Box 3 (Box 1 parcialmente ocupado, Box 4 inactivo).
    let libres = pedir(
        &router,
        "GET",
        "/boxes/libres?fecha=2026-10-06&desde=0&hasta=11",
        None,
    )
    .await;
    assert_eq!(libres.estado, StatusCode::OK);
    let ids: Vec<i64> = libres
        .cuerpo
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![box_2, box_3]);

    // Filtrando por etiqueta Enfermería → solo Box 2.
    let libres_enf = pedir(
        &router,
        "GET",
        &format!("/boxes/libres?fecha=2026-10-06&desde=0&hasta=11&etiqueta_id={et_enf}"),
        None,
    )
    .await;
    assert_eq!(libres_enf.estado, StatusCode::OK);
    let ids_enf: Vec<i64> = libres_enf
        .cuerpo
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids_enf, vec![box_2]);
}

#[tokio::test]
async fn boxes_libres_validaciones_400() {
    let router = router_de_prueba();

    for uri in [
        "/boxes/libres?fecha=2026-10-06&desde=10&hasta=5",
        "/boxes/libres?fecha=2026-10-06&desde=0&hasta=24",
        "/boxes/libres?desde=0&hasta=5",
        "/boxes/libres?fecha=no-es-fecha&desde=0&hasta=5",
    ] {
        let r = pedir(&router, "GET", uri, None).await;
        assert_eq!(r.estado, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(r.cuerpo["codigo"], "datos_invalidos", "{uri}");
    }
}
