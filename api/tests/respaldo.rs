//! Tests de integración para `GET /respaldo` (RF-18, ADR-020, ADR-021).

mod comun;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use comun::{crear_box, crear_funcionario, pedir};
use gestion_box_api::db;
use gestion_box_api::estado::{AppState, InfoServidor};
use gestion_box_api::http::crear_router;
use http_body_util::BodyExt;
use rusqlite::Connection;
use tower::ServiceExt;

#[tokio::test]
async fn get_respaldo_entrega_base_sqlite_consistente_y_limpia_temporales() {
    let carpeta_respaldos = tempfile::tempdir().expect("tempdir respaldos");
    let conn = db::abrir_en_memoria().expect("base en memoria");
    let info = InfoServidor {
        version: "0.0.0-test".into(),
        equipo: "PC-PRUEBA".into(),
        puerto_http: 8000,
    };
    let estado =
        AppState::con_carpeta_respaldos(conn, info, carpeta_respaldos.path().to_path_buf());
    let router = crear_router(estado);

    // 1. Crear datos de prueba (box, funcionario, reserva)
    let box_id = crear_box(&router, "Box Respaldo").await;
    let funcionario_id = crear_funcionario(&router, "Dra. Respaldo").await;

    let r_reserva = pedir(
        &router,
        "POST",
        "/reservas",
        Some(serde_json::json!({
            "box_id": box_id,
            "fecha": "2026-11-06",
            "bloque": 2,
            "funcionario_id": funcionario_id,
            "motivo": "Consulta general"
        })),
    )
    .await;
    assert_eq!(r_reserva.estado, StatusCode::CREATED);

    // 2. Hacer GET /respaldo
    let peticion = Request::builder()
        .method("GET")
        .uri("/respaldo")
        .body(Body::empty())
        .expect("construir peticion");

    let respuesta = router.oneshot(peticion).await.expect("ejecutar oneshot");

    // Verificar status y cabeceras
    assert_eq!(respuesta.status(), StatusCode::OK);

    let content_type = respuesta
        .headers()
        .get(header::CONTENT_TYPE)
        .expect("header content-type")
        .to_str()
        .expect("to_str");
    assert_eq!(content_type, "application/octet-stream");

    let content_disposition = respuesta
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .expect("header content-disposition")
        .to_str()
        .expect("to_str");
    assert!(
        content_disposition.starts_with("attachment; filename=\"gestion-box-"),
        "Content-Disposition inesperado: {content_disposition}"
    );
    assert!(content_disposition.ends_with(".db\""));

    // 3. Verificar cuerpo
    let bytes = respuesta
        .into_body()
        .collect()
        .await
        .expect("leer bytes")
        .to_bytes();

    // Comienza con la cabecera mágica de SQLite
    assert!(
        bytes.starts_with(b"SQLite format 3\0"),
        "El archivo no comienza con la cabecera mágica de SQLite 3"
    );

    // 4. Escribir a un tempfile, abrirlo con rusqlite y verificar las reservas
    let temp_archivo = tempfile::NamedTempFile::new().expect("tempfile");
    std::fs::write(temp_archivo.path(), &bytes).expect("escribir bytes");

    let conn_copia = Connection::open(temp_archivo.path()).expect("abrir base copia");
    let cant_reservas: i64 = conn_copia
        .query_row("SELECT COUNT(*) FROM reserva", [], |f| f.get(0))
        .expect("contar reservas");
    assert_eq!(cant_reservas, 1);

    let motivo: String = conn_copia
        .query_row("SELECT motivo FROM reserva WHERE id = 1", [], |f| f.get(0))
        .expect("obtener motivo");
    assert_eq!(motivo, "Consulta general");

    // 5. Validar V2: la carpeta de respaldos no debe tener archivos .tmp tras la petición
    let temporales: Vec<_> = std::fs::read_dir(carpeta_respaldos.path())
        .expect("read_dir")
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .into_string()
                .map(|nombre| nombre.ends_with(".tmp"))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        temporales.is_empty(),
        "Quedaron archivos temporales en la carpeta de respaldos: {temporales:?}"
    );
}
