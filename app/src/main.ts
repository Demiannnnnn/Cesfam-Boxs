/**
 * Arranque y ciclo de refresco de la app.
 *
 * - Al iniciar: horario → conectar (guardada → localhost → broadcast) → grilla,
 *   o pantalla de conexión manual si no se encuentra el servidor (RF-09).
 * - Cada 20 s refresca boxes y reservas del día visible (RF-07); si falla, marca
 *   "Sin conexión" (RF-08) y en el siguiente ciclo intenta redescubrir el servidor.
 */
import { api } from "./api";
import { mostrarAviso } from "./componentes/aviso";
import { renderizarConexion } from "./componentes/conexion";
import { alCerrarDialogos, hayDialogoAbierto } from "./componentes/dialogo";
import { renderizarGrilla } from "./componentes/grilla";
import { abrirBoxes, abrirConexionManual, abrirDetalle, abrirExportar, abrirReservar } from "./componentes/modales";
import { porId } from "./dom";
import { estado } from "./estado";
import { fechaLarga, hoy, sumarDias } from "./fechas";
import type { ErrorUi, ResultadoConexion } from "./tipos";

/** Intervalo de refresco automático (RF-07 pide entre 15 y 30 s). */
const INTERVALO_REFRESCO_MS = 20_000;

const elementos = {
  conexion: porId<HTMLDivElement>("conexion"),
  grilla: porId<HTMLElement>("grilla"),
  selectorFecha: porId<HTMLInputElement>("selector-fecha"),
  fechaLarga: porId<HTMLHeadingElement>("fecha-larga"),
};

let temporizador: number | undefined;
let refrescando = false;

function renderizar(): void {
  renderizarConexion(elementos.conexion, estado.conexion);
  elementos.selectorFecha.value = estado.fecha;
  elementos.fechaLarga.textContent = fechaLarga(estado.fecha);
  renderizarGrilla(elementos.grilla, {
    alElegirLibre: (box, bloque) =>
      abrirReservar(box, bloque, {
        alReservar: () => void refrescar(),
        alConflicto: (error) => {
          mostrarAviso(error.mensaje);
          void refrescar();
        },
      }),
    alElegirOcupada: (reserva) => abrirDetalle(reserva, () => void refrescar()),
  });
}

/** Pide boxes y reservas del día visible. Nunca lanza. */
async function refrescar(): Promise<void> {
  if (refrescando) return;
  refrescando = true;
  try {
    if (estado.conexion.tipo === "sin_conexion") {
      await reconectarEnSilencio();
    }
    const fecha = estado.fecha;
    const [boxes, reservas] = await Promise.all([api.listarBoxes(false), api.listarReservas(fecha)]);
    // Si el usuario cambió de día mientras llegaba la respuesta, se descarta.
    if (fecha !== estado.fecha) return;
    estado.boxes = boxes;
    estado.reservas = reservas;
    estado.conexion = { tipo: "conectado", equipo: estado.equipo };
  } catch (error) {
    const e = error as ErrorUi;
    if (e.codigo === "sin_conexion" || e.codigo === "nombre_no_resuelto") {
      estado.conexion = { tipo: "sin_conexion", mensaje: e.mensaje };
    } else {
      mostrarAviso(e.mensaje);
    }
  } finally {
    refrescando = false;
    renderizar();
  }
}

/** Intenta redescubrir el servidor (puede haber cambiado de IP). */
async function reconectarEnSilencio(): Promise<void> {
  const resultado = await api.conectar();
  if (resultado.tipo === "conectado") estado.equipo = resultado.equipo;
}

function programarRefresco(): void {
  window.clearTimeout(temporizador);
  temporizador = window.setTimeout(async () => {
    // Con un modal abierto no se mueve la grilla bajo el usuario; se refresca al cerrar.
    if (!hayDialogoAbierto()) await refrescar();
    programarRefresco();
  }, INTERVALO_REFRESCO_MS);
}

function irA(fecha: string): void {
  if (!fecha || fecha === estado.fecha) return;
  estado.fecha = fecha;
  estado.reservas = [];
  renderizar();
  void refrescar();
}

function alConectar(resultado: ResultadoConexion): void {
  if (resultado.tipo !== "conectado") return;
  estado.equipo = resultado.equipo;
  estado.conexion = { tipo: "conectado", equipo: resultado.equipo };
  void refrescar();
}

function enlazarEventos(): void {
  porId("btn-anterior").addEventListener("click", () => irA(sumarDias(estado.fecha, -1)));
  porId("btn-siguiente").addEventListener("click", () => irA(sumarDias(estado.fecha, 1)));
  porId("btn-hoy").addEventListener("click", () => irA(hoy()));
  elementos.selectorFecha.addEventListener("change", () => irA(elementos.selectorFecha.value));
  porId("btn-boxes").addEventListener("click", () => abrirBoxes(() => void refrescar()));
  porId("btn-exportar").addEventListener("click", () => abrirExportar());
  alCerrarDialogos(() => void refrescar());
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible" && !hayDialogoAbierto()) void refrescar();
  });
}

async function iniciar(): Promise<void> {
  enlazarEventos();
  renderizar();
  try {
    const [horario, preferencias] = await Promise.all([api.horario(), api.preferencias()]);
    estado.horario = horario;
    estado.ultimoFuncionario = preferencias.ultimo_funcionario ?? "";
    renderizar();

    const resultado = await api.conectar();
    if (resultado.tipo === "conectado") {
      alConectar(resultado);
    } else {
      estado.conexion = { tipo: "sin_conexion", mensaje: resultado.mensaje };
      renderizar();
      abrirConexionManual(resultado.mensaje, alConectar);
    }
  } catch (error) {
    mostrarAviso((error as ErrorUi).mensaje);
  }
  programarRefresco();
}

void iniciar();
