//! Respaldo de la base al arrancar y rotación (RF-11, ADR-011).

use std::io;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use rusqlite::{params, Connection};

/// Cantidad de respaldos que se conservan.
pub const RESPALDOS_A_CONSERVAR: usize = 30;

const PREFIJO: &str = "gestion-box-";
const EXTENSION: &str = ".db";

/// Error al respaldar.
#[derive(Debug, thiserror::Error)]
pub enum ErrorRespaldo {
    #[error("no se pudo escribir el respaldo: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("error de archivos al respaldar: {0}")]
    Io(#[from] io::Error),
}

/// Crea `gestion-box-AAAAMMDD-HHMMSS.db` en `carpeta` con `VACUUM INTO`, que produce
/// una copia consistente aun con la base en modo WAL. Si el nombre existe (dos
/// arranques en el mismo segundo) agrega `-2`, `-3`, …
pub fn respaldar(
    conn: &Connection,
    carpeta: &Path,
    ahora: NaiveDateTime,
) -> Result<PathBuf, ErrorRespaldo> {
    std::fs::create_dir_all(carpeta)?;
    let base = format!("{PREFIJO}{}", ahora.format("%Y%m%d-%H%M%S"));
    let mut ruta = carpeta.join(format!("{base}{EXTENSION}"));
    let mut sufijo = 2;
    while ruta.exists() {
        ruta = carpeta.join(format!("{base}-{sufijo}{EXTENSION}"));
        sufijo += 1;
    }
    // El destino va como parámetro, nunca concatenado (RNF-06).
    conn.execute("VACUUM INTO ?1", params![ruta.to_string_lossy()])?;
    Ok(ruta)
}

/// Borra los respaldos más antiguos dejando `conservar`. Solo toca archivos con el
/// patrón `gestion-box-*.db`; cualquier otro archivo de la carpeta se respeta.
/// Devuelve cuántos borró.
pub fn rotar(carpeta: &Path, conservar: usize) -> io::Result<usize> {
    let mut respaldos: Vec<(String, PathBuf)> = std::fs::read_dir(carpeta)?
        .filter_map(Result::ok)
        .filter(|entrada| entrada.file_type().is_ok_and(|tipo| tipo.is_file()))
        .filter_map(|entrada| {
            let nombre = entrada.file_name().into_string().ok()?;
            let sin_extension = nombre.strip_prefix(PREFIJO)?.strip_suffix(EXTENSION)?;
            // La clave sin extensión ordena cronológicamente, incluidos los sufijos -2, -3…
            Some((sin_extension.to_owned(), entrada.path()))
        })
        .collect();

    if respaldos.len() <= conservar {
        return Ok(0);
    }
    respaldos.sort();
    let sobrantes = respaldos.len() - conservar;
    for (_, ruta) in respaldos.iter().take(sobrantes) {
        std::fs::remove_file(ruta)?;
    }
    Ok(sobrantes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use chrono::Duration;

    fn inicio() -> NaiveDateTime {
        "2026-10-01T08:00:00".parse().unwrap()
    }

    fn nombres(carpeta: &Path) -> Vec<String> {
        let mut lista: Vec<String> = std::fs::read_dir(carpeta)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        lista.sort();
        lista
    }

    #[test]
    fn treinta_y_cinco_respaldos_quedan_en_treinta_los_mas_recientes() {
        let carpeta = tempfile::tempdir().unwrap();
        let conn = db::abrir_en_memoria().unwrap();

        for minuto in 0..35 {
            respaldar(&conn, carpeta.path(), inicio() + Duration::minutes(minuto)).unwrap();
            rotar(carpeta.path(), RESPALDOS_A_CONSERVAR).unwrap();
        }

        let lista = nombres(carpeta.path());
        assert_eq!(lista.len(), 30);
        assert_eq!(lista[0], "gestion-box-20261001-080500.db");
        assert_eq!(lista[29], "gestion-box-20261001-083400.db");
    }

    #[test]
    fn rotar_no_borra_archivos_ajenos() {
        let carpeta = tempfile::tempdir().unwrap();
        let conn = db::abrir_en_memoria().unwrap();
        std::fs::write(carpeta.path().join("otro.db"), b"x").unwrap();
        for minuto in 0..3 {
            respaldar(&conn, carpeta.path(), inicio() + Duration::minutes(minuto)).unwrap();
        }

        let borrados = rotar(carpeta.path(), 1).unwrap();

        assert_eq!(borrados, 2);
        assert!(carpeta.path().join("otro.db").exists());
    }

    #[test]
    fn mismo_segundo_agrega_sufijo_y_respeta_el_orden() {
        let carpeta = tempfile::tempdir().unwrap();
        let conn = db::abrir_en_memoria().unwrap();

        let a = respaldar(&conn, carpeta.path(), inicio()).unwrap();
        let b = respaldar(&conn, carpeta.path(), inicio()).unwrap();
        rotar(carpeta.path(), 1).unwrap();

        assert!(!a.exists());
        assert!(b.ends_with("gestion-box-20261001-080000-2.db"));
        assert!(b.exists());
    }

    #[test]
    fn respaldo_contiene_los_datos() {
        let carpeta = tempfile::tempdir().unwrap();
        let conn = db::abrir_en_memoria().unwrap();
        conn.execute("INSERT INTO box (nombre) VALUES ('Box 1')", [])
            .unwrap();

        let ruta = respaldar(&conn, carpeta.path(), inicio()).unwrap();

        let copia = Connection::open(ruta).unwrap();
        let cantidad: i64 = copia
            .query_row("SELECT COUNT(*) FROM box", [], |f| f.get(0))
            .unwrap();
        assert_eq!(cantidad, 1);
    }
}
