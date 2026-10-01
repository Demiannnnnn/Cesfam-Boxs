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
    fn campos_desconocidos_se_rechazan() {
        assert!(serde_json::from_str::<NuevoBox>(r#"{"nombre":"a","x":1}"#).is_err());
    }
}
