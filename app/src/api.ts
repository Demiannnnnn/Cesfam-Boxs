/**
 * Envoltorios tipados de `invoke`. Única puerta hacia el backend Rust: toda la
 * red ocurre allá (ADR-007).
 */
import { invoke } from "@tauri-apps/api/core";

import { textos } from "./textos";
import type {
  BoxAtencion,
  CambiosBox,
  CambiosEtiqueta,
  CambiosFuncionario,
  ErrorUi,
  Etiqueta,
  FiltroBoxesLibres,
  FiltroBuscarReservas,
  Funcionario,
  Horario,
  NuevoLote,
  Preferencias,
  Reserva,
  ResultadoConexion,
  ResultadoLote,
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
  crearBox: (nombre: string, etiquetaId: number | null) =>
    llamar<BoxAtencion>("crear_box", { nombre, etiquetaId }),
  actualizarBox: (id: number, cambios: CambiosBox) =>
    llamar<BoxAtencion>("actualizar_box", { id, cambios }),
  boxesLibres: (filtro: FiltroBoxesLibres) =>
    llamar<BoxAtencion[]>("boxes_libres", {
      fecha: filtro.fecha,
      desde: filtro.desde,
      hasta: filtro.hasta,
      etiquetaId: filtro.etiqueta_id,
    }),
  listarEtiquetas: (incluirInactivos: boolean) =>
    llamar<Etiqueta[]>("listar_etiquetas", { incluirInactivos }),
  crearEtiqueta: (nombre: string, color: string) =>
    llamar<Etiqueta>("crear_etiqueta", { nombre, color }),
  actualizarEtiqueta: (id: number, cambios: CambiosEtiqueta) =>
    llamar<Etiqueta>("actualizar_etiqueta", { id, cambios }),
  listarFuncionarios: (incluirInactivos: boolean) =>
    llamar<Funcionario[]>("listar_funcionarios", { incluirInactivos }),
  crearFuncionario: (nombre: string, etiquetaId: number | null) =>
    llamar<Funcionario>("crear_funcionario", { nombre, etiquetaId }),
  actualizarFuncionario: (id: number, cambios: CambiosFuncionario) =>
    llamar<Funcionario>("actualizar_funcionario", { id, cambios }),
  listarReservas: (fecha: string) => llamar<Reserva[]>("listar_reservas", { fecha }),
  listarReservasRango: (desde: string, hasta: string) =>
    llamar<Reserva[]>("listar_reservas_rango", { desde, hasta }),
  buscarReservas: (filtro: FiltroBuscarReservas) =>
    llamar<Reserva[]>("buscar_reservas", {
      desde: filtro.desde,
      hasta: filtro.hasta,
      funcionarioId: filtro.funcionario_id,
      etiquetaId: filtro.etiqueta_id,
      boxId: filtro.box_id,
    }),
  crearReservas: (lote: NuevoLote) => llamar<ResultadoLote>("crear_reservas", { lote }),
  cancelarReserva: (id: number) => llamar<void>("cancelar_reserva", { id }),
  cancelarGrupo: (grupo: number, desde: string) =>
    llamar<{ canceladas: number }>("cancelar_grupo", { grupo, desde }),
  exportarCsv: (desde: string, hasta: string, ruta: string) =>
    llamar<number>("exportar_csv", { desde, hasta, ruta }),
};
