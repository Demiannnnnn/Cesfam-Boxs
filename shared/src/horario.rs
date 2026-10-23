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
/// Hora en que empieza la tarde (PM). La planilla del CESFAM divide cada día en
/// "BOX AM" y "BOX PM"; la grilla y los atajos Mañana/Tarde usan este corte.
/// Pendiente de confirmar en terreno (T-0.1).
pub const HORA_INICIO_TARDE: u32 = 14;

/// Primer bloque de la tarde (14:00 → 12).
pub const fn bloque_inicio_tarde() -> u8 {
    ((HORA_INICIO_TARDE - HORA_INICIO) * 60 / MINUTOS_POR_BLOQUE) as u8
}

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
    /// Primer bloque de la tarde; los anteriores son la mañana.
    pub bloque_inicio_tarde: u8,
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
            bloque_inicio_tarde: bloque_inicio_tarde(),
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

    #[test]
    fn la_tarde_empieza_a_las_14_y_cae_dentro_del_dia() {
        let tarde = Bloque::new(bloque_inicio_tarde()).unwrap();

        assert_eq!(etiqueta_bloque(tarde), "14:00");
    }
}
