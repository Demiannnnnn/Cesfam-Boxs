/**
 * Envoltorios tipados de `invoke`. Única puerta hacia el backend Rust: toda la
 * red ocurre allá (ADR-007).
 */
import { invoke } from "@tauri-apps/api/core";

import { textos } from "./textos";
import type {
  BoxAtencion,
  CambiosBox,
  ErrorUi,
  Horario,
  NuevaReserva,
  Preferencias,
  Reserva,
  ResultadoConexion,
} from "./tipos";

/** Convierte cualquier rechazo de `invoke` en un `ErrorUi`. */
export function comoErrorUi(error: unknown): ErrorUi {
  if (
    typeof error === "object" &&
    error !== null &&
    "codigo" in error &&
    "mensaje" in error &&
    typeof error.mensaje === "string"
  ) {
    return { codigo: String(error.codigo), mensaje: error.mensaje };
  }
  console.error("Error no tipado desde el backend", error);
  return { codigo: "interno", mensaje: textos.comunes.errorDesconocido };
}

async function llamar<T>(comando: string, argumentos?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(comando, argumentos);
  } catch (error) {
    throw comoErrorUi(error);
  }
}

export const api = {
  horario: () => llamar<Horario>("horario"),
  preferencias: () => llamar<Preferencias>("preferencias"),
  conectar: () => llamar<ResultadoConexion>("conectar"),
  conectarManual: (direccion: string) =>
    llamar<ResultadoConexion>("conectar_manual", { direccion }),
  listarBoxes: (incluirInactivos: boolean) =>
    llamar<BoxAtencion[]>("listar_boxes", { incluirInactivos }),
  crearBox: (nombre: string) => llamar<BoxAtencion>("crear_box", { nombre }),
  actualizarBox: (id: number, cambios: CambiosBox) =>
    llamar<BoxAtencion>("actualizar_box", { id, cambios }),
  listarReservas: (fecha: string) => llamar<Reserva[]>("listar_reservas", { fecha }),
  crearReserva: (nueva: NuevaReserva) => llamar<Reserva>("crear_reserva", { nueva }),
  cancelarReserva: (id: number) => llamar<void>("cancelar_reserva", { id }),
  exportarCsv: (desde: string, hasta: string, ruta: string) =>
    llamar<number>("exportar_csv", { desde, hasta, ruta }),
};
