//! Protocolo de descubrimiento del servidor por broadcast UDP (RF-09).
//!
//! 1. El cliente envía [`MENSAJE_BUSQUEDA`] a `255.255.255.255:PUERTO_UDP_POR_DEFECTO`.
//! 2. La API responde al origen con un JSON [`RespuestaDescubrimiento`].

use serde::{Deserialize, Serialize};

/// Datagrama exacto que envía el cliente.
pub const MENSAJE_BUSQUEDA: &[u8] = b"GESTIONBOX_BUSCAR";
/// Valor del campo `servicio` en la respuesta.
pub const NOMBRE_SERVICIO: &str = "gestionbox";
/// Puerto HTTP de la API (RNF-03).
pub const PUERTO_HTTP_POR_DEFECTO: u16 = 8000;
/// Puerto UDP de descubrimiento (RNF-03).
pub const PUERTO_UDP_POR_DEFECTO: u16 = 8001;

/// Respuesta del servidor a una búsqueda.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RespuestaDescubrimiento {
    pub servicio: String,
    pub version: String,
    pub puerto_http: u16,
    pub equipo: String,
}

impl RespuestaDescubrimiento {
    /// `true` si la respuesta proviene de una API de Gestión de Boxes.
    pub fn es_valida(&self) -> bool {
        self.servicio == NOMBRE_SERVICIO
    }
}
