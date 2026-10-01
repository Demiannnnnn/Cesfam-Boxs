//! Apertura de SQLite, PRAGMAs y migraciones versionadas (RF-05, RNF-06).
//!
//! La versión del esquema se guarda en `PRAGMA user_version`. Nunca edites una
//! migración ya publicada: agrega una nueva al final de [`MIGRACIONES`].

use std::path::Path;

use rusqlite::Connection;

/// Migraciones en orden. El índice + 1 es la versión que dejan aplicada.
///
/// El `CHECK` de `bloque` debe coincidir con `shared::horario::CANTIDAD_BLOQUES`
/// (lo verifica un test).
const MIGRACIONES: &[&str] = &[
    // v1 — esquema inicial.
    "CREATE TABLE IF NOT EXISTS box (
        id      INTEGER PRIMARY KEY AUTOINCREMENT,
        nombre  TEXT    NOT NULL UNIQUE COLLATE NOCASE,
        activo  INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0, 1)),
        orden   INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS reserva (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        box_id      INTEGER NOT NULL REFERENCES box(id),
        fecha       TEXT    NOT NULL,
        bloque      INTEGER NOT NULL CHECK (bloque BETWEEN 0 AND 23),
        funcionario TEXT    NOT NULL,
        motivo      TEXT,
        creada_en   TEXT    NOT NULL,
        UNIQUE (box_id, fecha, bloque)
    );
    CREATE INDEX IF NOT EXISTS idx_reserva_fecha ON reserva (fecha);",
];

/// Abre (o crea) la base en `ruta`, aplica los PRAGMAs y las migraciones pendientes.
pub fn abrir(ruta: &Path) -> rusqlite::Result<Connection> {
    let mut conn = Connection::open(ruta)?;
    configurar(&conn)?;
    migrar(&mut conn)?;
    Ok(conn)
}

/// Base en memoria lista para usar. Pensada para tests.
pub fn abrir_en_memoria() -> rusqlite::Result<Connection> {
    let mut conn = Connection::open_in_memory()?;
    configurar(&conn)?;
    migrar(&mut conn)?;
    Ok(conn)
}

fn configurar(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    // `journal_mode` devuelve una fila, por eso se usa `query_row`.
    conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
    Ok(())
}

/// Aplica, dentro de una transacción, las migraciones con versión mayor a la actual.
/// Es idempotente.
pub fn migrar(conn: &mut Connection) -> rusqlite::Result<()> {
    let actual: i64 = conn.query_row("PRAGMA user_version", [], |fila| fila.get(0))?;
    let actual = usize::try_from(actual).unwrap_or(0);
    if actual >= MIGRACIONES.len() {
        return Ok(());
    }

    let tx = conn.transaction()?;
    for (indice, sql) in MIGRACIONES.iter().enumerate().skip(actual) {
        tx.execute_batch(sql)?;
        let version = i64::try_from(indice + 1).unwrap_or(i64::MAX);
        tx.pragma_update(None, "user_version", version)?;
        tracing::info!(version = indice + 1, "migración de esquema aplicada");
    }
    tx.commit()
}

/// Versión de esquema que dejan las migraciones conocidas.
pub fn version_esquema() -> usize {
    MIGRACIONES.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gestion_box_shared::horario::CANTIDAD_BLOQUES;
    use rusqlite::{ffi, params};

    fn con_box() -> Connection {
        let conn = abrir_en_memoria().unwrap();
        conn.execute("INSERT INTO box (nombre) VALUES ('Box 1')", [])
            .unwrap();
        conn
    }

    fn insertar_reserva(conn: &Connection, box_id: i64, bloque: i64) -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO reserva (box_id, fecha, bloque, funcionario, creada_en)
             VALUES (?1, '2026-10-01', ?2, 'Ana', '2026-10-01T08:00:00')",
            params![box_id, bloque],
        )
    }

    fn codigo_extendido(error: rusqlite::Error) -> i32 {
        match error {
            rusqlite::Error::SqliteFailure(e, _) => e.extended_code,
            otro => panic!("error inesperado: {otro:?}"),
        }
    }

    #[test]
    fn migrar_dos_veces_es_idempotente() {
        let mut conn = abrir_en_memoria().unwrap();

        migrar(&mut conn).unwrap();

        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |f| f.get(0))
            .unwrap();
        assert_eq!(usize::try_from(version).unwrap(), version_esquema());
    }

    #[test]
    fn reserva_duplicada_viola_unique() {
        let conn = con_box();
        insertar_reserva(&conn, 1, 4).unwrap();

        let error = insertar_reserva(&conn, 1, 4).unwrap_err();

        assert_eq!(codigo_extendido(error), ffi::SQLITE_CONSTRAINT_UNIQUE);
    }

    #[test]
    fn check_de_bloque_coincide_con_el_horario() {
        let conn = con_box();

        assert!(insertar_reserva(&conn, 1, i64::from(CANTIDAD_BLOQUES) - 1).is_ok());
        let error = insertar_reserva(&conn, 1, i64::from(CANTIDAD_BLOQUES)).unwrap_err();

        assert_eq!(codigo_extendido(error), ffi::SQLITE_CONSTRAINT_CHECK);
    }

    #[test]
    fn reserva_de_box_inexistente_viola_foreign_key() {
        let conn = con_box();

        let error = insertar_reserva(&conn, 999, 0).unwrap_err();

        assert_eq!(codigo_extendido(error), ffi::SQLITE_CONSTRAINT_FOREIGNKEY);
    }
}
