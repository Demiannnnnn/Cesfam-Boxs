//! Opciones avanzadas protegidas por clave (RF-19, ADR-022).
//!
//! Permite alternar entre modos de `escritura` y `lectura`, y en casos de desastre
//! donde el servidor anterior se dañó irreparablemente, promover un equipo de
//! escritura a nuevo servidor utilizando la copia más reciente de la base de datos.

use std::path::{Path, PathBuf};

use tauri::{Manager, State};

use crate::conexion::{Broadcast, Estrategia, Servidor};
use crate::config_local::ModoEquipo;
use crate::errores::ErrorUi;

/// Clave fija para acceder al menú de opciones avanzadas (ADR-022).
///
/// No constituye una barrera de seguridad criptográfica, sino una protección contra
/// cambios accidentales o por curiosidad de parte de los usuarios.
pub const CLAVE_AVANZADA: &str = "sum41rules";

/// Mensaje de error exacto cuando no se encuentran copias locales de respaldo (RF-19, ADR-022).
pub const MENSAJE_SIN_COPIAS: &str = "No hay copias de los datos en este equipo";

/// Comprueba si la clave ingresada coincide exactamente con la clave configurada (RF-19, ADR-022).
///
/// La comparación es estricta: distingue mayúsculas y no recorta espacios en blanco.
pub fn clave_correcta(clave: &str) -> bool {
    clave == CLAVE_AVANZADA
}

/// Comando Tauri para validar la clave de opciones avanzadas (RF-19, ADR-022).
#[tauri::command]
pub fn abrir_avanzado(clave: String) -> bool {
    clave_correcta(&clave)
}

/// Cambia el modo de operación de este equipo exclusivamente entre Escritura y Lectura (RF-19, ADR-022).
#[tauri::command]
pub fn cambiar_modo(
    estado: State<'_, crate::comandos::EstadoApp>,
    modo: ModoEquipo,
) -> Result<(), ErrorUi> {
    estado.cambiar_modo(modo)
}

/// Construye el mensaje de error exacto cuando se detecta un servidor activo en la red (RF-19, ADR-022).
pub fn mensaje_servidor_existente(servidor: &Servidor) -> String {
    let nombre = if servidor.equipo.trim().is_empty() {
        &servidor.direccion
    } else {
        &servidor.equipo
    };
    format!(
        "Hay un servidor funcionando en {nombre}. Esta opción es solo si el servidor no existe."
    )
}

/// Busca si existe algún servidor activo en la red antes de permitir la promoción (RF-19, ADR-022).
///
/// Consulta primero la dirección guardada en la configuración (si no es localhost) y luego
/// emite consultas mediante la estrategia de broadcast UDP.
pub async fn buscar_servidor_vivo(
    broadcast: &Broadcast,
    guardada: Option<&str>,
) -> Option<Servidor> {
    let local = format!(
        "127.0.0.1:{}",
        gestion_box_shared::descubrimiento::PUERTO_HTTP_POR_DEFECTO
    );
    if let Some(dir) = guardada.filter(|d| *d != local) {
        if let Some(servidor) = crate::conexion::confirmar(dir).await {
            return Some(servidor);
        }
    }
    broadcast.intentar().await
}

/// Valida los requisitos locales para la promoción a servidor: modo escritura y existencia de copias (RF-19, ADR-022).
pub fn validar_requisitos_promocion(
    modo: Option<ModoEquipo>,
    carpeta_copias: &Path,
) -> Result<PathBuf, ErrorUi> {
    if modo != Some(ModoEquipo::Escritura) {
        return Err(ErrorUi::datos_invalidos(
            "Solo un equipo de trabajo para reservar puede convertirse en servidor.",
        ));
    }
    crate::copia_diaria::copia_mas_reciente(carpeta_copias)
        .ok_or_else(|| ErrorUi::new("sin_copias", MENSAJE_SIN_COPIAS))
}

/// Promueve un equipo en modo escritura a servidor principal utilizando su copia más reciente (RF-19, ADR-022).
///
/// Sigue el orden estricto de comprobaciones:
/// 1. El modo actual del equipo debe ser `Escritura`.
/// 2. Busca si existe un servidor activo en la red (dirección guardada y broadcast UDP). Si hay uno, se niega.
/// 3. Comprueba la existencia de al menos una copia de respaldo en la carpeta local.
/// 4. Ejecuta la instalación del servicio con la copia más reciente.
#[tauri::command]
pub async fn promover_a_servidor(app: tauri::AppHandle) -> Result<(), ErrorUi> {
    let estado = app.state::<crate::comandos::EstadoApp>();
    let config = estado.config();

    // 1. El modo actual debe ser Escritura.
    if config.modo != Some(ModoEquipo::Escritura) {
        return Err(ErrorUi::datos_invalidos(
            "Solo un equipo de trabajo para reservar puede convertirse en servidor.",
        ));
    }

    // 2. Buscar servidor durante unos 3 s con Broadcast y confirmar guardada.
    let broadcast = Broadcast::default();
    if let Some(servidor) = buscar_servidor_vivo(&broadcast, config.servidor.as_deref()).await {
        return Err(ErrorUi::new(
            "servidor_existente",
            mensaje_servidor_existente(&servidor),
        ));
    }

    // 3. Comprobar copia local de respaldo.
    let carpeta_copias = estado.carpeta_copias();
    let ruta_copia = crate::copia_diaria::copia_mas_reciente(&carpeta_copias)
        .ok_or_else(|| ErrorUi::new("sin_copias", MENSAJE_SIN_COPIAS))?;

    // 4. Invocar la lógica de instalación del servicio con la copia de datos.
    let ruta_copia_str = ruta_copia.to_string_lossy().to_string();
    crate::servidor_local::instalar_servidor_logica(&app, Some(&ruta_copia_str)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, UdpSocket};

    #[test]
    fn v1_clave_correcta_abre_incorrecta_y_vacia_no() {
        // Clave exacta esperada
        assert!(clave_correcta(CLAVE_AVANZADA));
        assert!(clave_correcta("sum41rules"));

        // Distintas variaciones que deben rechazarse
        assert!(!clave_correcta("incorrecta"));
        assert!(!clave_correcta(""));
        assert!(!clave_correcta(" sum41rules"));
        assert!(!clave_correcta("sum41rules "));
        assert!(!clave_correcta(" sum41rules "));
        assert!(!clave_correcta("SUM41RULES"));
        assert!(!clave_correcta("Sum41rules"));
        assert!(!clave_correcta("sum41rule"));
        assert!(!clave_correcta("sum41rules!"));
    }

    #[test]
    fn v1_comando_abrir_avanzado() {
        assert!(abrir_avanzado("sum41rules".to_owned()));
        assert!(!abrir_avanzado("otra".to_owned()));
    }

    #[test]
    fn mensaje_servidor_existente_con_equipo_y_con_direccion() {
        let con_equipo = Servidor {
            direccion: "192.168.1.50:8000".to_owned(),
            equipo: "PC-SECRETARIA".to_owned(),
        };
        assert_eq!(
            mensaje_servidor_existente(&con_equipo),
            "Hay un servidor funcionando en PC-SECRETARIA. Esta opción es solo si el servidor no existe."
        );

        let sin_equipo = Servidor {
            direccion: "192.168.1.50:8000".to_owned(),
            equipo: "".to_owned(),
        };
        assert_eq!(
            mensaje_servidor_existente(&sin_equipo),
            "Hay un servidor funcionando en 192.168.1.50:8000. Esta opción es solo si el servidor no existe."
        );
    }

    #[test]
    fn validar_requisitos_promocion_rechaza_no_escritura_y_sin_copias() {
        let dir_temp = tempfile::tempdir().unwrap();
        let ruta_carpeta = dir_temp.path();

        // Modo None
        assert!(validar_requisitos_promocion(None, ruta_carpeta).is_err());

        // Modo Lectura
        assert!(validar_requisitos_promocion(Some(ModoEquipo::Lectura), ruta_carpeta).is_err());

        // Modo Servidor
        assert!(validar_requisitos_promocion(Some(ModoEquipo::Servidor), ruta_carpeta).is_err());

        // Modo Escritura pero carpeta vacía -> mensaje exacto sin copias
        let err =
            validar_requisitos_promocion(Some(ModoEquipo::Escritura), ruta_carpeta).unwrap_err();
        assert_eq!(err.mensaje, MENSAJE_SIN_COPIAS);
        assert_eq!(err.codigo, "sin_copias");

        // Con una copia válida presente
        let copia = ruta_carpeta.join("gestion-box-20261106-120000.db");
        std::fs::write(&copia, b"SQLite format 3\0").unwrap();

        let res = validar_requisitos_promocion(Some(ModoEquipo::Escritura), ruta_carpeta);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), copia);
    }

    #[tokio::test]
    async fn v2_buscar_servidor_vivo_por_guardada_devuelve_servidor_y_mensaje_exacto() {
        // Levantamos un listener HTTP mínimo que responda a /health con EstadoSalud
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let cuerpo = r#"{"estado":"ok","version":"0.1.0","equipo":"PC-BOX1"}"#;
                let respuesta = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    cuerpo.len(),
                    cuerpo
                );
                let _ = socket.write_all(respuesta.as_bytes()).await;
            }
        });

        let broadcast = Broadcast {
            destino: "127.0.0.1:9".parse().unwrap(),
            espera: Duration::from_millis(50),
            intentos: 1,
        };

        let direccion_guardada = addr.to_string();
        let servidor = buscar_servidor_vivo(&broadcast, Some(&direccion_guardada)).await;
        assert!(servidor.is_some());
        let s = servidor.unwrap();
        assert_eq!(s.equipo, "PC-BOX1");

        let mensaje = mensaje_servidor_existente(&s);
        assert_eq!(
            mensaje,
            "Hay un servidor funcionando en PC-BOX1. Esta opción es solo si el servidor no existe."
        );
    }

    #[tokio::test]
    async fn v2_buscar_servidor_vivo_por_udp_devuelve_servidor() {
        // Listener HTTP para /health
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let http_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let cuerpo = r#"{"estado":"ok","version":"0.1.0","equipo":"PC-SERVER-UDP"}"#;
                let respuesta = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    cuerpo.len(),
                    cuerpo
                );
                let _ = socket.write_all(respuesta.as_bytes()).await;
            }
        });

        // Socket UDP que simula al servidor respondiendo al mensaje de broadcast
        let udp_listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let udp_addr = udp_listener.local_addr().unwrap();

        let http_puerto = http_addr.port();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            if let Ok((_len, origen)) = udp_listener.recv_from(&mut buf).await {
                let resp = format!(
                    r#"{{"servicio":"gestionbox","version":"0.1.0","puerto_http":{http_puerto},"equipo":"PC-SERVER-UDP"}}"#
                );
                let _ = udp_listener.send_to(resp.as_bytes(), origen).await;
            }
        });

        let broadcast = Broadcast {
            destino: udp_addr,
            espera: Duration::from_millis(500),
            intentos: 2,
        };

        // Sin dirección guardada: el descubrimiento se realiza por UDP
        let servidor = buscar_servidor_vivo(&broadcast, None).await;
        assert!(servidor.is_some());
        let s = servidor.unwrap();
        assert_eq!(s.equipo, "PC-SERVER-UDP");

        let mensaje = mensaje_servidor_existente(&s);
        assert_eq!(
            mensaje,
            "Hay un servidor funcionando en PC-SERVER-UDP. Esta opción es solo si el servidor no existe."
        );
    }

    #[tokio::test]
    async fn v2_sin_servidor_y_sin_copias_error_es_exacto() {
        let dir_temp = tempfile::tempdir().unwrap();
        let carpeta_copias = dir_temp.path();

        // Broadcast a un puerto cerrado con espera muy corta
        let broadcast = Broadcast {
            destino: "127.0.0.1:9".parse().unwrap(),
            espera: Duration::from_millis(50),
            intentos: 1,
        };

        // Sin servidor vivo
        let servidor = buscar_servidor_vivo(&broadcast, None).await;
        assert_eq!(servidor, None);

        // Sin copias en la carpeta
        let res = validar_requisitos_promocion(Some(ModoEquipo::Escritura), carpeta_copias);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.mensaje, "No hay copias de los datos en este equipo");
    }
}
