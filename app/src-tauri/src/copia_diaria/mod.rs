//! Copia diaria automática de la base de datos en los equipos de escritura (RF-18, ADR-021).
//!
//! Cada equipo en modo `escritura` obtiene periódicamente una copia consistente
//! de la base del servidor a través de `GET /respaldo` y conserva las últimas
//! [`COPIAS_A_CONSERVAR`] copias en disco local.

pub mod tarea;

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub use tarea::{realizar_intento_copia, tarea_copia_diaria};

use crate::config_local::ModoEquipo;

/// Cantidad predeterminada de copias locales a conservar (RF-18, ADR-021).
pub const COPIAS_A_CONSERVAR: usize = 7;

/// Encabezado característico de un archivo SQLite 3 (RF-18).
pub const CABECERA_SQLITE: &[u8] = b"SQLite format 3\0";

const PREFIJO: &str = "gestion-box-";
const EXTENSION: &str = ".db";

/// Error al guardar una copia de respaldo descargada (RF-18).
#[derive(Debug, thiserror::Error)]
pub enum ErrorGuardarCopia {
    /// Los datos descargados están vacíos.
    #[error("el archivo recibido está vacío")]
    Vacio,
    /// El archivo no empieza con la firma de una base SQLite válida.
    #[error("el archivo recibido no corresponde a una base de datos SQLite válida")]
    EncabezadoInvalido,
    /// Error de entrada/salida en el sistema de archivos.
    #[error("error de entrada/salida al guardar la copia: {0}")]
    Io(#[from] std::io::Error),
}

/// Estado de las copias locales de respaldo disponibles en el equipo (RF-18, ADR-021).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstadoCopias {
    /// Fecha y hora de la última copia de respaldo (`%Y-%m-%dT%H:%M:%S`), o `None` si no hay copias.
    pub ultima: Option<String>,
    /// Cantidad total de archivos de respaldo disponibles en la carpeta.
    pub cantidad: usize,
}

/// Resuelve la carpeta donde se almacenan las copias del servidor (RF-18, ADR-021).
///
/// En Windows retorna `%ProgramData%\GestionBox\copias-del-servidor`. En otros
/// sistemas utiliza el subdirectorio `copias-del-servidor` junto a la configuración.
pub fn carpeta_copias(dir_config: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let _ = dir_config;
        resolver_carpeta_copias_windows(|var| std::env::var(var).ok())
    }
    #[cfg(not(windows))]
    {
        dir_config.join("copias-del-servidor")
    }
}

/// Resuelve la ruta de copias en Windows consultando el entorno con valor de respaldo (RF-18).
pub fn resolver_carpeta_copias_windows(entorno: impl Fn(&str) -> Option<String>) -> PathBuf {
    let base = entorno("ProgramData")
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| r"C:\ProgramData".to_owned());
    PathBuf::from(base)
        .join("GestionBox")
        .join("copias-del-servidor")
}

/// Determina si corresponde realizar una copia de respaldo según la fecha previa (RF-18, ADR-021).
///
/// Devuelve `true` si no existe copia previa, si transcurrieron 24 horas o más desde la
/// última, o si la fecha registrada es posterior a la hora actual (reloj del sistema alterado).
pub fn toca_copiar(ultima: Option<NaiveDateTime>, ahora: NaiveDateTime) -> bool {
    match ultima {
        None => true,
        Some(ult) => {
            if ult > ahora {
                true
            } else {
                ahora - ult >= chrono::Duration::hours(24)
            }
        }
    }
}

/// Evalúa si este equipo debe realizar una copia en este instante (RF-18, ADR-020, ADR-021).
///
/// Solo los equipos configurados en [`ModoEquipo::Escritura`] ejecutan la copia diaria.
/// Equipos de lectura, servidor o sin modo configurado devuelven siempre `false`.
pub fn debe_copiar(
    modo: Option<ModoEquipo>,
    ultima: Option<NaiveDateTime>,
    ahora: NaiveDateTime,
) -> bool {
    match modo {
        Some(ModoEquipo::Escritura) => toca_copiar(ultima, ahora),
        _ => false,
    }
}

/// Guarda los bytes descargados de forma atómica en `carpeta` (RF-18, ADR-021).
///
/// Valida que el archivo no esté vacío y comience con la cabecera `SQLite format 3\0`.
/// Escribe a un archivo temporal `.tmp` y luego lo renombra al nombre definitivo
/// `gestion-box-AAAAMMDD-HHMMSS.db`. Si el nombre ya existe, añade sufijos `-2`, `-3`, etc.
/// Si ocurre algún error en la escritura o renombrado, elimina el temporal.
pub fn guardar_copia(
    carpeta: &Path,
    bytes: &[u8],
    ahora: NaiveDateTime,
) -> Result<PathBuf, ErrorGuardarCopia> {
    if bytes.is_empty() {
        return Err(ErrorGuardarCopia::Vacio);
    }
    if !bytes.starts_with(CABECERA_SQLITE) {
        return Err(ErrorGuardarCopia::EncabezadoInvalido);
    }

    std::fs::create_dir_all(carpeta)?;

    let base = format!("{PREFIJO}{}", ahora.format("%Y%m%d-%H%M%S"));
    let mut ruta_final = carpeta.join(format!("{base}{EXTENSION}"));
    let mut sufijo = 2;
    while ruta_final.exists() {
        ruta_final = carpeta.join(format!("{base}-{sufijo}{EXTENSION}"));
        sufijo += 1;
    }

    let nombre_final = ruta_final
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archivo.db");
    let ruta_tmp = carpeta.join(format!("{nombre_final}.tmp"));

    let resultado = (|| -> std::io::Result<()> {
        std::fs::write(&ruta_tmp, bytes)?;
        std::fs::rename(&ruta_tmp, &ruta_final)?;
        Ok(())
    })();

    if let Err(error) = resultado {
        if ruta_tmp.exists() {
            let _ = std::fs::remove_file(&ruta_tmp);
        }
        return Err(ErrorGuardarCopia::Io(error));
    }

    Ok(ruta_final)
}

/// Lista los archivos de respaldo válidos en `carpeta`.
///
/// Retorna tuplas `(clave, ruta)` donde `clave` es el nombre sin el prefijo `gestion-box-`
/// ni la extensión `.db`. Al ordenar por esta clave se asegura el orden cronológico
/// estricto (por ejemplo, `...-120000-2` queda después de `...-120000`, a diferencia de
/// ordenar con la extensión `.db` donde `'-' < '.'`).
fn listar_archivos_copia(carpeta: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    let mut copias = Vec::new();
    if !carpeta.exists() {
        return Ok(copias);
    }
    for entrada in std::fs::read_dir(carpeta)? {
        let entrada = entrada?;
        if !entrada.file_type()?.is_file() {
            continue;
        }
        let Some(nombre) = entrada.file_name().to_str().map(|s| s.to_owned()) else {
            continue;
        };
        let Some(sin_prefijo) = nombre.strip_prefix(PREFIJO) else {
            continue;
        };
        let Some(clave) = sin_prefijo.strip_suffix(EXTENSION) else {
            continue;
        };
        copias.push((clave.to_owned(), entrada.path()));
    }
    Ok(copias)
}

/// Extrae la fecha legible `%Y-%m-%dT%H:%M:%S` del nombre de archivo de respaldo (RF-18).
fn extraer_fecha_de_nombre(ruta: &Path) -> Option<String> {
    let nombre = ruta.file_name()?.to_str()?;
    let sin_prefijo = nombre.strip_prefix(PREFIJO)?;
    let sin_ext = sin_prefijo.strip_suffix(EXTENSION)?;
    if sin_ext.len() >= 15 {
        let segmento_fecha = &sin_ext[..15];
        if let Ok(dt) = NaiveDateTime::parse_from_str(segmento_fecha, "%Y%m%d-%H%M%S") {
            return Some(dt.format("%Y-%m-%dT%H:%M:%S").to_string());
        }
    }
    Some(sin_ext.to_string())
}

/// Elimina las copias más antiguas dejando a lo sumo `conservar` archivos (RF-18, ADR-021).
///
/// Solo afecta archivos que coincidan con el patrón `gestion-box-*.db`. Cualquier
/// otro archivo o extensión (como `.tmp`) se preserva sin cambios.
pub fn rotar(carpeta: &Path, conservar: usize) -> std::io::Result<usize> {
    if !carpeta.exists() {
        return Ok(0);
    }
    let mut copias = listar_archivos_copia(carpeta)?;
    if copias.len() <= conservar {
        return Ok(0);
    }
    copias.sort();
    let sobrantes = copias.len() - conservar;
    for (_, ruta) in copias.iter().take(sobrantes) {
        std::fs::remove_file(ruta)?;
    }
    Ok(sobrantes)
}

/// Devuelve la ruta de la copia más reciente disponible en la carpeta (RF-18, ADR-022).
pub fn copia_mas_reciente(carpeta: &Path) -> Option<PathBuf> {
    let mut copias = listar_archivos_copia(carpeta).ok()?;
    if copias.is_empty() {
        return None;
    }
    copias.sort();
    copias.pop().map(|(_, ruta)| ruta)
}

/// Devuelve el estado actual de las copias en `carpeta` (RF-18, ADR-021).
pub fn estado_copias(carpeta: &Path) -> EstadoCopias {
    let mut copias = match listar_archivos_copia(carpeta) {
        Ok(c) => c,
        Err(_) => {
            return EstadoCopias {
                ultima: None,
                cantidad: 0,
            };
        }
    };
    if copias.is_empty() {
        return EstadoCopias {
            ultima: None,
            cantidad: 0,
        };
    }
    copias.sort();
    let cantidad = copias.len();
    let ultima = match copias.last() {
        Some((_, ruta)) => extraer_fecha_de_nombre(ruta),
        None => None,
    };

    EstadoCopias { ultima, cantidad }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    fn fecha(cadena: &str) -> NaiveDateTime {
        cadena.parse().unwrap()
    }

    #[test]
    fn rotacion_conserva_siete_mas_recientes_y_respeta_otros_archivos() {
        let temp = tempfile::tempdir().unwrap();
        let carpeta = temp.path();

        // Crear 9 archivos de copia
        for dia in 1..=9 {
            let nombre = format!("gestion-box-2026110{dia}-100000.db");
            std::fs::write(carpeta.join(nombre), CABECERA_SQLITE).unwrap();
        }

        // Crear archivo ajeno y archivo temporal
        std::fs::write(carpeta.join("otro-archivo.txt"), b"datos").unwrap();
        std::fs::write(
            carpeta.join("gestion-box-20261110-100000.db.tmp"),
            b"temporal",
        )
        .unwrap();

        let borrados = rotar(carpeta, 7).unwrap();
        assert_eq!(borrados, 2);

        // Los dos más antiguos (días 1 y 2) deben haber sido eliminados
        assert!(!carpeta.join("gestion-box-20261101-100000.db").exists());
        assert!(!carpeta.join("gestion-box-20261102-100000.db").exists());

        // Los 7 más recientes (días 3 a 9) deben conservarse
        for dia in 3..=9 {
            let nombre = format!("gestion-box-2026110{dia}-100000.db");
            assert!(carpeta.join(nombre).exists());
        }

        // El archivo ajeno y el .tmp deben respetarse
        assert!(carpeta.join("otro-archivo.txt").exists());
        assert!(carpeta.join("gestion-box-20261110-100000.db.tmp").exists());
    }

    #[test]
    fn orden_cronologico_con_sufijos_prefiere_archivo_con_guion() {
        let temp = tempfile::tempdir().unwrap();
        let carpeta = temp.path();

        let nombre_base = "gestion-box-20261106-120000.db";
        let nombre_sufijo = "gestion-box-20261106-120000-2.db";

        let ruta_base = carpeta.join(nombre_base);
        let ruta_sufijo = carpeta.join(nombre_sufijo);

        std::fs::write(&ruta_base, CABECERA_SQLITE).unwrap();
        std::fs::write(&ruta_sufijo, CABECERA_SQLITE).unwrap();

        // La copia más reciente debe ser la que tiene el sufijo -2 (creada después)
        assert_eq!(copia_mas_reciente(carpeta), Some(ruta_sufijo.clone()));

        // Rotar conservando 1 debe borrar la base antigua y dejar la -2
        let borrados = rotar(carpeta, 1).unwrap();
        assert_eq!(borrados, 1);
        assert!(
            !ruta_base.exists(),
            "la copia base antigua debió ser eliminada"
        );
        assert!(
            ruta_sufijo.exists(),
            "la copia con sufijo -2 debe conservarse"
        );

        // El estado de copias debe reflejar la única copia restante (-2)
        let estado = estado_copias(carpeta);
        assert_eq!(estado.cantidad, 1);
        assert_eq!(estado.ultima.as_deref(), Some("2026-11-06T12:00:00"));
    }

    #[test]
    fn toca_copiar_evalua_intervalo_de_veinticuatro_horas_y_reloj_futuro() {
        let ahora = fecha("2026-11-06T12:00:00");

        // Sin copia previa -> toca copiar
        assert!(toca_copiar(None, ahora));

        // Hace 23 horas -> no toca copiar
        let hace_23h = ahora - ChronoDuration::hours(23);
        assert!(!toca_copiar(Some(hace_23h), ahora));

        // Hace 24 horas exactas -> toca copiar
        let hace_24h = ahora - ChronoDuration::hours(24);
        assert!(toca_copiar(Some(hace_24h), ahora));

        // Hace 25 horas -> toca copiar
        let hace_25h = ahora - ChronoDuration::hours(25);
        assert!(toca_copiar(Some(hace_25h), ahora));

        // Fecha futura (reloj modificado o adelantado) -> toca copiar
        let futura = ahora + ChronoDuration::hours(2);
        assert!(toca_copiar(Some(futura), ahora));
    }

    #[test]
    fn debe_copiar_solo_permite_modo_escritura() {
        let ahora = fecha("2026-11-06T12:00:00");
        let hace_25h = ahora - ChronoDuration::hours(25);
        let hace_1h = ahora - ChronoDuration::hours(1);

        // Solo Escritura puede copiar cuando toca
        assert!(debe_copiar(Some(ModoEquipo::Escritura), None, ahora));
        assert!(debe_copiar(
            Some(ModoEquipo::Escritura),
            Some(hace_25h),
            ahora
        ));
        assert!(!debe_copiar(
            Some(ModoEquipo::Escritura),
            Some(hace_1h),
            ahora
        ));

        // Lectura nunca copia
        assert!(!debe_copiar(Some(ModoEquipo::Lectura), None, ahora));
        assert!(!debe_copiar(
            Some(ModoEquipo::Lectura),
            Some(hace_25h),
            ahora
        ));

        // Servidor nunca descarga (la API ya corre localmente)
        assert!(!debe_copiar(Some(ModoEquipo::Servidor), None, ahora));
        assert!(!debe_copiar(
            Some(ModoEquipo::Servidor),
            Some(hace_25h),
            ahora
        ));

        // Sin modo asignado (asistente inicial no completado) nunca descarga
        assert!(!debe_copiar(None, None, ahora));
        assert!(!debe_copiar(None, Some(hace_25h), ahora));
    }

    #[test]
    fn bytes_invalidos_y_vacios_no_escriben_ni_borran_existentes() {
        let temp = tempfile::tempdir().unwrap();
        let carpeta = temp.path();
        let ahora = fecha("2026-11-06T12:00:00");

        // Guardar previamente una copia válida
        let previa = guardar_copia(carpeta, CABECERA_SQLITE, ahora).unwrap();
        assert!(previa.exists());

        // Intento con bytes vacíos -> falla
        let ahora_siguiente = ahora + ChronoDuration::seconds(1);
        let res_vacio = guardar_copia(carpeta, &[], ahora_siguiente);
        assert!(matches!(res_vacio, Err(ErrorGuardarCopia::Vacio)));

        // Intento con bytes sin encabezado SQLite -> falla
        let res_invalido =
            guardar_copia(carpeta, b"esto no es sqlite ni parecido", ahora_siguiente);
        assert!(matches!(
            res_invalido,
            Err(ErrorGuardarCopia::EncabezadoInvalido)
        ));

        // La copia previa sigue intacta y no hay archivos temporales residuales
        assert!(previa.exists());
        let entradas: Vec<_> = std::fs::read_dir(carpeta)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entradas.len(), 1);
        assert_eq!(entradas[0], previa.file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn nombre_duplicado_recibe_sufijo() {
        let temp = tempfile::tempdir().unwrap();
        let carpeta = temp.path();
        let ahora = fecha("2026-11-06T12:00:00");

        let ruta1 = guardar_copia(carpeta, CABECERA_SQLITE, ahora).unwrap();
        assert_eq!(
            ruta1.file_name().unwrap().to_str().unwrap(),
            "gestion-box-20261106-120000.db"
        );

        let ruta2 = guardar_copia(carpeta, CABECERA_SQLITE, ahora).unwrap();
        assert_eq!(
            ruta2.file_name().unwrap().to_str().unwrap(),
            "gestion-box-20261106-120000-2.db"
        );

        let ruta3 = guardar_copia(carpeta, CABECERA_SQLITE, ahora).unwrap();
        assert_eq!(
            ruta3.file_name().unwrap().to_str().unwrap(),
            "gestion-box-20261106-120000-3.db"
        );

        assert!(ruta1.exists());
        assert!(ruta2.exists());
        assert!(ruta3.exists());
    }

    #[test]
    fn estado_copias_y_mas_reciente_con_carpeta_vacia_y_con_archivos() {
        let temp = tempfile::tempdir().unwrap();
        let carpeta = temp.path();

        // Carpeta vacía
        let estado_vacio = estado_copias(carpeta);
        assert_eq!(
            estado_vacio,
            EstadoCopias {
                ultima: None,
                cantidad: 0,
            }
        );
        assert_eq!(copia_mas_reciente(carpeta), None);

        // Con 2 archivos
        let f1 = fecha("2026-11-05T09:30:00");
        let f2 = fecha("2026-11-06T15:45:00");
        guardar_copia(carpeta, CABECERA_SQLITE, f1).unwrap();
        let ruta2 = guardar_copia(carpeta, CABECERA_SQLITE, f2).unwrap();

        let estado = estado_copias(carpeta);
        assert_eq!(
            estado,
            EstadoCopias {
                ultima: Some("2026-11-06T15:45:00".to_owned()),
                cantidad: 2,
            }
        );
        assert_eq!(copia_mas_reciente(carpeta), Some(ruta2));
    }

    #[test]
    fn carpeta_copias_windows_usa_program_data_o_respaldo() {
        let con_var = resolver_carpeta_copias_windows(|var| {
            (var == "ProgramData").then(|| r"D:\Datos".into())
        });
        assert_eq!(
            con_var,
            PathBuf::from(r"D:\Datos")
                .join("GestionBox")
                .join("copias-del-servidor")
        );

        let sin_var = resolver_carpeta_copias_windows(|_| None);
        assert_eq!(
            sin_var,
            PathBuf::from(r"C:\ProgramData")
                .join("GestionBox")
                .join("copias-del-servidor")
        );
    }
}
