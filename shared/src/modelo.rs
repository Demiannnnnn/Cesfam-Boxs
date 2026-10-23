//! Entidades del dominio y DTOs del contrato HTTP (ver `docs/04-api.md`).

use serde::{Deserialize, Serialize};

use crate::tipos::{Bloque, Fecha};

/// Box de atención. Se llama `BoxAtencion` para no chocar con `std::boxed::Box`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxAtencion {
    pub id: i64,
    pub nombre: String,
    pub activo: bool,
    /// Posición de la columna en la grilla (menor = más a la izquierda).
    pub orden: i64,
}

/// Reserva de un box en una fecha y bloque.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reserva {
    pub id: i64,
    pub box_id: i64,
    pub fecha: Fecha,
    pub bloque: Bloque,
    pub funcionario: String,
    pub motivo: Option<String>,
    /// Fecha y hora local de creación, `AAAA-MM-DDTHH:MM:SS`.
    pub creada_en: String,
    /// Reservas creadas juntas (un rango de bloques o una serie semanal) comparten
    /// grupo, para cancelarlas de una vez. `None` en reservas sueltas.
    #[serde(default)]
    pub grupo: Option<i64>,
}

/// Cuerpo de `POST /boxes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoBox {
    pub nombre: String,
}

/// Cuerpo de `PATCH /boxes/{id}`. Los campos ausentes no se modifican.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CambiosBox {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nombre: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activo: Option<bool>,
}

/// Cuerpo de `POST /reservas`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevaReserva {
    pub box_id: i64,
    pub fecha: Fecha,
    pub bloque: Bloque,
    pub funcionario: String,
    #[serde(default)]
    pub motivo: Option<String>,
}

/// Cuerpo de `POST /reservas/lote`: los bloques `desde..=hasta` de un box en cada
/// una de las `fechas` (un rango del día, o una serie semanal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoLote {
    pub box_id: i64,
    pub fechas: Vec<Fecha>,
    pub desde: Bloque,
    pub hasta: Bloque,
    pub funcionario: String,
    #[serde(default)]
    pub motivo: Option<String>,
    /// `false` (por defecto): si un bloque está ocupado no se crea nada (409).
    /// `true`: se crean los libres y se informan los ocupados en `omitidos`.
    #[serde(default)]
    pub omitir_ocupados: bool,
}

/// Bloque que no se reservó porque ya estaba ocupado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BloqueOmitido {
    pub fecha: Fecha,
    pub bloque: Bloque,
}

/// Respuesta de `POST /reservas/lote`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultadoLote {
    pub creadas: Vec<Reserva>,
    pub omitidos: Vec<BloqueOmitido>,
}

/// Respuesta de `DELETE /reservas/grupo/{grupo}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultadoCancelacion {
    pub canceladas: usize,
}

/// Respuesta de `GET /health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstadoSalud {
    pub estado: String,
    pub version: String,
    /// Nombre de equipo del PC servidor (se muestra en la app).
    pub equipo: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserva_ida_y_vuelta_json() {
        let reserva = Reserva {
            id: 1,
            box_id: 2,
            fecha: "2026-10-01".parse().unwrap(),
            bloque: Bloque::new(4).unwrap(),
            funcionario: "María Pérez".into(),
            motivo: None,
            creada_en: "2026-09-30T16:12:03".into(),
            grupo: None,
        };

        let json = serde_json::to_string(&reserva).unwrap();
        let de_vuelta: Reserva = serde_json::from_str(&json).unwrap();

        assert_eq!(de_vuelta, reserva);
        assert!(json.contains("\"fecha\":\"2026-10-01\""));
        assert!(json.contains("\"bloque\":4"));
    }

    #[test]
    fn nueva_reserva_con_bloque_invalido_no_se_deserializa() {
        let json = r#"{"box_id":1,"fecha":"2026-10-01","bloque":24,"funcionario":"x"}"#;

        assert!(serde_json::from_str::<NuevaReserva>(json).is_err());
    }

    #[test]
    fn nueva_reserva_sin_motivo_es_valida() {
        let json = r#"{"box_id":1,"fecha":"2026-10-01","bloque":3,"funcionario":"x"}"#;

        let nueva: NuevaReserva = serde_json::from_str(json).unwrap();

        assert_eq!(nueva.motivo, None);
    }

    #[test]
    fn reserva_sin_grupo_en_el_json_se_acepta() {
        let json = r#"{"id":1,"box_id":1,"fecha":"2026-10-01","bloque":0,"funcionario":"x","motivo":null,"creada_en":"2026-10-01T08:00:00"}"#;

        assert_eq!(serde_json::from_str::<Reserva>(json).unwrap().grupo, None);
    }

    #[test]
    fn lote_sin_omitir_ocupados_es_todo_o_nada() {
        let json = r#"{"box_id":1,"fechas":["2026-10-01"],"desde":0,"hasta":11,"funcionario":"x"}"#;

        assert!(
            !serde_json::from_str::<NuevoLote>(json)
                .unwrap()
                .omitir_ocupados
        );
    }

    #[test]
    fn campos_desconocidos_se_rechazan() {
        assert!(serde_json::from_str::<NuevoBox>(r#"{"nombre":"a","x":1}"#).is_err());
    }
}
