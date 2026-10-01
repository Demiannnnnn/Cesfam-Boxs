//! Listener UDP de descubrimiento en un hilo aparte (RF-09, RNF-03).
//!
//! Responde al datagrama `GESTIONBOX_BUSCAR` con un JSON
//! [`RespuestaDescubrimiento`]. Cualquier otro contenido se ignora para no
//! servir de amplificador de tráfico.

use std::io;
use std::net::UdpSocket;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use gestion_box_shared::descubrimiento::{MENSAJE_BUSQUEDA, NOMBRE_SERVICIO};
use gestion_box_shared::RespuestaDescubrimiento;

use crate::estado::InfoServidor;

/// Pausa tras un error de socket, para no girar en vacío si el error persiste.
const PAUSA_TRAS_ERROR: Duration = Duration::from_millis(200);

/// Enlaza el socket UDP en `0.0.0.0:puerto`.
pub fn enlazar(puerto: u16) -> io::Result<UdpSocket> {
    UdpSocket::bind(("0.0.0.0", puerto))
}

/// Respuesta que se envía a quien busca, a partir de la información del servidor.
pub fn respuesta_para(info: &InfoServidor) -> RespuestaDescubrimiento {
    RespuestaDescubrimiento {
        servicio: NOMBRE_SERVICIO.to_owned(),
        version: info.version.clone(),
        puerto_http: info.puerto_http,
        equipo: info.equipo.clone(),
    }
}

/// Lógica pura: qué responder a un datagrama. `None` = no responder.
pub fn responder(datagrama: &[u8], respuesta: &RespuestaDescubrimiento) -> Option<Vec<u8>> {
    if datagrama == MENSAJE_BUSQUEDA {
        serde_json::to_vec(respuesta).ok()
    } else {
        None
    }
}

/// Atiende búsquedas en un hilo de sistema dedicado, aislado del runtime HTTP.
/// Un error con un paquete nunca detiene el hilo.
pub fn iniciar(
    socket: UdpSocket,
    respuesta: RespuestaDescubrimiento,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("descubrimiento-udp".into())
        .spawn(move || atender(&socket, &respuesta))
}

fn atender(socket: &UdpSocket, respuesta: &RespuestaDescubrimiento) {
    let mut bufer = [0u8; 64];
    loop {
        let (largo, origen) = match socket.recv_from(&mut bufer) {
            Ok(recibido) => recibido,
            Err(error) => {
                // En Windows, un ICMP "puerto inalcanzable" de un envío anterior aparece
                // aquí como WSAECONNRESET; no es grave.
                tracing::debug!(%error, "error recibiendo datagrama de descubrimiento");
                thread::sleep(PAUSA_TRAS_ERROR);
                continue;
            }
        };
        let Some(datos) = responder(&bufer[..largo], respuesta) else {
            continue;
        };
        match socket.send_to(&datos, origen) {
            Ok(_) => tracing::debug!(%origen, "respuesta de descubrimiento enviada"),
            Err(error) => tracing::warn!(%origen, %error, "no se pudo responder al descubrimiento"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn respuesta() -> RespuestaDescubrimiento {
        respuesta_para(&InfoServidor {
            version: "0.1.0".into(),
            equipo: "PC-SOME".into(),
            puerto_http: 8000,
        })
    }

    #[test]
    fn responde_solo_al_mensaje_exacto() {
        assert!(responder(b"GESTIONBOX_BUSCAR", &respuesta()).is_some());
        assert!(responder(b"GESTIONBOX_BUSCAR\n", &respuesta()).is_none());
        assert!(responder(b"hola", &respuesta()).is_none());
        assert!(responder(b"", &respuesta()).is_none());
    }

    #[test]
    fn respuesta_es_json_valido() {
        let datos = responder(MENSAJE_BUSQUEDA, &respuesta()).unwrap();

        let decodificada: RespuestaDescubrimiento = serde_json::from_slice(&datos).unwrap();

        assert!(decodificada.es_valida());
        assert_eq!(decodificada.puerto_http, 8000);
        assert_eq!(decodificada.equipo, "PC-SOME");
    }
}
