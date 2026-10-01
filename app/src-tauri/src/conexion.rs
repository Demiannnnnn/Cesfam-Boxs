//! Descubrimiento del servidor en capas (RF-09, RNF-08), como cadena de responsabilidad:
//! cada [`Estrategia`] intenta encontrar el servidor y, si falla, se pasa a la siguiente.
//!
//! Orden por defecto: dirección guardada → localhost (PC servidor) → broadcast UDP.
//! Si ninguna funciona, la interfaz pide el ingreso manual.

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::time::Duration;

use gestion_box_shared::descubrimiento::{
    MENSAJE_BUSQUEDA, PUERTO_HTTP_POR_DEFECTO, PUERTO_UDP_POR_DEFECTO,
};
use gestion_box_shared::RespuestaDescubrimiento;
use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::cliente_api::ClienteApi;

/// Espera por respuestas a cada broadcast.
pub const ESPERA_BROADCAST: Duration = Duration::from_millis(1500);
/// Cantidad de broadcasts antes de rendirse.
pub const INTENTOS_BROADCAST: u8 = 2;

/// Servidor encontrado y confirmado con `/health`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Servidor {
    /// `host:puerto`.
    pub direccion: String,
    /// Nombre de equipo informado por el servidor.
    pub equipo: String,
}

/// Futuro devuelto por las estrategias (los traits con `async fn` no admiten `dyn`).
pub type FuturoServidor<'a> = Pin<Box<dyn Future<Output = Option<Servidor>> + Send + 'a>>;

/// Una forma de encontrar el servidor.
pub trait Estrategia: Send + Sync {
    /// Nombre para el registro de depuración.
    fn nombre(&self) -> &str;
    /// Intenta encontrar y confirmar el servidor.
    fn intentar(&self) -> FuturoServidor<'_>;
}

/// Prueba las estrategias en orden y devuelve el primer servidor confirmado.
pub async fn descubrir(estrategias: &[Box<dyn Estrategia>]) -> Option<Servidor> {
    for estrategia in estrategias {
        if let Some(servidor) = estrategia.intentar().await {
            eprintln!(
                "[gestion-box] servidor encontrado por {}: {}",
                estrategia.nombre(),
                servidor.direccion
            );
            return Some(servidor);
        }
        eprintln!(
            "[gestion-box] {} no encontró el servidor",
            estrategia.nombre()
        );
    }
    None
}

/// Confirma que en `direccion` responde una API de Gestión de Boxes.
pub async fn confirmar(direccion: &str) -> Option<Servidor> {
    let cliente = ClienteApi::new(direccion).ok()?;
    let salud = cliente.salud().await.ok()?;
    Some(Servidor {
        direccion: direccion.to_owned(),
        equipo: salud.equipo,
    })
}

/// Cadena por defecto según la dirección guardada (si hay).
pub fn estrategias_por_defecto(guardada: Option<&str>) -> Vec<Box<dyn Estrategia>> {
    let local = format!("127.0.0.1:{PUERTO_HTTP_POR_DEFECTO}");
    let mut estrategias: Vec<Box<dyn Estrategia>> = Vec::new();
    if let Some(direccion) = guardada.filter(|d| *d != local) {
        estrategias.push(Box::new(DireccionFija::new(
            "dirección guardada",
            direccion,
        )));
    }
    estrategias.push(Box::new(DireccionFija::new("localhost", &local)));
    estrategias.push(Box::new(Broadcast::default()));
    estrategias
}

/// Prueba una dirección conocida con `/health`.
pub struct DireccionFija {
    nombre: String,
    direccion: String,
}

impl DireccionFija {
    pub fn new(nombre: &str, direccion: &str) -> Self {
        Self {
            nombre: nombre.to_owned(),
            direccion: direccion.to_owned(),
        }
    }
}

impl Estrategia for DireccionFija {
    fn nombre(&self) -> &str {
        &self.nombre
    }

    fn intentar(&self) -> FuturoServidor<'_> {
        Box::pin(confirmar(&self.direccion))
    }
}

/// Envía `GESTIONBOX_BUSCAR` por broadcast y confirma al primero que responde.
pub struct Broadcast {
    pub destino: SocketAddr,
    pub espera: Duration,
    pub intentos: u8,
}

impl Default for Broadcast {
    fn default() -> Self {
        Self {
            destino: SocketAddr::from((Ipv4Addr::BROADCAST, PUERTO_UDP_POR_DEFECTO)),
            espera: ESPERA_BROADCAST,
            intentos: INTENTOS_BROADCAST,
        }
    }
}

impl Broadcast {
    async fn buscar(&self) -> Option<Servidor> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.ok()?;
        socket.set_broadcast(true).ok()?;
        for _ in 0..self.intentos {
            if socket
                .send_to(MENSAJE_BUSQUEDA, self.destino)
                .await
                .is_err()
            {
                continue;
            }
            if let Ok(Some(servidor)) = timeout(self.espera, esperar_respuesta(&socket)).await {
                return Some(servidor);
            }
        }
        None
    }
}

impl Estrategia for Broadcast {
    fn nombre(&self) -> &str {
        "broadcast UDP"
    }

    fn intentar(&self) -> FuturoServidor<'_> {
        Box::pin(self.buscar())
    }
}

/// Lee respuestas hasta encontrar una válida que además responda `/health`.
async fn esperar_respuesta(socket: &UdpSocket) -> Option<Servidor> {
    let mut bufer = [0u8; 512];
    loop {
        let (largo, origen) = socket.recv_from(&mut bufer).await.ok()?;
        let Some(direccion) = direccion_desde_respuesta(&bufer[..largo], origen) else {
            continue;
        };
        if let Some(servidor) = confirmar(&direccion).await {
            return Some(servidor);
        }
    }
}

/// Lógica pura: dirección HTTP a partir de un datagrama de respuesta y su origen.
pub fn direccion_desde_respuesta(datagrama: &[u8], origen: SocketAddr) -> Option<String> {
    let respuesta: RespuestaDescubrimiento = serde_json::from_slice(datagrama).ok()?;
    respuesta
        .es_valida()
        .then(|| format!("{}:{}", origen.ip(), respuesta.puerto_http))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct Falsa {
        resultado: Option<Servidor>,
        llamadas: Arc<AtomicUsize>,
    }

    impl Estrategia for Falsa {
        fn nombre(&self) -> &str {
            "falsa"
        }

        fn intentar(&self) -> FuturoServidor<'_> {
            self.llamadas.fetch_add(1, Ordering::SeqCst);
            let resultado = self.resultado.clone();
            Box::pin(async move { resultado })
        }
    }

    fn servidor(direccion: &str) -> Servidor {
        Servidor {
            direccion: direccion.into(),
            equipo: "PC".into(),
        }
    }

    fn falsa(resultado: Option<Servidor>, llamadas: &Arc<AtomicUsize>) -> Box<dyn Estrategia> {
        Box::new(Falsa {
            resultado,
            llamadas: Arc::clone(llamadas),
        })
    }

    #[tokio::test]
    async fn cadena_devuelve_el_primer_exito_y_no_sigue() {
        let llamadas = Arc::new(AtomicUsize::new(0));
        let cadena = vec![
            falsa(None, &llamadas),
            falsa(Some(servidor("a:8000")), &llamadas),
            falsa(Some(servidor("b:8000")), &llamadas),
        ];

        let encontrado = descubrir(&cadena).await;

        assert_eq!(encontrado, Some(servidor("a:8000")));
        assert_eq!(llamadas.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn cadena_sin_exitos_devuelve_none() {
        let llamadas = Arc::new(AtomicUsize::new(0));
        let cadena = vec![falsa(None, &llamadas), falsa(None, &llamadas)];

        assert_eq!(descubrir(&cadena).await, None);
    }

    #[test]
    fn direccion_desde_respuesta_usa_ip_de_origen_y_puerto_informado() {
        let datagrama =
            br#"{"servicio":"gestionbox","version":"0.1.0","puerto_http":8000,"equipo":"PC-SOME"}"#;
        let origen: SocketAddr = "192.168.1.20:8001".parse().unwrap();

        assert_eq!(
            direccion_desde_respuesta(datagrama, origen).as_deref(),
            Some("192.168.1.20:8000")
        );
    }

    #[test]
    fn respuesta_de_otro_servicio_o_basura_se_ignora() {
        let origen: SocketAddr = "192.168.1.20:8001".parse().unwrap();
        let otro = br#"{"servicio":"otro","version":"1","puerto_http":80,"equipo":"x"}"#;

        assert_eq!(direccion_desde_respuesta(otro, origen), None);
        assert_eq!(direccion_desde_respuesta(b"hola", origen), None);
    }

    #[test]
    fn cadena_por_defecto_no_repite_localhost() {
        let con_guardada = estrategias_por_defecto(Some("PC-SOME:8000"));
        let con_local = estrategias_por_defecto(Some("127.0.0.1:8000"));

        assert_eq!(con_guardada.len(), 3);
        assert_eq!(con_local.len(), 2);
    }

    #[tokio::test]
    async fn broadcast_sin_servidor_termina_a_tiempo() {
        // Destino local sin nadie escuchando: debe rendirse tras intentos × espera.
        let broadcast = Broadcast {
            destino: "127.0.0.1:9".parse().unwrap(),
            espera: Duration::from_millis(100),
            intentos: 2,
        };

        let inicio = std::time::Instant::now();
        let resultado = broadcast.intentar().await;

        assert_eq!(resultado, None);
        assert!(inicio.elapsed() < Duration::from_secs(2));
    }
}
