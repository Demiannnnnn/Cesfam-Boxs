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
  funcionario: string;
  motivo: string | null;
  /** Reservas del tramo, ordenadas por bloque. */
  reservas: Reserva[];
}

function continua(tramo: Tramo, reserva: Reserva): boolean {
  return (
    tramo.box_id === reserva.box_id &&
    tramo.hasta + 1 === reserva.bloque &&
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
      funcionario: reserva.funcionario,
      motivo: reserva.motivo,
      reservas: [reserva],
    };
    tramos.set(`${reserva.box_id}-${reserva.bloque}`, actual);
  }
  return tramos;
}
