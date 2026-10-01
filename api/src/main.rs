// En release y en Windows no se abre ventana de consola (RNF-02).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;

use gestion_box_api::{arranque, config::Config};

fn main() -> ExitCode {
    let config = match Config::desde_entorno() {
        Ok(config) => config,
        Err(error) => {
            // Aún no hay log: stderr lo captura NSSM (AppStderr) o la consola en desarrollo.
            eprintln!("Error de configuración: {error}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = arranque::preparar_entorno(&config) {
        eprintln!("No se pudo preparar la carpeta de datos: {error}");
        return ExitCode::FAILURE;
    }

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "no se pudo crear el runtime de tokio");
            return ExitCode::FAILURE;
        }
    };

    match runtime.block_on(arranque::ejecutar(config)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "la API terminó con error");
            eprintln!("La API terminó con error: {error}");
            ExitCode::FAILURE
        }
    }
}
