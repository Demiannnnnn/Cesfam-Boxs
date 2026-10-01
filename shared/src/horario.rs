//! Horario de atención y bloques. Fuente única: si el CESFAM confirma otro
//! horario (T-0.1), solo se cambian estas constantes y el `CHECK` del esquema.

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::tipos::Bloque;

/// Hora del primer bloque (08:00).
pub const HORA_INICIO: u32 = 8;
/// Duración de cada bloque en minutos.
pub const MINUTOS_POR_BLOQUE: u32 = 30;
/// Cantidad de bloques por día (08:00–20:00 → 24).
pub const CANTIDAD_BLOQUES: u8 = 24;

/// Hora de inicio de un bloque.
pub fn hora_inicio_bloque(bloque: Bloque) -> NaiveTime {
    let minutos = HORA_INICIO * 60 + u32::from(bloque.valor()) * MINUTOS_POR_BLOQUE;
    NaiveTime::from_hms_opt(minutos / 60, minutos % 60, 0).unwrap_or(NaiveTime::MIN)
}

/// Etiqueta `HH:MM` del inicio de un bloque. Ej.: bloque 0 → `"08:00"`.
pub fn etiqueta_bloque(bloque: Bloque) -> String {
    hora_inicio_bloque(bloque).format("%H:%M").to_string()
}

/// Descripción del horario para enviarla al frontend (evita duplicar constantes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Horario {
    pub hora_inicio: u32,
    pub minutos_por_bloque: u32,
    pub cantidad_bloques: u8,
    /// Etiquetas `HH:MM` de cada bloque, en orden.
    pub etiquetas: Vec<String>,
}

impl Horario {
    /// Horario vigente según las constantes de este módulo.
    pub fn actual() -> Self {
        Self {
            hora_inicio: HORA_INICIO,
            minutos_por_bloque: MINUTOS_POR_BLOQUE,
            cantidad_bloques: CANTIDAD_BLOQUES,
            etiquetas: Bloque::todos().map(etiqueta_bloque).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primer_y_ultimo_bloque_tienen_la_hora_esperada() {
        let primero = Bloque::new(0).unwrap();
        let ultimo = Bloque::new(CANTIDAD_BLOQUES - 1).unwrap();

        assert_eq!(etiqueta_bloque(primero), "08:00");
        assert_eq!(etiqueta_bloque(ultimo), "19:30");
    }

    #[test]
    fn horario_actual_tiene_una_etiqueta_por_bloque() {
        let horario = Horario::actual();

        assert_eq!(horario.etiquetas.len(), usize::from(CANTIDAD_BLOQUES));
        assert_eq!(horario.etiquetas[1], "08:30");
    }
}
