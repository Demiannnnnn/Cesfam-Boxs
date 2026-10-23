/**
 * Estado de la pantalla. Fuente única: los componentes se reconstruyen desde aquí
 * y no guardan estado escondido en el DOM.
 */
import { hoy } from "./fechas";
import type { BoxAtencion, EstadoConexion, Horario, Reserva } from "./tipos";

export type Vista = "dia" | "semana";

export interface EstadoPantalla {
  /** Día visible (en la vista semanal, cualquier día de la semana), `AAAA-MM-DD`. */
  fecha: string;
  vista: Vista;
  /** Texto del buscador de funcionario; vacío = sin filtro. */
  filtro: string;
  /** Nombres recientes para autocompletar (`GET /funcionarios`). */
  funcionarios: string[];
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
  vista: "dia",
  filtro: "",
  funcionarios: [],
  horario: null,
  boxes: [],
  reservas: [],
  conexion: { tipo: "buscando" },
  equipo: "",
  ultimoFuncionario: "",
};

/** Nombre del box o un texto de respaldo si ya no está en la lista. */
export function nombreBox(boxId: number): string {
  return estado.boxes.find((b) => b.id === boxId)?.nombre ?? `Box ${boxId}`;
}

/** Etiqueta `HH:MM` del inicio del bloque. */
export function etiquetaBloque(bloque: number): string {
  return estado.horario?.etiquetas[bloque] ?? `#${bloque}`;
}

/** Etiqueta `HH:MM` del término del bloque (inicio del siguiente). */
export function etiquetaFinBloque(bloque: number): string {
  const horario = estado.horario;
  if (!horario) return `#${bloque + 1}`;
  const minutos = horario.hora_inicio * 60 + (bloque + 1) * horario.minutos_por_bloque;
  return `${String(Math.floor(minutos / 60)).padStart(2, "0")}:${String(minutos % 60).padStart(2, "0")}`;
}

/** "09:00 – 11:00" para un rango de bloques inclusivo. */
export function rangoHorario(desde: number, hasta: number): string {
  return `${etiquetaBloque(desde)} – ${etiquetaFinBloque(hasta)}`;
}

/** Minúsculas y sin tildes, para comparar nombres como los escribe la gente. */
function normalizar(texto: string): string {
  return texto.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase().trim();
}

/**
 * Resultado del buscador para un nombre: `null` si no hay filtro, `true` si el
 * nombre lo contiene (sin distinguir tildes ni mayúsculas).
 */
export function coincideFiltro(funcionario: string): boolean | null {
  const filtro = normalizar(estado.filtro);
  return filtro ? normalizar(funcionario).includes(filtro) : null;
}
