use std::net::UdpSocket;
use std::time::Duration;

use gestion_box_api::descubrimiento;
use gestion_box_api::estado::InfoServidor;
use gestion_box_shared::descubrimiento::MENSAJE_BUSQUEDA;
use gestion_box_shared::RespuestaDescubrimiento;

#[test]
fn responde_a_busqueda_por_udp() {
    let servidor = UdpSocket::bind("127.0.0.1:0").unwrap();
    let direccion = servidor.local_addr().unwrap();
    let info = InfoServidor {
        version: "0.1.0".into(),
        equipo: "PC-SOME".into(),
        puerto_http: 8000,
    };
    descubrimiento::iniciar(servidor, descubrimiento::respuesta_para(&info)).unwrap();

    let cliente = UdpSocket::bind("127.0.0.1:0").unwrap();
    cliente
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    cliente.send_to(b"basura", direccion).unwrap();
    cliente.send_to(MENSAJE_BUSQUEDA, direccion).unwrap();

    let mut bufer = [0u8; 512];
    let (largo, origen) = cliente.recv_from(&mut bufer).unwrap();
    let respuesta: RespuestaDescubrimiento = serde_json::from_slice(&bufer[..largo]).unwrap();

    assert_eq!(origen, direccion);
    assert!(respuesta.es_valida());
    assert_eq!(respuesta.equipo, "PC-SOME");
}
