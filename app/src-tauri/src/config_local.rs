//! Configuración local del PC cliente (RF-10): última dirección válida del servidor.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Contenido de `cliente.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigLocal {
    /// Última dirección válida, `host:puerto`.
    pub servidor: Option<String>,
    /// Nombre de equipo del servidor, para los mensajes de error.
    pub equipo_servidor: Option<String>,
    /// Identificador del último funcionario usado al reservar desde este PC
    /// (comodidad, no identidad). Cubre RF-10 y RF-13.
    pub ultimo_funcionario_id: Option<i64>,
}

impl ConfigLocal {
    /// Lee la configuración. Si no existe o está dañada, devuelve valores por defecto:
    /// la app nunca debe dejar de abrir por un archivo de configuración.
    pub fn cargar(ruta: &Path) -> Self {
        match std::fs::read_to_string(ruta) {
            Ok(texto) => serde_json::from_str(&texto).unwrap_or_else(|error| {
                eprintln!("cliente.json dañado, se usarán valores por defecto: {error}");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Guarda de forma atómica: escribe un temporal y lo renombra, para no dejar
    /// un archivo a medias si se corta la luz.
    pub fn guardar(&self, ruta: &Path) -> io::Result<()> {
        if let Some(carpeta) = ruta.parent() {
            std::fs::create_dir_all(carpeta)?;
        }
        let temporal = ruta.with_extension("json.tmp");
        let texto = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        std::fs::write(&temporal, texto)?;
        std::fs::rename(&temporal, ruta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardar_y_cargar_ida_y_vuelta() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("sub").join("cliente.json");
        let config = ConfigLocal {
            servidor: Some("PC-SOME:8000".into()),
            equipo_servidor: Some("PC-SOME".into()),
            ultimo_funcionario_id: Some(7),
        };

        config.guardar(&ruta).unwrap();

        assert_eq!(ConfigLocal::cargar(&ruta), config);
    }

    #[test]
    fn archivo_inexistente_o_corrupto_da_valores_por_defecto() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");

        assert_eq!(ConfigLocal::cargar(&ruta), ConfigLocal::default());
        std::fs::write(&ruta, "{ esto no es json").unwrap();
        assert_eq!(ConfigLocal::cargar(&ruta), ConfigLocal::default());
    }

    #[test]
    fn campos_faltantes_y_campo_viejo_de_texto_se_aceptan_sin_fallar() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");
        std::fs::write(
            &ruta,
            r#"{"servidor":"10.0.0.5:8000","ultimo_funcionario":"María Pérez"}"#,
        )
        .unwrap();

        let config = ConfigLocal::cargar(&ruta);

        assert_eq!(config.servidor.as_deref(), Some("10.0.0.5:8000"));
        assert_eq!(config.ultimo_funcionario_id, None);
    }
}
