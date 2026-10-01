/**
 * Estado de la pantalla. Fuente única: los componentes se reconstruyen desde aquí
 * y no guardan estado escondido en el DOM.
 */
import { hoy } from "./fechas";
import type { BoxAtencion, EstadoConexion, Horario, Reserva } from "./tipos";

export interface EstadoPantalla {
  /** Día visible, `AAAA-MM-DD`. */
  fecha: string;
  horario: Horario | null;
  boxes: BoxAtencion[];
  reservas: Reserva[];
  conexion: EstadoConexion;
  /** Nombre del PC servidor de la última conexión exitosa. */
  equipo: string;
  ultimoFuncionario: string;
}

export const estado: EstadoPantalla = {
  fecha: hoy(),
  horario: null,
  boxes: [],
  reservas: [],
  conexion: { tipo: "buscando" },
  equipo: "",
  ultimoFuncionario: "",
};

/** Índice `"boxId-bloque"` → reserva, para pintar la grilla en O(1) por celda. */
export function indiceReservas(reservas: Reserva[]): Map<string, Reserva> {
  return new Map(reservas.map((r) => [`${r.box_id}-${r.bloque}`, r]));
}

/** Nombre del box o un texto de respaldo si ya no está en la lista. */
export function nombreBox(boxId: number): string {
  return estado.boxes.find((b) => b.id === boxId)?.nombre ?? `Box ${boxId}`;
}

/** Etiqueta `HH:MM` del bloque. */
export function etiquetaBloque(bloque: number): string {
  return estado.horario?.etiquetas[bloque] ?? `#${bloque}`;
}
