/**
 * Agrupa las reservas de un día en tramos: bloques seguidos del mismo box con el
 * mismo funcionario y motivo. La grilla dibuja cada tramo como una sola barra
 * (una mañana reservada se ve como en la planilla, no como 10 celdas repetidas).
 */
import type { Reserva } from "./tipos";

export interface Tramo {
  box_id: number;
  /** Primer bloque del tramo. */
  desde: number;
  /** Último bloque del tramo (inclusivo). */
  hasta: number;
  funcionario_id: number | null;
  funcionario: string;
  motivo: string | null;
  /** Reservas del tramo, ordenadas por bloque. */
  reservas: Reserva[];
}

function continua(tramo: Tramo, reserva: Reserva): boolean {
  return (
    tramo.reservas[0]?.fecha === reserva.fecha &&
    tramo.box_id === reserva.box_id &&
    tramo.hasta + 1 === reserva.bloque &&
    tramo.funcionario_id === reserva.funcionario_id &&
    tramo.funcionario === reserva.funcionario &&
    tramo.motivo === reserva.motivo
  );
}

/** Tramos de un día, indexados por `"boxId-bloqueInicial"`. */
export function agruparEnTramos(reservas: Reserva[]): Map<string, Tramo> {
  const ordenadas = [...reservas].sort((a, b) => a.box_id - b.box_id || a.bloque - b.bloque);
  const tramos = new Map<string, Tramo>();
  let actual: Tramo | null = null;
  for (const reserva of ordenadas) {
    if (actual && continua(actual, reserva)) {
      actual.hasta = reserva.bloque;
      actual.reservas.push(reserva);
      continue;
    }
    actual = {
      box_id: reserva.box_id,
      desde: reserva.bloque,
      hasta: reserva.bloque,
      funcionario_id: reserva.funcionario_id,
      funcionario: reserva.funcionario,
      motivo: reserva.motivo,
      reservas: [reserva],
    };
    tramos.set(`${reserva.box_id}-${reserva.bloque}`, actual);
  }
  return tramos;
}

/**
 * Agrupa reservas de una o varias fechas en una lista de tramos ordenada por
 * `(fecha, box_id, desde)` reutilizando `agruparEnTramos` por cada día (RF-15).
 */
export function listarTramos(reservas: Reserva[]): Tramo[] {
  const porFecha = new Map<string, Reserva[]>();
  for (const r of reservas) {
    const lista = porFecha.get(r.fecha) ?? [];
    lista.push(r);
    porFecha.set(r.fecha, lista);
  }
  const resultado: Tramo[] = [];
  for (const fecha of [...porFecha.keys()].sort()) {
    const delDia = porFecha.get(fecha) ?? [];
    resultado.push(...agruparEnTramos(delDia).values());
  }
  return resultado;
}
