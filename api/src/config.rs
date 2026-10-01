//! Configuración de la API resuelta una sola vez al arrancar (RNF-03, RNF-04).
//!
//! Un servicio de Windows arranca con directorio de trabajo `C:\Windows\System32`,
//! por eso todas las rutas se resuelven a absolutas aquí y ningún otro módulo
//! lee variables de entorno.

use std::io;
use std::path::PathBuf;

use gestion_box_shared::descubrimiento::{PUERTO_HTTP_POR_DEFECTO, PUERTO_UDP_POR_DEFECTO};

/// Variable que sobrescribe la carpeta de datos (ADR-013).
pub const VAR_CARPETA_DATOS: &str = "GESTIONBOX_DATA_DIR";
/// Variable que sobrescribe el puerto HTTP.
pub const VAR_PUERTO_HTTP: &str = "GESTIONBOX_HTTP_PORT";
/// Variable que sobrescribe el puerto UDP.
pub const VAR_PUERTO_UDP: &str = "GESTIONBOX_UDP_PORT";

const NOMBRE_DB: &str = "gestion-box.db";
const NOMBRE_CARPETA_RESPALDOS: &str = "backups";
const NOMBRE_LOG: &str = "api.log";

/// Error al leer la configuración.
#[derive(Debug, thiserror::Error)]
pub enum ErrorConfig {
    #[error("el valor \"{valor}\" de {variable} no es un puerto válido (1-65535)")]
    PuertoInvalido {
        variable: &'static str,
        valor: String,
    },
    #[error("no se pudo resolver la ruta absoluta de la carpeta de datos: {0}")]
    RutaInvalida(#[source] io::Error),
}

/// Rutas absolutas y puertos de la API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub carpeta_datos: PathBuf,
    pub ruta_db: PathBuf,
    pub carpeta_respaldos: PathBuf,
    pub ruta_log: PathBuf,
    pub puerto_http: u16,
    pub puerto_udp: u16,
}

impl Config {
    /// Lee la configuración desde las variables de entorno del proceso.
    pub fn desde_entorno() -> Result<Self, ErrorConfig> {
        Self::resolver(|nombre| std::env::var(nombre).ok())
    }

    /// Resuelve la configuración con una función de entorno inyectable (testeable).
    pub fn resolver(entorno: impl Fn(&str) -> Option<String>) -> Result<Self, ErrorConfig> {
        let carpeta = entorno(VAR_CARPETA_DATOS)
            .filter(|valor| !valor.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| carpeta_por_defecto(&entorno));
        let carpeta = std::path::absolute(carpeta).map_err(ErrorConfig::RutaInvalida)?;

        let puerto_http = leer_puerto(&entorno, VAR_PUERTO_HTTP, PUERTO_HTTP_POR_DEFECTO)?;
        let puerto_udp = leer_puerto(&entorno, VAR_PUERTO_UDP, PUERTO_UDP_POR_DEFECTO)?;

        Ok(Self::con_carpeta(carpeta, puerto_http, puerto_udp))
    }

    /// Construye la configuración a partir de una carpeta de datos ya absoluta.
    pub fn con_carpeta(carpeta_datos: PathBuf, puerto_http: u16, puerto_udp: u16) -> Self {
        Self {
            ruta_db: carpeta_datos.join(NOMBRE_DB),
            carpeta_respaldos: carpeta_datos.join(NOMBRE_CARPETA_RESPALDOS),
            ruta_log: carpeta_datos.join(NOMBRE_LOG),
            carpeta_datos,
            puerto_http,
            puerto_udp,
        }
    }

    /// Crea la carpeta de datos y la de respaldos si no existen.
    pub fn crear_carpetas(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.carpeta_datos)?;
        std::fs::create_dir_all(&self.carpeta_respaldos)
    }
}

#[cfg(windows)]
fn carpeta_por_defecto(entorno: &impl Fn(&str) -> Option<String>) -> PathBuf {
    let program_data = entorno("ProgramData").unwrap_or_else(|| r"C:\ProgramData".to_owned());
    PathBuf::from(program_data).join("GestionBox")
}

#[cfg(not(windows))]
fn carpeta_por_defecto(_entorno: &impl Fn(&str) -> Option<String>) -> PathBuf {
    PathBuf::from("datos-dev")
}

fn leer_puerto(
    entorno: &impl Fn(&str) -> Option<String>,
    variable: &'static str,
    por_defecto: u16,
) -> Result<u16, ErrorConfig> {
    match entorno(variable) {
        None => Ok(por_defecto),
        Some(valor) => match valor.trim().parse::<u16>() {
            Ok(puerto) if puerto != 0 => Ok(puerto),
            _ => Err(ErrorConfig::PuertoInvalido { variable, valor }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn entorno(pares: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let mapa: HashMap<String, String> = pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |nombre| mapa.get(nombre).cloned()
    }

    #[test]
    fn carpeta_configurada_produce_rutas_absolutas() {
        let config = Config::resolver(entorno(&[(VAR_CARPETA_DATOS, "datos-prueba")])).unwrap();

        for ruta in [
            &config.carpeta_datos,
            &config.ruta_db,
            &config.carpeta_respaldos,
            &config.ruta_log,
        ] {
            assert!(ruta.is_absolute(), "{ruta:?} no es absoluta");
            assert!(ruta.starts_with(&config.carpeta_datos));
        }
        assert!(config.ruta_log.ends_with("api.log"));
    }

    #[test]
    fn puertos_por_defecto_son_8000_y_8001() {
        let config = Config::resolver(entorno(&[])).unwrap();

        assert_eq!(config.puerto_http, 8000);
        assert_eq!(config.puerto_udp, 8001);
    }

    #[test]
    fn puerto_invalido_produce_error_en_espanol() {
        let error = Config::resolver(entorno(&[(VAR_PUERTO_HTTP, "abc")])).unwrap_err();

        assert_eq!(
            error.to_string(),
            "el valor \"abc\" de GESTIONBOX_HTTP_PORT no es un puerto válido (1-65535)"
        );
    }

    #[test]
    fn puerto_cero_se_rechaza() {
        assert!(Config::resolver(entorno(&[(VAR_PUERTO_UDP, "0")])).is_err());
    }
}
