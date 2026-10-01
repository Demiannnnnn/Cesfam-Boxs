//! Cliente HTTP tipado de la API, con timeouts explícitos (RF-03 – RF-08).

use std::error::Error as _;
use std::time::Duration;

use gestion_box_shared::descubrimiento::PUERTO_HTTP_POR_DEFECTO;
use gestion_box_shared::{
    BoxAtencion, CambiosBox, ErrorRespuesta, EstadoSalud, Fecha, NuevaReserva, NuevoBox, Reserva,
};
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;

/// Tiempo máximo para establecer la conexión TCP.
pub const TIMEOUT_CONEXION: Duration = Duration::from_millis(1500);
/// Tiempo máximo total de una petición.
pub const TIMEOUT_PETICION: Duration = Duration::from_secs(5);

/// Error al hablar con la API.
#[derive(Debug, thiserror::Error)]
pub enum ErrorCliente {
    /// El servidor no respondió (apagado, firewall, sin red, tiempo agotado).
    #[error("sin conexión con el servidor: {0}")]
    SinConexion(String),
    /// El nombre del equipo no se pudo resolver a una IP.
    #[error("no se pudo resolver el nombre del servidor: {0}")]
    NombreNoResuelto(String),
    /// La API respondió con un error de negocio (409, 404, …).
    #[error("la API respondió {0:?}")]
    Api(ErrorRespuesta),
    /// Respuesta que no entendemos (versión distinta, error interno sin cuerpo, …).
    #[error("respuesta inesperada: {0}")]
    Inesperado(String),
}

impl From<reqwest::Error> for ErrorCliente {
    fn from(error: reqwest::Error) -> Self {
        let detalle = cadena_de_errores(&error);
        if es_error_dns(&detalle) {
            ErrorCliente::NombreNoResuelto(detalle)
        } else if error.is_connect() || error.is_timeout() || error.is_request() {
            ErrorCliente::SinConexion(detalle)
        } else {
            ErrorCliente::Inesperado(detalle)
        }
    }
}

/// Concatena el error y todas sus causas, para clasificar y para el log.
fn cadena_de_errores(error: &reqwest::Error) -> String {
    let mut texto = error.to_string();
    let mut causa = error.source();
    while let Some(actual) = causa {
        texto.push_str(": ");
        texto.push_str(&actual.to_string());
        causa = actual.source();
    }
    texto
}

/// reqwest no expone un tipo para fallas de DNS; se reconocen por el texto de la causa.
fn es_error_dns(detalle: &str) -> bool {
    let minusculas = detalle.to_lowercase();
    [
        "dns error",
        "failed to lookup",
        "no such host",
        "name or service not known",
    ]
    .iter()
    .any(|patron| minusculas.contains(patron))
}

/// Normaliza lo que escribe el usuario a `host:puerto`.
///
/// `"http://PC-SOME/"` → `"PC-SOME:8000"`, `"10.0.0.5:9000"` → `"10.0.0.5:9000"`.
/// Devuelve `None` si queda vacío.
pub fn normalizar_direccion(entrada: &str) -> Option<String> {
    let sin_esquema = entrada
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host_puerto = sin_esquema.split('/').next().unwrap_or("").trim();
    if host_puerto.is_empty() || host_puerto.contains(char::is_whitespace) {
        return None;
    }
    let tiene_puerto = host_puerto
        .rsplit_once(':')
        .is_some_and(|(_, puerto)| puerto.parse::<u16>().is_ok());
    if tiene_puerto {
        Some(host_puerto.to_owned())
    } else {
        Some(format!("{host_puerto}:{PUERTO_HTTP_POR_DEFECTO}"))
    }
}

/// Parte del host de una dirección `host:puerto` (para mostrar "PC-SOME").
pub fn host_de(direccion: &str) -> &str {
    direccion
        .rsplit_once(':')
        .map_or(direccion, |(host, _)| host)
}

/// Cliente de la API apuntando a un servidor concreto. Barato de clonar.
#[derive(Debug, Clone)]
pub struct ClienteApi {
    base: String,
    direccion: String,
    http: Client,
}

impl ClienteApi {
    /// Crea un cliente para `direccion` (`host:puerto`).
    pub fn new(direccion: &str) -> Result<Self, ErrorCliente> {
        let http = Client::builder()
            .connect_timeout(TIMEOUT_CONEXION)
            .timeout(TIMEOUT_PETICION)
            // Un proxy corporativo configurado en Windows no debe interceptar tráfico de la LAN.
            .no_proxy()
            .build()
            .map_err(|e| ErrorCliente::Inesperado(e.to_string()))?;
        Ok(Self {
            base: format!("http://{direccion}"),
            direccion: direccion.to_owned(),
            http,
        })
    }

    /// Dirección `host:puerto` del servidor.
    pub fn direccion(&self) -> &str {
        &self.direccion
    }

    fn url(&self, ruta: &str) -> String {
        format!("{}{ruta}", self.base)
    }

    async fn enviar(peticion: RequestBuilder) -> Result<Response, ErrorCliente> {
        let respuesta = peticion.send().await?;
        if respuesta.status().is_success() {
            return Ok(respuesta);
        }
        let estado = respuesta.status();
        match respuesta.json::<ErrorRespuesta>().await {
            Ok(cuerpo) => Err(ErrorCliente::Api(cuerpo)),
            Err(_) => Err(ErrorCliente::Inesperado(format!("HTTP {estado}"))),
        }
    }

    async fn json<T: DeserializeOwned>(peticion: RequestBuilder) -> Result<T, ErrorCliente> {
        Self::enviar(peticion)
            .await?
            .json::<T>()
            .await
            .map_err(|e| ErrorCliente::Inesperado(e.to_string()))
    }

    /// `GET /health` (RF-08).
    pub async fn salud(&self) -> Result<EstadoSalud, ErrorCliente> {
        Self::json(self.http.get(self.url("/health"))).await
    }

    /// `GET /boxes`.
    pub async fn listar_boxes(
        &self,
        incluir_inactivos: bool,
    ) -> Result<Vec<BoxAtencion>, ErrorCliente> {
        let peticion = self
            .http
            .get(self.url("/boxes"))
            .query(&[("incluir_inactivos", incluir_inactivos)]);
        Self::json(peticion).await
    }

    /// `POST /boxes`.
    pub async fn crear_box(&self, nuevo: &NuevoBox) -> Result<BoxAtencion, ErrorCliente> {
        Self::json(self.http.post(self.url("/boxes")).json(nuevo)).await
    }

    /// `PATCH /boxes/{id}`.
    pub async fn actualizar_box(
        &self,
        id: i64,
        cambios: &CambiosBox,
    ) -> Result<BoxAtencion, ErrorCliente> {
        Self::json(
            self.http
                .patch(self.url(&format!("/boxes/{id}")))
                .json(cambios),
        )
        .await
    }

    /// `GET /reservas?fecha=`.
    pub async fn reservas_del_dia(&self, fecha: Fecha) -> Result<Vec<Reserva>, ErrorCliente> {
        let peticion = self
            .http
            .get(self.url("/reservas"))
            .query(&[("fecha", fecha.to_string())]);
        Self::json(peticion).await
    }

    /// `GET /reservas?desde=&hasta=` (RF-12).
    pub async fn reservas_del_rango(
        &self,
        desde: Fecha,
        hasta: Fecha,
    ) -> Result<Vec<Reserva>, ErrorCliente> {
        let peticion = self
            .http
            .get(self.url("/reservas"))
            .query(&[("desde", desde.to_string()), ("hasta", hasta.to_string())]);
        Self::json(peticion).await
    }

    /// `POST /reservas` (RF-03, RF-05).
    pub async fn crear_reserva(&self, nueva: &NuevaReserva) -> Result<Reserva, ErrorCliente> {
        Self::json(self.http.post(self.url("/reservas")).json(nueva)).await
    }

    /// `DELETE /reservas/{id}` (RF-04).
    pub async fn cancelar_reserva(&self, id: i64) -> Result<(), ErrorCliente> {
        Self::enviar(self.http.delete(self.url(&format!("/reservas/{id}"))))
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_direcciones_escritas_por_el_usuario() {
        let casos = [
            ("PC-SOME", Some("PC-SOME:8000")),
            ("  pc-some  ", Some("pc-some:8000")),
            ("http://PC-SOME/", Some("PC-SOME:8000")),
            ("http://10.0.0.5:9000/health", Some("10.0.0.5:9000")),
            ("10.0.0.5:9000", Some("10.0.0.5:9000")),
            ("localhost", Some("localhost:8000")),
            ("", None),
            ("   ", None),
            ("PC SOME", None),
        ];
        for (entrada, esperado) in casos {
            assert_eq!(
                normalizar_direccion(entrada).as_deref(),
                esperado,
                "{entrada:?}"
            );
        }
    }

    #[test]
    fn host_de_quita_el_puerto() {
        assert_eq!(host_de("PC-SOME:8000"), "PC-SOME");
        assert_eq!(host_de("PC-SOME"), "PC-SOME");
    }

    #[test]
    fn reconoce_errores_dns() {
        assert!(es_error_dns(
            "error sending request: dns error: failed to lookup address"
        ));
        assert!(!es_error_dns("error sending request: connection refused"));
    }

    #[tokio::test]
    async fn nombre_inexistente_es_nombre_no_resuelto() {
        // El dominio `.invalid` está reservado (RFC 6761): nunca resuelve.
        let cliente = ClienteApi::new("pc-que-no-existe.invalid:8000").unwrap();

        let error = cliente.salud().await.unwrap_err();

        assert!(
            matches!(error, ErrorCliente::NombreNoResuelto(_)),
            "{error:?}"
        );
    }

    #[tokio::test]
    async fn puerto_cerrado_es_sin_conexion() {
        // Puerto 1 en localhost: nadie escucha, la conexión se rechaza de inmediato.
        let cliente = ClienteApi::new("127.0.0.1:1").unwrap();

        let error = cliente.salud().await.unwrap_err();

        assert!(matches!(error, ErrorCliente::SinConexion(_)), "{error:?}");
    }
}
