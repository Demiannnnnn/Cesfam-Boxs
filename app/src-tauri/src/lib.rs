//! Backend de la app de escritorio. Toda la red (HTTP y UDP) vive aquí;
//! la interfaz solo invoca comandos (ADR-007).

pub mod cliente_api;
pub mod comandos;
pub mod conexion;
pub mod config_local;
pub mod errores;
pub mod exportar;

use tauri::Manager;

/// Nombre del archivo de configuración local (RF-10).
const ARCHIVO_CONFIG: &str = "cliente.json";

/// Arranca la aplicación Tauri.
pub fn ejecutar() {
    let resultado = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let carpeta = app.path().app_config_dir()?;
            let estado = comandos::EstadoApp::cargar(carpeta.join(ARCHIVO_CONFIG));
            app.manage(estado);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comandos::horario,
            comandos::preferencias,
            comandos::conectar,
            comandos::conectar_manual,
            comandos::estado_salud,
            comandos::listar_boxes,
            comandos::crear_box,
            comandos::actualizar_box,
            comandos::listar_reservas,
            comandos::crear_reserva,
            comandos::cancelar_reserva,
            comandos::exportar_csv,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = resultado {
        eprintln!("No se pudo iniciar Gestión de Boxes: {error}");
    }
}
