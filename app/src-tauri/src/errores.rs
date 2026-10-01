//! Traducción de errores a mensajes en español que dicen qué revisar (RNF-09).
//!
//! El detalle técnico va a la consola de depuración; nunca a la interfaz.

use gestion_box_shared::CodigoError;
use serde::Serialize;

use crate::cliente_api::ErrorCliente;

/// Nombre genérico cuando no se conoce el equipo servidor.
const EQUIPO_DESCONOCIDO: &str = "el PC servidor";

/// Error tal como lo recibe la interfaz: `{ codigo, mensaje }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorUi {
    /// Código de la API (`bloque_ocupado`, …) o local (`sin_conexion`, `nombre_no_resuelto`, `interno`).
    pub codigo: String,
    pub mensaje: String,
}

impl ErrorUi {
    pub fn new(codigo: &str, mensaje: impl Into<String>) -> Self {
        Self {
            codigo: codigo.to_owned(),
            mensaje: mensaje.into(),
        }
    }

    /// Error de datos ingresados en la interfaz.
    pub fn datos_invalidos(mensaje: impl Into<String>) -> Self {
        Self::new("datos_invalidos", mensaje)
    }

    /// Aún no hay servidor conectado.
    pub fn sin_servidor() -> Self {
        Self::new(
            "sin_conexion",
            "Todavía no hay conexión con el servidor. Ingresa el nombre del PC servidor.",
        )
    }
}

/// Mensaje para un servidor que no responde.
pub fn mensaje_sin_conexion(equipo: Option<&str>) -> String {
    let equipo = equipo.unwrap_or(EQUIPO_DESCONOCIDO);
    format!("No se pudo conectar con el servidor. Verifica que {equipo} esté encendido y conectado a la red.")
}

/// Mensaje cuando el descubrimiento automático no encontró nada.
pub const MENSAJE_NO_ENCONTRADO_EN_RED: &str = "No se encontró el servidor en la red. Verifica que el PC servidor esté encendido o ingresa su nombre manualmente.";

/// Convierte un error del cliente HTTP en un error para la interfaz.
///
/// `equipo` es el nombre del PC servidor si se conoce; `host` lo que se intentó resolver.
pub fn error_para_ui(error: &ErrorCliente, equipo: Option<&str>, host: Option<&str>) -> ErrorUi {
    eprintln!("[gestion-box] {error}");
    match error {
        ErrorCliente::SinConexion(_) => ErrorUi::new("sin_conexion", mensaje_sin_conexion(equipo)),
        ErrorCliente::NombreNoResuelto(_) => ErrorUi::new(
            "nombre_no_resuelto",
            format!(
                "No se encontró el equipo \"{}\". Revisa que el nombre esté bien escrito o usa su dirección IP.",
                host.or(equipo).unwrap_or(EQUIPO_DESCONOCIDO)
            ),
        ),
        ErrorCliente::Api(respuesta) => ErrorUi::new(
            codigo_texto(respuesta.codigo),
            mensaje_api(respuesta.codigo, &respuesta.mensaje),
        ),
        ErrorCliente::Inesperado(_) => ErrorUi::new(
            "interno",
            "Ocurrió un error en el servidor. Si se repite, avisa a TI y menciona la hora.",
        ),
    }
}

fn codigo_texto(codigo: CodigoError) -> &'static str {
    match codigo {
        CodigoError::DatosInvalidos => "datos_invalidos",
        CodigoError::NoEncontrado => "no_encontrado",
        CodigoError::BloqueOcupado => "bloque_ocupado",
        CodigoError::NombreDuplicado => "nombre_duplicado",
        CodigoError::BoxInactivo => "box_inactivo",
        CodigoError::ErrorInterno => "interno",
    }
}

/// Mensaje final para cada código de la API. Los que ya son claros se pasan tal cual.
fn mensaje_api(codigo: CodigoError, mensaje: &str) -> String {
    match codigo {
        CodigoError::BloqueOcupado => gestion_box_shared::error::MENSAJE_BLOQUE_OCUPADO.to_owned(),
        CodigoError::NombreDuplicado => "Ya existe un box con ese nombre.".to_owned(),
        CodigoError::BoxInactivo => {
            "Ese box está desactivado. Actívalo en Configuración de boxes.".to_owned()
        }
        CodigoError::ErrorInterno => {
            "Ocurrió un error en el servidor. Si se repite, avisa a TI y menciona la hora."
                .to_owned()
        }
        CodigoError::DatosInvalidos | CodigoError::NoEncontrado => mensaje.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gestion_box_shared::ErrorRespuesta;

    #[test]
    fn sin_conexion_nombra_el_equipo() {
        let error = ErrorCliente::SinConexion("connection refused".into());

        let ui = error_para_ui(&error, Some("PC-SOME"), None);

        assert_eq!(ui.codigo, "sin_conexion");
        assert_eq!(
            ui.mensaje,
            "No se pudo conectar con el servidor. Verifica que PC-SOME esté encendido y conectado a la red."
        );
    }

    #[test]
    fn sin_conexion_sin_equipo_conocido_usa_texto_generico() {
        let ui = error_para_ui(&ErrorCliente::SinConexion(String::new()), None, None);

        assert!(ui.mensaje.contains("el PC servidor"));
    }

    #[test]
    fn nombre_no_resuelto_menciona_lo_escrito() {
        let error = ErrorCliente::NombreNoResuelto("dns error".into());

        let ui = error_para_ui(&error, None, Some("PC-QUE-NO-EXISTE"));

        assert_eq!(ui.codigo, "nombre_no_resuelto");
        assert!(ui.mensaje.contains("\"PC-QUE-NO-EXISTE\""));
    }

    #[test]
    fn bloque_ocupado_usa_el_mensaje_exacto() {
        let error = ErrorCliente::Api(ErrorRespuesta::new(CodigoError::BloqueOcupado, "x"));

        let ui = error_para_ui(&error, None, None);

        assert_eq!(ui.codigo, "bloque_ocupado");
        assert_eq!(ui.mensaje, "Ese bloque acaba de ser tomado");
    }

    #[test]
    fn errores_inesperados_no_muestran_detalle_tecnico() {
        let error = ErrorCliente::Inesperado("HTTP 502 Bad Gateway".into());

        let ui = error_para_ui(&error, None, None);

        assert!(!ui.mensaje.contains("502"));
        assert!(ui.mensaje.contains("avisa a TI"));
    }

    #[test]
    fn no_encontrado_conserva_el_mensaje_de_la_api() {
        let error = ErrorCliente::Api(ErrorRespuesta::new(
            CodigoError::NoEncontrado,
            "No se encontró la reserva",
        ));

        assert_eq!(
            error_para_ui(&error, None, None).mensaje,
            "No se encontró la reserva"
        );
    }
}
