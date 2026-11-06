//! Instalación del servidor local desde la app (RF-16, RNF-12, RNF-14).
//!
//! En Windows ejecuta `instalar.bat` con elevación (UAC) mediante PowerShell,
//! verifica la respuesta del endpoint `/health` y registra el equipo como servidor.

use crate::errores::ErrorUi;

/// Construye la línea de argumentos de `cmd.exe` para invocar `instalar.bat` con o sin respaldo.
/// Cubre RF-16 y RNF-12.
pub fn linea_cmd(bat: &str, respaldo: Option<&str>) -> String {
    match respaldo {
        Some(ruta) => format!(r#"/c ""{bat}" /sinpausa /respaldo "{ruta}"""#),
        None => format!(r#"/c ""{bat}" /sinpausa""#),
    }
}

/// Traduce el código de salida de la instalación a un mensaje de error legible para el usuario.
/// Devuelve `None` si la ejecución terminó con éxito (código 0).
/// Cubre RF-16 y RNF-12.
pub fn mensaje_por_codigo(codigo: i32) -> Option<String> {
    match codigo {
        0 => None,
        1223 => Some(
            "Se canceló el permiso de administrador. Vuelve a intentarlo y acepta el aviso de Windows."
                .to_owned(),
        ),
        otro => Some(format!(
            "La instalación del servidor falló (código {otro}). Revisa C:\\ProgramData\\GestionBox\\api.log y servicio.log."
        )),
    }
}

#[cfg(windows)]
const SCRIPT_POWERSHELL: &str = r#"
try {
    $p = Start-Process -FilePath cmd.exe -ArgumentList $env:GB_ARGS -Verb RunAs -Wait -PassThru
    if ($p -eq $null) { exit 1 }
    exit $p.ExitCode
} catch {
    exit 1223
}
"#;

/// Instala la API como servicio en este equipo con o sin archivo de respaldo. Cubre RF-16, RF-19, RNF-12 y RNF-14.
#[cfg(windows)]
pub async fn instalar_servidor_logica(
    app: &tauri::AppHandle,
    respaldo: Option<&str>,
) -> Result<(), ErrorUi> {
    use std::time::Duration;
    use tauri::Manager;

    let carpeta_servidor = app
        .path()
        .resource_dir()
        .map_err(|e| {
            ErrorUi::new(
                "interno",
                format!("No se pudo obtener la carpeta de recursos: {e}"),
            )
        })?
        .join("servidor");

    let bat = carpeta_servidor.join("instalar.bat");
    let api_exe = carpeta_servidor.join("gestion-box-api.exe");
    let nssm_exe = carpeta_servidor.join("nssm.exe");

    if !bat.exists() || !api_exe.exists() || !nssm_exe.exists() {
        return Err(ErrorUi::new(
            "recursos_faltantes",
            "Faltan archivos del servidor en la instalación. Reinstala la app.",
        ));
    }

    let bat_str = bat.to_string_lossy().to_string();
    let linea_args = linea_cmd(&bat_str, respaldo);
    let codigo = tokio::task::spawn_blocking(move || -> Result<i32, ErrorUi> {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        let mut cmd = Command::new("powershell.exe");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            SCRIPT_POWERSHELL,
        ]);
        cmd.env("GB_ARGS", &linea_args);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let salida = cmd
            .status()
            .map_err(|e| ErrorUi::new("interno", format!("No se pudo ejecutar powershell: {e}")))?;

        Ok(salida.code().unwrap_or(1))
    })
    .await
    .map_err(|e| ErrorUi::new("interno", format!("Fallo al esperar instalación: {e}")))??;

    if let Some(mensaje) = mensaje_por_codigo(codigo) {
        return Err(ErrorUi::new("instalacion_fallida", mensaje));
    }

    let cliente = crate::cliente_api::ClienteApi::new("127.0.0.1:8000")
        .map_err(|e| ErrorUi::new("interno", format!("Error al preparar cliente: {e}")))?;

    let mut equipo = None;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if let Ok(salud) = cliente.salud().await {
            equipo = Some(salud.equipo);
            break;
        }
    }

    let equipo = equipo.ok_or_else(|| {
        ErrorUi::new(
            "servidor_no_responde",
            "El servidor se instaló pero no responde. Revisa services.msc y el puerto 8000.",
        )
    })?;

    let estado = app.state::<crate::comandos::EstadoApp>();
    estado.registrar_servidor_local(equipo)?;

    Ok(())
}

/// En sistemas distintos de Windows la instalación del servicio no está soportada (RNF-12).
#[cfg(not(windows))]
pub async fn instalar_servidor_logica(
    _app: &tauri::AppHandle,
    _respaldo: Option<&str>,
) -> Result<(), ErrorUi> {
    Err(ErrorUi::new("no_soportado", "Solo disponible en Windows."))
}

/// Instala la API como servicio de Windows en este equipo. Cubre RF-16, RNF-12 y RNF-14.
#[tauri::command]
pub async fn instalar_servidor(
    app: tauri::AppHandle,
    respaldo: Option<String>,
) -> Result<(), ErrorUi> {
    instalar_servidor_logica(&app, respaldo.as_deref()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linea_cmd_sin_respaldo() {
        let bat = r#"C:\Program Files\GestionBox\servidor\instalar.bat"#;
        assert_eq!(
            linea_cmd(bat, None),
            r#"/c ""C:\Program Files\GestionBox\servidor\instalar.bat" /sinpausa""#
        );
    }

    #[test]
    fn linea_cmd_con_respaldo_y_espacios() {
        let bat = r#"C:\Program Files\GestionBox\servidor\instalar.bat"#;
        let respaldo = r#"C:\Copias de seguridad\gestion-box 2026.db"#;
        assert_eq!(
            linea_cmd(bat, Some(respaldo)),
            r#"/c ""C:\Program Files\GestionBox\servidor\instalar.bat" /sinpausa /respaldo "C:\Copias de seguridad\gestion-box 2026.db"""#
        );
    }

    #[test]
    fn mensaje_por_codigo_exito() {
        assert_eq!(mensaje_por_codigo(0), None);
    }

    #[test]
    fn mensaje_por_codigo_uac_cancelado() {
        assert_eq!(
            mensaje_por_codigo(1223).as_deref(),
            Some("Se canceló el permiso de administrador. Vuelve a intentarlo y acepta el aviso de Windows.")
        );
    }

    #[test]
    fn mensaje_por_codigo_fallo_generico() {
        assert_eq!(
            mensaje_por_codigo(1).as_deref(),
            Some("La instalación del servidor falló (código 1). Revisa C:\\ProgramData\\GestionBox\\api.log y servicio.log.")
        );
        assert_eq!(
            mensaje_por_codigo(5).as_deref(),
            Some("La instalación del servidor falló (código 5). Revisa C:\\ProgramData\\GestionBox\\api.log y servicio.log.")
        );
    }
}
