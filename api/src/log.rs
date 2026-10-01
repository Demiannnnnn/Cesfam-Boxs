//! Registro de eventos y errores en `api.log` (RNF-10, ADR-012).

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::sync::Mutex;

use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

/// Tamaño desde el cual `api.log` se rota al arrancar.
pub const TAMANO_MAXIMO_LOG: u64 = 10 * 1024 * 1024;
/// Nombre del log anterior tras rotar.
pub const NOMBRE_LOG_ANTERIOR: &str = "api.anterior.log";

/// Si `ruta_log` supera `limite` bytes, la renombra a `api.anterior.log`
/// (reemplazando el anterior). Devuelve `true` si rotó.
pub fn rotar_si_excede(ruta_log: &Path, limite: u64) -> io::Result<bool> {
    let tamano = match std::fs::metadata(ruta_log) {
        Ok(metadatos) => metadatos.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if tamano <= limite {
        return Ok(false);
    }
    let anterior = ruta_log.with_file_name(NOMBRE_LOG_ANTERIOR);
    std::fs::rename(ruta_log, anterior)?;
    Ok(true)
}

/// Inicia `tracing` hacia `ruta_log` (sin colores ANSI, hora local). En compilaciones
/// de desarrollo también escribe en la consola. Registra los pánicos antes de terminar.
///
/// Nivel por `RUST_LOG` (por defecto `info`).
pub fn iniciar(ruta_log: &Path) -> io::Result<()> {
    let rotado = rotar_si_excede(ruta_log, TAMANO_MAXIMO_LOG)?;
    let archivo: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ruta_log)?;

    let filtro = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formato_hora = || ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_owned());
    let capa_archivo = fmt::layer()
        .with_writer(Mutex::new(archivo))
        .with_ansi(false)
        .with_timer(formato_hora());
    let capa_consola = cfg!(debug_assertions).then(|| {
        fmt::layer()
            .with_writer(io::stderr)
            .with_timer(formato_hora())
    });

    tracing_subscriber::registry()
        .with(filtro)
        .with(capa_archivo)
        .with(capa_consola)
        .try_init()
        .map_err(io::Error::other)?;

    std::panic::set_hook(Box::new(|info| {
        tracing::error!(%info, "pánico: la API se detendrá y NSSM la reiniciará");
    }));

    if rotado {
        tracing::info!("api.log superaba el tamaño máximo y se rotó a {NOMBRE_LOG_ANTERIOR}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_grande_se_rota() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("api.log");
        std::fs::write(&ruta, vec![b'x'; 11]).unwrap();

        assert!(rotar_si_excede(&ruta, 10).unwrap());

        assert!(!ruta.exists());
        assert!(carpeta.path().join(NOMBRE_LOG_ANTERIOR).exists());
    }

    #[test]
    fn log_pequeno_o_inexistente_no_se_rota() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("api.log");

        assert!(!rotar_si_excede(&ruta, 10).unwrap());
        std::fs::write(&ruta, b"abc").unwrap();
        assert!(!rotar_si_excede(&ruta, 10).unwrap());
    }
}
