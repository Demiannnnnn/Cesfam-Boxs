//! Tipos validados ("parsear, no validar"): si existe un valor, es válido.

use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::horario::CANTIDAD_BLOQUES;

/// Error al construir un tipo validado.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorTipo {
    #[error("el bloque {0} está fuera del horario (debe ser entre 0 y {max})", max = CANTIDAD_BLOQUES - 1)]
    BloqueFueraDeRango(u8),
    #[error("la fecha \"{0}\" no es válida (formato esperado AAAA-MM-DD)")]
    FechaInvalida(String),
}

/// Índice de un bloque horario dentro del día (`0..CANTIDAD_BLOQUES`).
///
/// Se serializa como número. Deserializar un valor fuera de rango falla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Bloque(u8);

impl Bloque {
    /// Crea un bloque si está dentro del horario.
    pub fn new(valor: u8) -> Result<Self, ErrorTipo> {
        if valor < CANTIDAD_BLOQUES {
            Ok(Self(valor))
        } else {
            Err(ErrorTipo::BloqueFueraDeRango(valor))
        }
    }

    /// Valor numérico del bloque.
    pub fn valor(self) -> u8 {
        self.0
    }

    /// Todos los bloques del día, en orden.
    pub fn todos() -> impl Iterator<Item = Bloque> {
        (0..CANTIDAD_BLOQUES).map(Bloque)
    }
}

impl TryFrom<u8> for Bloque {
    type Error = ErrorTipo;

    fn try_from(valor: u8) -> Result<Self, Self::Error> {
        Bloque::new(valor)
    }
}

impl From<Bloque> for u8 {
    fn from(bloque: Bloque) -> u8 {
        bloque.0
    }
}

impl fmt::Display for Bloque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Fecha de calendario en formato `AAAA-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Fecha(NaiveDate);

impl Fecha {
    const FORMATO: &'static str = "%Y-%m-%d";

    /// Envuelve una fecha de `chrono`.
    pub fn new(fecha: NaiveDate) -> Self {
        Self(fecha)
    }

    /// Fecha de `chrono` subyacente.
    pub fn como_naive(self) -> NaiveDate {
        self.0
    }

    /// Fecha local de hoy.
    pub fn hoy() -> Self {
        Self(chrono::Local::now().date_naive())
    }

    /// Días desde `self` hasta `otra` (negativo si `otra` es anterior).
    pub fn dias_hasta(self, otra: Fecha) -> i64 {
        (otra.0 - self.0).num_days()
    }
}

impl FromStr for Fecha {
    type Err = ErrorTipo;

    fn from_str(texto: &str) -> Result<Self, Self::Err> {
        // `parse_from_str` acepta años de más de 4 dígitos y ceros omitidos; exigimos el largo exacto.
        if texto.len() != 10 {
            return Err(ErrorTipo::FechaInvalida(texto.to_owned()));
        }
        NaiveDate::parse_from_str(texto, Self::FORMATO)
            .map(Self)
            .map_err(|_| ErrorTipo::FechaInvalida(texto.to_owned()))
    }
}

impl TryFrom<String> for Fecha {
    type Error = ErrorTipo;

    fn try_from(texto: String) -> Result<Self, Self::Error> {
        texto.parse()
    }
}

impl From<Fecha> for String {
    fn from(fecha: Fecha) -> String {
        fecha.to_string()
    }
}

impl fmt::Display for Fecha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format(Self::FORMATO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloque_fuera_de_rango_se_rechaza() {
        assert!(Bloque::new(23).is_ok());
        assert_eq!(Bloque::new(24), Err(ErrorTipo::BloqueFueraDeRango(24)));
    }

    #[test]
    fn bloque_fuera_de_rango_no_se_deserializa() {
        assert!(serde_json::from_str::<Bloque>("5").is_ok());
        assert!(serde_json::from_str::<Bloque>("24").is_err());
        assert!(serde_json::from_str::<Bloque>("-1").is_err());
    }

    #[test]
    fn fecha_valida_ida_y_vuelta() {
        let fecha: Fecha = "2026-10-01".parse().unwrap();

        assert_eq!(fecha.to_string(), "2026-10-01");
        assert_eq!(serde_json::to_string(&fecha).unwrap(), "\"2026-10-01\"");
    }

    #[test]
    fn fecha_imposible_o_mal_formada_se_rechaza() {
        for texto in ["2026-02-30", "01-10-2026", "2026-1-1", "", "hoy"] {
            assert!(texto.parse::<Fecha>().is_err(), "debió rechazar {texto:?}");
        }
    }

    #[test]
    fn dias_hasta_cuenta_dias_incluyendo_cambio_de_anio() {
        let a: Fecha = "2026-12-31".parse().unwrap();
        let b: Fecha = "2027-01-02".parse().unwrap();

        assert_eq!(a.dias_hasta(b), 2);
        assert_eq!(b.dias_hasta(a), -2);
    }
}
