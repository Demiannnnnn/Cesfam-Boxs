//! Estado compartido de la aplicación, inyectado en los handlers de Axum.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::servicio::ErrorServicio;

/// Datos del servidor que se informan en `/health` y en el descubrimiento UDP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoServidor {
    pub version: String,
    /// Nombre de equipo del PC servidor.
    pub equipo: String,
    pub puerto_http: u16,
}

impl InfoServidor {
    /// Información del proceso actual: versión del crate y nombre del equipo.
    pub fn actual(puerto_http: u16) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            equipo: nombre_equipo(),
            puerto_http,
        }
    }
}

/// Nombre del equipo, con alternativas si el sistema no lo informa.
fn nombre_equipo() -> String {
    hostname::get()
        .ok()
        .and_then(|nombre| nombre.into_string().ok())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .filter(|nombre| !nombre.is_empty())
        .unwrap_or_else(|| "servidor".to_owned())
}

/// Estado de Axum. Una sola conexión SQLite protegida por `Mutex` (ADR-010).
#[derive(Clone)]
pub struct AppState {
    db: Arc<Mutex<Connection>>,
    pub info: Arc<InfoServidor>,
}

impl AppState {
    pub fn new(conn: Connection, info: InfoServidor) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            info: Arc::new(info),
        }
    }

    /// Ejecuta `operacion` con la conexión en un hilo bloqueante, sin bloquear el runtime.
    ///
    /// Si un hilo anterior entró en pánico con el candado tomado, se recupera la conexión:
    /// SQLite mantiene la consistencia por sí mismo (transacciones), así que seguir es seguro
    /// y preferible a dejar la API inutilizable.
    pub async fn con_db<T, F>(&self, operacion: F) -> Result<T, ErrorServicio>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, ErrorServicio> + Send + 'static,
    {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || {
            let conn = db
                .lock()
                .unwrap_or_else(|envenenado| envenenado.into_inner());
            operacion(&conn)
        })
        .await
        .unwrap_or_else(|error| {
            tracing::error!(%error, "la tarea de base de datos terminó con pánico");
            Err(ErrorServicio::TareaFallida)
        })
    }
}
