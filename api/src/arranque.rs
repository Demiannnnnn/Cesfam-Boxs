//! Secuencia de arranque de la API (RNF-01 – RNF-04, RF-11).
//!
//! Orden: carpetas → log → base + migraciones → respaldo → UDP → HTTP.
//! Fallas no críticas (respaldo, UDP) se registran y la API sigue: disponibilidad
//! primero, porque nadie va a mantener el sistema.

use std::io;

use tokio::net::TcpListener;

use crate::config::Config;
use crate::estado::{AppState, InfoServidor};
use crate::{db, descubrimiento, http, log, respaldo};

/// Error fatal de arranque.
#[derive(Debug, thiserror::Error)]
pub enum ErrorArranque {
    #[error("no se pudo abrir la base de datos {ruta}: {error}")]
    BaseDeDatos {
        ruta: String,
        error: rusqlite::Error,
    },
    #[error("no se pudo escuchar en el puerto HTTP {puerto} (¿otro programa lo usa?): {error}")]
    PuertoHttp { puerto: u16, error: io::Error },
    #[error("el servidor HTTP se detuvo con error: {0}")]
    Servidor(io::Error),
}

/// Crea las carpetas de datos e inicia el log. Va antes del runtime para que
/// cualquier error posterior quede registrado.
pub fn preparar_entorno(config: &Config) -> io::Result<()> {
    config.crear_carpetas()?;
    log::iniciar(&config.ruta_log)
}

/// Ejecuta la API hasta recibir Ctrl+C (o la señal de detención de NSSM).
pub async fn ejecutar(config: Config) -> Result<(), ErrorArranque> {
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        carpeta = %config.carpeta_datos.display(),
        puerto_http = config.puerto_http,
        puerto_udp = config.puerto_udp,
        "iniciando API de Gestión de Boxes"
    );

    let conn = db::abrir(&config.ruta_db).map_err(|error| ErrorArranque::BaseDeDatos {
        ruta: config.ruta_db.display().to_string(),
        error,
    })?;

    respaldar_al_iniciar(&conn, &config);

    let info = InfoServidor::actual(config.puerto_http);
    iniciar_descubrimiento(&config, &info);

    let estado = AppState::new(conn, info);
    let direccion = ("0.0.0.0", config.puerto_http);
    let listener =
        TcpListener::bind(direccion)
            .await
            .map_err(|error| ErrorArranque::PuertoHttp {
                puerto: config.puerto_http,
                error,
            })?;
    tracing::info!(puerto = config.puerto_http, "API escuchando");

    axum::serve(listener, http::crear_router(estado))
        .with_graceful_shutdown(senal_de_detencion())
        .await
        .map_err(ErrorArranque::Servidor)?;

    tracing::info!("API detenida");
    Ok(())
}

fn respaldar_al_iniciar(conn: &rusqlite::Connection, config: &Config) {
    let ahora = chrono::Local::now().naive_local();
    match respaldo::respaldar(conn, &config.carpeta_respaldos, ahora) {
        Ok(ruta) => tracing::info!(ruta = %ruta.display(), "respaldo creado"),
        Err(error) => tracing::error!(%error, "no se pudo crear el respaldo; la API sigue"),
    }
    match respaldo::rotar(&config.carpeta_respaldos, respaldo::RESPALDOS_A_CONSERVAR) {
        Ok(0) => {}
        Ok(borrados) => tracing::info!(borrados, "respaldos antiguos eliminados"),
        Err(error) => tracing::error!(%error, "no se pudieron rotar los respaldos"),
    }
}

fn iniciar_descubrimiento(config: &Config, info: &InfoServidor) {
    let respuesta = descubrimiento::respuesta_para(info);
    let resultado = descubrimiento::enlazar(config.puerto_udp)
        .and_then(|socket| descubrimiento::iniciar(socket, respuesta));
    match resultado {
        Ok(_) => tracing::info!(puerto = config.puerto_udp, "descubrimiento UDP activo"),
        Err(error) => tracing::error!(
            puerto = config.puerto_udp,
            %error,
            "no se pudo iniciar el descubrimiento UDP; las apps deberán usar el nombre o la IP"
        ),
    }
}

async fn senal_de_detencion() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "no se pudo escuchar la señal de detención");
        std::future::pending::<()>().await;
    }
    tracing::info!("señal de detención recibida");
}
