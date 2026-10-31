//! Entidades del dominio y DTOs del contrato HTTP (ver `docs/04-api.md`).

use serde::{Deserialize, Deserializer, Serialize};

use crate::tipos::{Bloque, Color, Fecha};

/// Deserializa un campo opcional que distingue entre ausente (`None`),
/// explícitamente nulo (`Some(None)`) y con valor (`Some(Some(v))`).
/// Se usa junto con `#[serde(default)]`.
fn deserializar_doble_opcion<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Etiqueta con nombre y color asignable a boxes y funcionarios.
///
/// Cubre RF-14.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Etiqueta {
    pub id: i64,
    pub nombre: String,
    pub color: Color,
    pub activo: bool,
}

/// Cuerpo de `POST /etiquetas`. Cubre RF-14.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevaEtiqueta {
    pub nombre: String,
    pub color: Color,
}

/// Cuerpo de `PATCH /etiquetas/{id}`. Los campos ausentes no se modifican. Cubre RF-14.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CambiosEtiqueta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nombre: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activo: Option<bool>,
}

/// Funcionario registrado en el CESFAM.
///
/// Cubre RF-13 y RF-14.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Funcionario {
    pub id: i64,
    pub nombre: String,
    #[serde(default)]
    pub etiqueta_id: Option<i64>,
    pub activo: bool,
}

/// Cuerpo de `POST /funcionarios`. Cubre RF-13 y RF-14.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoFuncionario {
    pub nombre: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etiqueta_id: Option<i64>,
}

/// Cuerpo de `PATCH /funcionarios/{id}`. Los campos ausentes no se modifican;
/// `"etiqueta_id": null` quita la etiqueta asignada. Cubre RF-13 y RF-14.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CambiosFuncionario {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nombre: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserializar_doble_opcion"
    )]
    pub etiqueta_id: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activo: Option<bool>,
}

/// Box de atención. Se llama `BoxAtencion` para no chocar con `std::boxed::Box`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxAtencion {
    pub id: i64,
    pub nombre: String,
    pub activo: bool,
    /// Posición de la fila en la grilla.
    pub orden: i64,
    /// Etiqueta opcional del box (p. ej. especialidad). Cubre RF-14.
    #[serde(default)]
    pub etiqueta_id: Option<i64>,
}

/// Reserva de un box en una fecha y bloque.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reserva {
    pub id: i64,
    pub box_id: i64,
    pub fecha: Fecha,
    pub bloque: Bloque,
    /// Identificador del funcionario registrado (RF-13).
    #[serde(default)]
    pub funcionario_id: Option<i64>,
    /// Nombre resuelto del funcionario (desde la tabla `funcionario` si existe).
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etiqueta_id: Option<i64>,
}

/// Cuerpo de `PATCH /boxes/{id}`. Los campos ausentes no se modifican;
/// `"etiqueta_id": null` quita la etiqueta asignada.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CambiosBox {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nombre: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activo: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserializar_doble_opcion"
    )]
    pub etiqueta_id: Option<Option<i64>>,
}

/// Cuerpo de `POST /reservas`. Cubre RF-03 y RF-13.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevaReserva {
    pub box_id: i64,
    pub fecha: Fecha,
    pub bloque: Bloque,
    pub funcionario_id: i64,
    #[serde(default)]
    pub motivo: Option<String>,
}

/// Cuerpo de `POST /reservas/lote`: los bloques `desde..=hasta` de un box en cada
/// una de las `fechas` (un rango del día, o una serie semanal). Cubre RF-03 y RF-13.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoLote {
    pub box_id: i64,
    pub fechas: Vec<Fecha>,
    pub desde: Bloque,
    pub hasta: Bloque,
    pub funcionario_id: i64,
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

/// Parámetros de consulta de `GET /reservas/buscar`. Cubre RF-15.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultaBuscarReservas {
    pub desde: Fecha,
    pub hasta: Fecha,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funcionario_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etiqueta_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_id: Option<i64>,
}

/// Parámetros de consulta de `GET /boxes/libres`. Cubre RF-15.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsultaBoxesLibres {
    pub fecha: Fecha,
    pub desde: Bloque,
    pub hasta: Bloque,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etiqueta_id: Option<i64>,
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
            funcionario_id: Some(7),
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
        assert!(json.contains("\"funcionario_id\":7"));
    }

    #[test]
    fn nueva_reserva_con_bloque_invalido_no_se_deserializa() {
        let json = r#"{"box_id":1,"fecha":"2026-10-01","bloque":24,"funcionario_id":1}"#;

        assert!(serde_json::from_str::<NuevaReserva>(json).is_err());
    }

    #[test]
    fn nueva_reserva_con_funcionario_texto_se_rechaza() {
        let json = r#"{"box_id":1,"fecha":"2026-10-01","bloque":3,"funcionario":"Ana"}"#;

        assert!(serde_json::from_str::<NuevaReserva>(json).is_err());
    }

    #[test]
    fn nueva_reserva_sin_motivo_es_valida() {
        let json = r#"{"box_id":1,"fecha":"2026-10-01","bloque":3,"funcionario_id":5}"#;

        let nueva: NuevaReserva = serde_json::from_str(json).unwrap();

        assert_eq!(nueva.motivo, None);
        assert_eq!(nueva.funcionario_id, 5);
    }

    #[test]
    fn reserva_sin_grupo_ni_funcionario_id_en_el_json_se_acepta() {
        let json = r#"{"id":1,"box_id":1,"fecha":"2026-10-01","bloque":0,"funcionario":"x","motivo":null,"creada_en":"2026-10-01T08:00:00"}"#;

        let r: Reserva = serde_json::from_str(json).unwrap();
        assert_eq!(r.grupo, None);
        assert_eq!(r.funcionario_id, None);
    }

    #[test]
    fn lote_sin_omitir_ocupados_es_todo_o_nada() {
        let json =
            r#"{"box_id":1,"fechas":["2026-10-01"],"desde":0,"hasta":11,"funcionario_id":2}"#;

        assert!(
            !serde_json::from_str::<NuevoLote>(json)
                .unwrap()
                .omitir_ocupados
        );
    }

    #[test]
    fn doble_opcion_distingue_ausente_nulo_y_valor() {
        let ausente: CambiosFuncionario = serde_json::from_str(r#"{"nombre":"Ana"}"#).unwrap();
        let nulo: CambiosFuncionario = serde_json::from_str(r#"{"etiqueta_id":null}"#).unwrap();
        let con_valor: CambiosFuncionario = serde_json::from_str(r#"{"etiqueta_id":3}"#).unwrap();

        assert_eq!(ausente.etiqueta_id, None);
        assert_eq!(nulo.etiqueta_id, Some(None));
        assert_eq!(con_valor.etiqueta_id, Some(Some(3)));

        assert_eq!(
            serde_json::to_string(&nulo).unwrap(),
            r#"{"etiqueta_id":null}"#
        );
        assert_eq!(
            serde_json::to_string(&con_valor).unwrap(),
            r#"{"etiqueta_id":3}"#
        );
    }

    #[test]
    fn campos_desconocidos_se_rechazan() {
        assert!(serde_json::from_str::<NuevoBox>(r#"{"nombre":"a","x":1}"#).is_err());
    }
}
