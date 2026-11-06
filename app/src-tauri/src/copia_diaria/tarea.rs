//! Tarea en segundo plano para la copia periódica del servidor (RF-18, ADR-021).

use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono::NaiveDateTime;

use super::{debe_copiar, guardar_copia, rotar, COPIAS_A_CONSERVAR};

/// Guardia para asegurar que la bandera de descarga activa vuelva a `false` al terminar.
struct DescargaGuard<'a>(&'a std::sync::atomic::AtomicBool);

impl<'a> Drop for DescargaGuard<'a> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Realiza un intento de descarga y guardado de la copia diaria si corresponde (RF-18, ADR-021).
pub async fn realizar_intento_copia(
    estado: &crate::comandos::EstadoApp,
    cliente: &crate::cliente_api::ClienteApi,
) {
    let config = estado.config();
    let ahora = chrono::Local::now().naive_local();
    let ultima = config
        .ultima_copia
        .as_deref()
        .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());

    if !debe_copiar(config.modo, ultima, ahora) {
        return;
    }

    let en_descarga = estado.en_descarga();
    if en_descarga
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    let _guard = DescargaGuard(&en_descarga);

    match cliente.descargar_respaldo().await {
        Ok(bytes) => {
            let carpeta = estado.carpeta_copias();
            match guardar_copia(&carpeta, &bytes, ahora) {
                Ok(_ruta) => {
                    let ahora_str = ahora.format("%Y-%m-%dT%H:%M:%S").to_string();
                    estado.modificar_config(|c| {
                        c.ultima_copia = Some(ahora_str);
                    });
                    if let Err(error) = rotar(&carpeta, COPIAS_A_CONSERVAR) {
                        eprintln!("[gestion-box] error al rotar copias de respaldo: {error}");
                    }
                }
                Err(error) => {
                    eprintln!("[gestion-box] error al guardar copia de respaldo: {error}");
                }
            }
        }
        Err(error) => {
            eprintln!("[gestion-box] error al descargar copia de respaldo del servidor: {error}");
        }
    }
}

/// Tarea en segundo plano para la copia periódica del servidor (RF-18, ADR-021).
///
/// Bucle infinito que espera la conexión con el servidor. Al conectar o cada 1 hora,
/// evalúa si corresponde respaldar y ejecuta la copia.
pub async fn tarea_copia_diaria(estado: crate::comandos::EstadoApp) {
    let notificador = estado.notificador_copia();
    loop {
        let cliente = match estado.obtener_cliente() {
            Some(c) => c,
            None => {
                notificador.notified().await;
                continue;
            }
        };

        realizar_intento_copia(&estado, &cliente).await;

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(3600)) => {}
            _ = notificador.notified() => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use chrono::NaiveDateTime;

    use super::super::{copia_mas_reciente, estado_copias, guardar_copia, CABECERA_SQLITE};

    fn fecha(cadena: &str) -> NaiveDateTime {
        cadena.parse().unwrap()
    }

    #[tokio::test]
    #[ignore = "levanta la API real para validar la descarga de respaldo (V2)"]
    async fn descargar_y_guardar_respaldo_desde_api_real() {
        struct ProcesoGuard(std::process::Child);
        impl Drop for ProcesoGuard {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }

        let temp_api = tempfile::tempdir().unwrap();
        let temp_copias = tempfile::tempdir().unwrap();

        // Reservar puertos libres
        let (puerto_http, puerto_udp) = {
            let l1 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let p1 = l1.local_addr().unwrap().port();
            let l2 = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
            let p2 = l2.local_addr().unwrap().port();
            (p1, p2)
        };

        let ruta_binario =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/gestion-box-api");

        let mut comando = if ruta_binario.exists() {
            std::process::Command::new(ruta_binario)
        } else {
            let mut c = std::process::Command::new("cargo");
            c.args(["run", "-p", "gestion-box-api"]);
            c
        };

        comando
            .env("GESTIONBOX_DATA_DIR", temp_api.path())
            .env("GESTIONBOX_HTTP_PORT", puerto_http.to_string())
            .env("GESTIONBOX_UDP_PORT", puerto_udp.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let mut child = ProcesoGuard(
            comando
                .spawn()
                .expect("no se pudo iniciar el proceso de la API"),
        );

        // Esperar a que la API responda en /health
        let direccion = format!("127.0.0.1:{puerto_http}");
        let cliente = crate::cliente_api::ClienteApi::new(&direccion).unwrap();
        let mut lista = false;
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if cliente.salud().await.is_ok() {
                lista = true;
                break;
            }
        }
        assert!(lista, "la API no respondió a tiempo en /health");

        // Descargar el respaldo consistente
        let bytes = cliente
            .descargar_respaldo()
            .await
            .expect("falló la descarga del respaldo");
        assert!(!bytes.is_empty(), "los bytes descargados están vacíos");
        assert!(
            bytes.starts_with(CABECERA_SQLITE),
            "los bytes no tienen cabecera SQLite válida"
        );

        // Guardar la copia en la carpeta temporal
        let ahora = fecha("2026-11-06T15:30:00");
        let ruta_copia = guardar_copia(temp_copias.path(), &bytes, ahora)
            .expect("falló al guardar la copia descargada");
        assert!(ruta_copia.exists(), "el archivo de copia no existe");

        let estado = estado_copias(temp_copias.path());
        assert_eq!(estado.cantidad, 1);
        assert_eq!(estado.ultima.as_deref(), Some("2026-11-06T15:30:00"));

        let mas_reciente = copia_mas_reciente(temp_copias.path());
        assert_eq!(mas_reciente, Some(ruta_copia));

        // Terminar el proceso explícitamente
        let _ = child.0.kill();
        let _ = child.0.wait();
    }
}
