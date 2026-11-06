//! Configuración local del PC cliente (RF-10, RF-16, RF-17): última dirección
//! válida del servidor y modo de operación del equipo.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Modo de operación de la aplicación en este equipo (ADR-020). Cubre RF-16 y RF-17.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModoEquipo {
    /// Servidor principal con API y base de datos SQLite.
    Servidor,
    /// Equipo de trabajo para reservar y cancelar con copia diaria de respaldo.
    Escritura,
    /// Terminal de solo consulta para ver grilla y buscar.
    Lectura,
}

/// Contenido de `cliente.json`. Cubre RF-10, RF-16 y RF-17.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigLocal {
    /// Modo de operación de este equipo (`servidor`, `escritura` o `lectura`).
    pub modo: Option<ModoEquipo>,
    /// Última dirección válida, `host:puerto`.
    pub servidor: Option<String>,
    /// Nombre de equipo del servidor, para los mensajes de error.
    pub equipo_servidor: Option<String>,
    /// Identificador del último funcionario usado al reservar desde este PC
    /// (comodidad, no identidad). Cubre RF-10 y RF-13.
    pub ultimo_funcionario_id: Option<i64>,
    /// Fecha y hora de la última copia de respaldo descargada (RF-18, ADR-021). Formato `%Y-%m-%dT%H:%M:%S`.
    pub ultima_copia: Option<String>,
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

    /// Fija el modo del equipo por primera vez. Falla si ya tiene un modo asignado (RF-16).
    pub fn fijar_modo_inicial(&mut self, modo: ModoEquipo) -> Result<(), &'static str> {
        if self.modo.is_some() {
            return Err("El modo del equipo ya fue configurado.");
        }
        self.modo = Some(modo);
        Ok(())
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
            modo: Some(ModoEquipo::Escritura),
            servidor: Some("PC-SOME:8000".into()),
            equipo_servidor: Some("PC-SOME".into()),
            ultimo_funcionario_id: Some(7),
            ultima_copia: Some("2026-11-06T12:00:00".into()),
        };

        config.guardar(&ruta).unwrap();

        assert_eq!(ConfigLocal::cargar(&ruta), config);
    }

    #[test]
    fn ida_y_vuelta_de_los_tres_modos() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");

        for (modo, texto_esperado) in [
            (ModoEquipo::Servidor, "\"servidor\""),
            (ModoEquipo::Escritura, "\"escritura\""),
            (ModoEquipo::Lectura, "\"lectura\""),
        ] {
            let config = ConfigLocal {
                modo: Some(modo),
                ..ConfigLocal::default()
            };
            config.guardar(&ruta).unwrap();

            let json = std::fs::read_to_string(&ruta).unwrap();
            assert!(
                json.contains(texto_esperado),
                "JSON no contiene el modo serializado en minúsculas: {json}"
            );

            let cargada = ConfigLocal::cargar(&ruta);
            assert_eq!(cargada.modo, Some(modo));
        }
    }

    #[test]
    fn cliente_json_antiguo_sin_modo_carga_none_sin_perder_otros_campos() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");
        std::fs::write(
            &ruta,
            r#"{
                "servidor": "PC-SOME:8000",
                "equipo_servidor": "PC-SOME",
                "ultimo_funcionario_id": 7
            }"#,
        )
        .unwrap();

        let config = ConfigLocal::cargar(&ruta);

        assert_eq!(config.modo, None);
        assert_eq!(config.servidor.as_deref(), Some("PC-SOME:8000"));
        assert_eq!(config.equipo_servidor.as_deref(), Some("PC-SOME"));
        assert_eq!(config.ultimo_funcionario_id, Some(7));
        assert_eq!(config.ultima_copia, None);
    }

    #[test]
    fn cliente_json_antiguo_sin_ultima_copia_carga_none() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");
        std::fs::write(
            &ruta,
            r#"{
                "modo": "escritura",
                "servidor": "PC-SOME:8000",
                "equipo_servidor": "PC-SOME",
                "ultimo_funcionario_id": 7
            }"#,
        )
        .unwrap();

        let config = ConfigLocal::cargar(&ruta);

        assert_eq!(config.modo, Some(ModoEquipo::Escritura));
        assert_eq!(config.servidor.as_deref(), Some("PC-SOME:8000"));
        assert_eq!(config.equipo_servidor.as_deref(), Some("PC-SOME"));
        assert_eq!(config.ultimo_funcionario_id, Some(7));
        assert_eq!(config.ultima_copia, None);
    }

    #[test]
    fn fijar_modo_inicial_con_none_guarda_y_con_some_falla() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");

        let mut config = ConfigLocal {
            modo: None,
            servidor: Some("PC-SOME:8000".into()),
            ..ConfigLocal::default()
        };

        // None -> ok y se guarda
        assert!(config.fijar_modo_inicial(ModoEquipo::Escritura).is_ok());
        assert_eq!(config.modo, Some(ModoEquipo::Escritura));
        config.guardar(&ruta).unwrap();

        let cargada = ConfigLocal::cargar(&ruta);
        assert_eq!(cargada.modo, Some(ModoEquipo::Escritura));
        assert_eq!(cargada.servidor.as_deref(), Some("PC-SOME:8000"));

        // Some -> error y no cambia
        let res = config.fijar_modo_inicial(ModoEquipo::Lectura);
        assert!(res.is_err());
        assert_eq!(config.modo, Some(ModoEquipo::Escritura));

        // El archivo en disco tampoco cambió
        let cargada_despues = ConfigLocal::cargar(&ruta);
        assert_eq!(cargada_despues.modo, Some(ModoEquipo::Escritura));
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

        assert_eq!(config.modo, None);
        assert_eq!(config.servidor.as_deref(), Some("10.0.0.5:8000"));
        assert_eq!(config.ultimo_funcionario_id, None);
    }
}
