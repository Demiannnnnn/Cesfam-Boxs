/**
 * Estado de la pantalla. Fuente única: los componentes se reconstruyen desde aquí
 * y no guardan estado escondido en el DOM.
 */
import { hoy } from "./fechas";
import type { BoxAtencion, EstadoConexion, Etiqueta, Funcionario, Horario, Reserva } from "./tipos";

export type Vista = "dia" | "semana";

export interface EstadoPantalla {
  /** Día visible (en la vista semanal, cualquier día de la semana), `AAAA-MM-DD`. */
  fecha: string;
  vista: Vista;
  /** Texto del buscador de funcionario; vacío = sin filtro. */
  filtro: string;
  /** Identificador de etiqueta seleccionada en la barra de navegación; `null` = todas. */
  filtroEtiquetaId: number | null;
  /** Etiquetas registradas (`GET /etiquetas?incluir_inactivos=true`). */
  etiquetas: Etiqueta[];
  /** Funcionarios registrados (`GET /funcionarios?incluir_inactivos=true`). */
  funcionarios: Funcionario[];
  horario: Horario | null;
  boxes: BoxAtencion[];
  reservas: Reserva[];
  conexion: EstadoConexion;
  /** Nombre del PC servidor de la última conexión exitosa. */
  equipo: string;
  /** Identificador del último funcionario usado al reservar en este PC. */
  ultimoFuncionarioId: number | null;
}

export const estado: EstadoPantalla = {
  fecha: hoy(),
  vista: "dia",
  filtro: "",
  filtroEtiquetaId: null,
  etiquetas: [],
  funcionarios: [],
  horario: null,
  boxes: [],
  reservas: [],
  conexion: { tipo: "buscando" },
  equipo: "",
  ultimoFuncionarioId: null,
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
export function normalizarTexto(texto: string): string {
  return texto.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase().trim();
}

/** Etiquetas activas, ordenadas por nombre. */
export function etiquetasActivas(): Etiqueta[] {
  return estado.etiquetas.filter((e) => e.activo);
}

/** Funcionarios activos, ordenados por nombre. */
export function funcionariosActivos(): Funcionario[] {
  return estado.funcionarios.filter((f) => f.activo);
}

/** Busca un funcionario activo cuyo nombre coincida sin distinguir mayúsculas ni tildes. */
export function buscarFuncionarioActivoPorNombre(nombre: string): Funcionario | null {
  const buscado = normalizarTexto(nombre);
  if (!buscado) return null;
  return estado.funcionarios.find((f) => f.activo && normalizarTexto(f.nombre) === buscado) ?? null;
}

/** Etiqueta activa de un box, o `null` si no tiene o fue desactivada. */
export function etiquetaDeBox(box: BoxAtencion): Etiqueta | null {
  if (box.etiqueta_id === null) return null;
  return estado.etiquetas.find((e) => e.id === box.etiqueta_id && e.activo) ?? null;
}

/** Etiqueta activa del funcionario de una reserva (por `funcionario_id`), o `null`. */
export function etiquetaDeFuncionario(funcionarioId: number | null): Etiqueta | null {
  if (funcionarioId === null) return null;
  const funcionario = estado.funcionarios.find((f) => f.id === funcionarioId);
  if (!funcionario || funcionario.etiqueta_id === null) return null;
  return estado.etiquetas.find((e) => e.id === funcionario.etiqueta_id && e.activo) ?? null;
}

function coincideFiltroFuncionario(
  filtro: string,
  item: { funcionario_id: number | null; nombre?: string },
): boolean {
  const exacto = estado.funcionarios.find((f) => normalizarTexto(f.nombre) === filtro);
  if (exacto) {
    if (item.funcionario_id !== null) return item.funcionario_id === exacto.id;
    return item.nombre ? normalizarTexto(item.nombre) === filtro : false;
  }

  if (item.funcionario_id !== null) {
    const func = estado.funcionarios.find((f) => f.id === item.funcionario_id);
    if (func) return normalizarTexto(func.nombre).includes(filtro);
  }
  return item.nombre ? normalizarTexto(item.nombre).includes(filtro) : false;
}

function coincideFiltroEtiqueta(
  etiquetaId: number,
  item: { box_id?: number; funcionario_id: number | null },
): boolean {
  const etiqFunc =
    item.funcionario_id !== null
      ? estado.funcionarios.find((f) => f.id === item.funcionario_id)?.etiqueta_id ?? null
      : null;
  if (etiqFunc === etiquetaId) return true;
  if (item.box_id !== undefined) {
    const etiqBox = estado.boxes.find((b) => b.id === item.box_id)?.etiqueta_id ?? null;
    if (etiqBox === etiquetaId) return true;
  }
  return false;
}

/**
 * Resultado de los filtros rápidos de la barra de navegación (funcionario y etiqueta)
 * para una reserva o tramo:
 * `null` si no hay ningún filtro activo, `true` si cumple los filtros activos,
 * `false` en caso contrario.
 */
export function coincideFiltro(item: {
  box_id?: number;
  funcionario_id: number | null;
  nombre?: string;
}): boolean | null {
  const filtro = normalizarTexto(estado.filtro);
  const etiquetaId = estado.filtroEtiquetaId;
  if (!filtro && etiquetaId === null) return null;

  const porFunc = filtro ? coincideFiltroFuncionario(filtro, item) : true;
  const porEtiq = etiquetaId !== null ? coincideFiltroEtiqueta(etiquetaId, item) : true;
  return porFunc && porEtiq;
}
