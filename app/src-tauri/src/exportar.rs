//! Exportación de reservas a CSV compatible con Excel en español (RF-12, RF-14).

use std::collections::HashMap;

use gestion_box_shared::horario::etiqueta_bloque;
use gestion_box_shared::{BoxAtencion, Etiqueta, Funcionario, Reserva};

/// Marca de orden de bytes: Excel la necesita para leer UTF-8 (tildes, ñ).
const BOM: &str = "\u{FEFF}";
/// Excel en configuración regional es-CL usa `;` como separador de listas.
const SEPARADOR: char = ';';
const ENCABEZADO: [&str; 6] = ["Fecha", "Hora", "Box", "Funcionario", "Etiqueta", "Motivo"];

/// Escapa un campo según RFC 4180 y neutraliza fórmulas (`=`, `+`, `-`, `@`) para
/// que Excel no ejecute contenido escrito por un usuario.
fn campo(valor: &str) -> String {
    let neutralizado = if valor.starts_with(['=', '+', '-', '@']) {
        format!("'{valor}")
    } else {
        valor.to_owned()
    };
    if neutralizado.contains([SEPARADOR, '"', '\n', '\r']) {
        format!("\"{}\"", neutralizado.replace('"', "\"\""))
    } else {
        neutralizado
    }
}

/// Genera el CSV completo (con BOM). Función pura, sin E/S.
///
/// Cubre RF-12 y RF-14.
pub fn generar_csv(
    reservas: &[Reserva],
    boxes: &[BoxAtencion],
    funcionarios: &[Funcionario],
    etiquetas: &[Etiqueta],
) -> String {
    let nombres_box: HashMap<i64, &str> = boxes.iter().map(|b| (b.id, b.nombre.as_str())).collect();
    let nombres_etiqueta: HashMap<i64, &str> = etiquetas
        .iter()
        .map(|e| (e.id, e.nombre.as_str()))
        .collect();
    let etiqueta_por_funcionario: HashMap<i64, &str> = funcionarios
        .iter()
        .filter_map(|f| {
            let etiq_id = f.etiqueta_id?;
            let nombre = nombres_etiqueta.get(&etiq_id)?;
            Some((f.id, *nombre))
        })
        .collect();

    let mut salida = String::from(BOM);
    salida.push_str(&ENCABEZADO.join(&SEPARADOR.to_string()));
    salida.push_str("\r\n");

    for reserva in reservas {
        let fecha = reserva.fecha.como_naive().format("%d-%m-%Y").to_string();
        let box_nombre = nombres_box
            .get(&reserva.box_id)
            .map_or_else(|| format!("Box {}", reserva.box_id), |n| (*n).to_owned());
        let etiqueta_nombre = reserva
            .funcionario_id
            .and_then(|id| etiqueta_por_funcionario.get(&id).copied())
            .unwrap_or_default()
            .to_owned();
        let columnas = [
            fecha,
            etiqueta_bloque(reserva.bloque),
            box_nombre,
            reserva.funcionario.clone(),
            etiqueta_nombre,
            reserva.motivo.clone().unwrap_or_default(),
        ];
        let fila: Vec<String> = columnas.iter().map(|c| campo(c)).collect();
        salida.push_str(&fila.join(&SEPARADOR.to_string()));
        salida.push_str("\r\n");
    }
    salida
}

#[cfg(test)]
mod tests {
    use super::*;
    use gestion_box_shared::Bloque;

    fn reserva(funcionario_id: Option<i64>, funcionario: &str, motivo: Option<&str>) -> Reserva {
        Reserva {
            id: 1,
            box_id: 1,
            fecha: "2026-10-01".parse().unwrap(),
            bloque: Bloque::new(0).unwrap(),
            funcionario_id,
            funcionario: funcionario.into(),
            motivo: motivo.map(Into::into),
            creada_en: "2026-09-30T10:00:00".into(),
            grupo: None,
        }
    }

    fn boxes() -> Vec<BoxAtencion> {
        vec![BoxAtencion {
            id: 1,
            nombre: "Box 1".into(),
            activo: true,
            orden: 1,
            etiqueta_id: None,
        }]
    }

    #[test]
    fn genera_encabezado_y_filas_con_formato_chileno_y_etiqueta() {
        let funcionarios = vec![Funcionario {
            id: 3,
            nombre: "María Pérez".into(),
            etiqueta_id: Some(10),
            activo: true,
        }];
        let etiquetas = vec![Etiqueta {
            id: 10,
            nombre: "Enfermería".into(),
            color: "#2563eb".parse().unwrap(),
            activo: true,
        }];
        let csv = generar_csv(
            &[reserva(Some(3), "María Pérez", Some("Control sano"))],
            &boxes(),
            &funcionarios,
            &etiquetas,
        );

        assert!(csv.starts_with('\u{FEFF}'));
        let lineas: Vec<&str> = csv.trim_start_matches('\u{FEFF}').lines().collect();
        assert_eq!(lineas[0], "Fecha;Hora;Box;Funcionario;Etiqueta;Motivo");
        assert_eq!(
            lineas[1],
            "01-10-2026;08:00;Box 1;María Pérez;Enfermería;Control sano"
        );
    }

    #[test]
    fn escapa_separador_y_comillas() {
        let csv = generar_csv(
            &[reserva(None, "Ana", Some("dice \"hola\"; chao"))],
            &boxes(),
            &[],
            &[],
        );

        assert!(csv.contains("\"dice \"\"hola\"\"; chao\""));
    }

    #[test]
    fn neutraliza_formulas() {
        let csv = generar_csv(
            &[reserva(None, "=HYPERLINK(\"x\")", None)],
            &boxes(),
            &[],
            &[],
        );

        assert!(csv.contains("\"'=HYPERLINK(\"\"x\"\")\""));
    }
}
