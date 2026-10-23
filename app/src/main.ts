/**
 * Arranque y ciclo de refresco de la app.
 *
 * - Al iniciar: horario → conectar (guardada → localhost → broadcast) → grilla,
 *   o pantalla de conexión manual si no se encuentra el servidor (RF-09).
 * - Cada 20 s refresca boxes y reservas del día o la semana visible (RF-07); si
 *   falla, marca "Sin conexión" (RF-08) y en el siguiente ciclo intenta redescubrir
 *   el servidor.
 */
import { api } from "./api";
import { mostrarAviso } from "./componentes/aviso";
import { renderizarConexion } from "./componentes/conexion";
import { alCerrarDialogos, hayDialogoAbierto } from "./componentes/dialogo";
import { renderizarGrilla } from "./componentes/grilla";
import { abrirBoxes, abrirConexionManual, abrirDetalle, abrirExportar } from "./componentes/modales";
import { abrirReservar, type AccionesReserva } from "./componentes/reservar";
import { renderizarSemana } from "./componentes/semana";
import { el, porId } from "./dom";
import { estado, type Vista } from "./estado";
import { diaHabil, diasHabilesDeSemana, fechaLarga, hoy, inicioSemana, sumarDias, tituloSemana } from "./fechas";
import { textos } from "./textos";
import type { ErrorUi, Reserva, ResultadoConexion } from "./tipos";

/** Intervalo de refresco automático (RF-07 pide entre 15 y 30 s). */
const INTERVALO_REFRESCO_MS = 20_000;

const elementos = {
  conexion: porId<HTMLDivElement>("conexion"),
  grilla: porId<HTMLElement>("grilla"),
  selectorFecha: porId<HTMLInputElement>("selector-fecha"),
  fechaLarga: porId<HTMLHeadingElement>("fecha-larga"),
  vistaDia: porId<HTMLButtonElement>("btn-vista-dia"),
  vistaSemana: porId<HTMLButtonElement>("btn-vista-semana"),
  filtro: porId<HTMLInputElement>("filtro-funcionario"),
  listaFuncionarios: porId<HTMLDataListElement>("lista-funcionarios"),
};

let temporizador: number | undefined;
let refrescando = false;

const accionesReserva: AccionesReserva = {
  alReservar: (resultado) => {
    const dias = new Set(resultado.creadas.map((r) => r.fecha)).size;
    if (resultado.omitidos.length > 0) mostrarAviso(textos.reserva.parcial(resultado.omitidos.length), "exito");
    else mostrarAviso(textos.reserva.listo(dias), "exito");
    void refrescar();
    void cargarFuncionarios();
  },
  alConflicto: (error) => {
    mostrarAviso(error.mensaje);
    void refrescar();
  },
};

function renderizar(): void {
  renderizarConexion(elementos.conexion, estado.conexion);
  elementos.selectorFecha.value = estado.fecha;
  elementos.vistaDia.setAttribute("aria-pressed", String(estado.vista === "dia"));
  elementos.vistaSemana.setAttribute("aria-pressed", String(estado.vista === "semana"));

  if (estado.vista === "semana") {
    elementos.fechaLarga.textContent = tituloSemana(estado.fecha);
    renderizarSemana(elementos.grilla, {
      alReservar: (box, fecha, desde) => abrirReservar(box, fecha, desde, null, accionesReserva),
      alVerDia: (fecha) => cambiarVista("dia", fecha),
    });
    return;
  }
  elementos.fechaLarga.textContent = fechaLarga(estado.fecha);
  renderizarGrilla(elementos.grilla, {
    alElegirLibre: (box, desde, hasta) => abrirReservar(box, estado.fecha, desde, hasta, accionesReserva),
    alElegirOcupada: (tramo) => abrirDetalle(tramo, () => void refrescar()),
  });
}

/** Reservas de lo que se ve: el día, o de lunes a viernes en la vista semanal. */
function pedirReservas(vista: Vista, fecha: string): Promise<Reserva[]> {
  if (vista === "dia") return api.listarReservas(fecha);
  const dias = diasHabilesDeSemana(fecha);
  return api.listarReservasRango(dias[0] ?? fecha, dias[dias.length - 1] ?? fecha);
}

/** Pide boxes y reservas de lo visible. Nunca lanza. */
async function refrescar(): Promise<void> {
  if (refrescando) return;
  refrescando = true;
  try {
    if (estado.conexion.tipo === "sin_conexion") {
      await reconectarEnSilencio();
    }
    const { vista, fecha } = estado;
    const [boxes, reservas] = await Promise.all([api.listarBoxes(false), pedirReservas(vista, fecha)]);
    // Si el usuario cambió de día o de vista mientras llegaba la respuesta, se descarta.
    if (fecha !== estado.fecha || vista !== estado.vista) return;
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

/** Llena las sugerencias de nombres (modal de reserva y buscador). No es crítico: si falla, se omite. */
async function cargarFuncionarios(): Promise<void> {
  try {
    estado.funcionarios = await api.listarFuncionarios();
    elementos.listaFuncionarios.replaceChildren(
      ...estado.funcionarios.map((nombre) => el("option", { atributos: { value: nombre } })),
    );
  } catch {
    // Sin sugerencias se puede escribir igual; el próximo intento las recupera.
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

function cambiarVista(vista: Vista, fecha: string = estado.fecha): void {
  if (vista === estado.vista && fecha === estado.fecha) return;
  estado.vista = vista;
  estado.fecha = fecha;
  estado.reservas = [];
  renderizar();
  void refrescar();
}

/** Anterior / Siguiente: un día hábil en la vista diaria, una semana en la semanal. */
function avanzar(sentido: 1 | -1): void {
  if (estado.vista === "semana") irA(sumarDias(inicioSemana(estado.fecha), 7 * sentido));
  else irA(diaHabil(estado.fecha, sentido));
}

function alConectar(resultado: ResultadoConexion): void {
  if (resultado.tipo !== "conectado") return;
  estado.equipo = resultado.equipo;
  estado.conexion = { tipo: "conectado", equipo: resultado.equipo };
  void refrescar();
  void cargarFuncionarios();
}

function enlazarEventos(): void {
  porId("btn-anterior").addEventListener("click", () => avanzar(-1));
  porId("btn-siguiente").addEventListener("click", () => avanzar(1));
  porId("btn-hoy").addEventListener("click", () => irA(hoy()));
  elementos.selectorFecha.addEventListener("change", () => irA(elementos.selectorFecha.value));
  elementos.vistaDia.addEventListener("click", () => cambiarVista("dia"));
  elementos.vistaSemana.addEventListener("click", () => cambiarVista("semana"));
  elementos.filtro.addEventListener("input", () => {
    estado.filtro = elementos.filtro.value;
    renderizar();
  });
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
