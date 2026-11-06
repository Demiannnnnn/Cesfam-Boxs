/**
 * Arranque y ciclo de refresco de la app.
 *
 * - Al iniciar: horario → conectar (guardada → localhost → broadcast) → grilla,
 *   o pantalla de conexión manual si no se encuentra el servidor (RF-09).
 * - Cada 20 s refresca boxes, etiquetas, funcionarios y reservas del día o la
 *   semana visible (RF-07, RF-13, RF-14); si falla, marca "Sin conexión" (RF-08) y
 *   en el siguiente ciclo intenta redescubrir el servidor.
 */
import { api } from "./api";
import { mostrarAsistente } from "./componentes/asistente";
import { mostrarAviso } from "./componentes/aviso";
import { abrirBuscar, type AccionesBuscar } from "./componentes/buscar";
import { renderizarConexion } from "./componentes/conexion";
import { alCerrarDialogos, hayDialogoAbierto } from "./componentes/dialogo";
import { abrirEtiquetas } from "./componentes/etiquetas";
import { abrirFuncionarios } from "./componentes/funcionarios";
import { renderizarGrilla, renderizarLeyendaEtiquetas } from "./componentes/grilla";
import { abrirBoxes, abrirConexionManual, abrirDetalle, abrirExportar } from "./componentes/modales";
import { abrirReservar, type AccionesReserva } from "./componentes/reservar";
import { renderizarSemana } from "./componentes/semana";
import { el, porId } from "./dom";
import { estado, etiquetasActivas, funcionariosActivos, puedeEscribir, type Vista } from "./estado";
import { diaHabil, diasHabilesDeSemana, fechaLarga, hoy, inicioSemana, sumarDias, tituloSemana } from "./fechas";
import { textos } from "./textos";
import type { ErrorUi, Reserva, ResultadoConexion } from "./tipos";

/** Intervalo de refresco automático (RF-07 pide entre 15 y 30 s). */
const INTERVALO_REFRESCO_MS = 20_000;

const elementos = {
  conexion: porId<HTMLDivElement>("conexion"),
  leyendaEtiquetas: porId<HTMLDivElement>("leyenda-etiquetas"),
  grilla: porId<HTMLElement>("grilla"),
  selectorFecha: porId<HTMLInputElement>("selector-fecha"),
  fechaLarga: porId<HTMLHeadingElement>("fecha-larga"),
  vistaDia: porId<HTMLButtonElement>("btn-vista-dia"),
  vistaSemana: porId<HTMLButtonElement>("btn-vista-semana"),
  filtro: porId<HTMLInputElement>("filtro-funcionario"),
  filtroEtiqueta: porId<HTMLSelectElement>("filtro-etiqueta"),
  listaFuncionarios: porId<HTMLDataListElement>("lista-funcionarios"),
  btnBoxes: porId<HTMLButtonElement>("btn-boxes"),
  btnFuncionarios: porId<HTMLButtonElement>("btn-funcionarios"),
  btnEtiquetas: porId<HTMLButtonElement>("btn-etiquetas"),
  indicadorSoloConsulta: porId<HTMLSpanElement>("indicador-solo-consulta"),
};

let temporizador: number | undefined;
let refrescando = false;

const accionesReserva: AccionesReserva = {
  alReservar: (resultado) => {
    const dias = new Set(resultado.creadas.map((r) => r.fecha)).size;
    if (resultado.omitidos.length > 0) mostrarAviso(textos.reserva.parcial(resultado.omitidos.length), "exito");
    else mostrarAviso(textos.reserva.listo(dias), "exito");
    void refrescar();
  },
  alConflicto: (error) => {
    mostrarAviso(error.mensaje);
    void refrescar();
  },
};

const accionesBuscar: AccionesBuscar = {
  alIrADia: (fecha) => cambiarVista("dia", fecha),
  alReservarBox: (box, fecha, desde, hasta) => {
    if (!puedeEscribir()) return;
    abrirReservar(box, fecha, desde, hasta, accionesReserva);
  },
};

function renderizar(): void {
  renderizarConexion(elementos.conexion, estado.conexion);
  renderizarLeyendaEtiquetas(elementos.leyendaEtiquetas);
  elementos.listaFuncionarios.replaceChildren(
    ...funcionariosActivos().map((f) => el("option", { atributos: { value: f.nombre } })),
  );
  const activas = etiquetasActivas();
  if (estado.filtroEtiquetaId !== null && !activas.some((e) => e.id === estado.filtroEtiquetaId)) {
    estado.filtroEtiquetaId = null;
  }
  elementos.filtroEtiqueta.replaceChildren(
    el("option", { texto: textos.navegacion.todasLasEtiquetas, atributos: { value: "" } }),
    ...activas.map((e) => el("option", { texto: e.nombre, atributos: { value: String(e.id) } })),
  );
  elementos.filtroEtiqueta.value =
    estado.filtroEtiquetaId !== null ? String(estado.filtroEtiquetaId) : "";
  elementos.selectorFecha.value = estado.fecha;
  elementos.vistaDia.setAttribute("aria-pressed", String(estado.vista === "dia"));
  elementos.vistaSemana.setAttribute("aria-pressed", String(estado.vista === "semana"));

  const escribir = puedeEscribir();
  elementos.btnBoxes.hidden = !escribir;
  elementos.btnFuncionarios.hidden = !escribir;
  elementos.btnEtiquetas.hidden = !escribir;
  elementos.indicadorSoloConsulta.hidden = estado.modo !== "lectura";
  elementos.indicadorSoloConsulta.textContent = textos.barra.soloConsulta;

  if (estado.vista === "semana") {
    elementos.fechaLarga.textContent = tituloSemana(estado.fecha);
    renderizarSemana(elementos.grilla, {
      alReservar: (box, fecha, desde) => {
        if (!puedeEscribir()) return;
        abrirReservar(box, fecha, desde, null, accionesReserva);
      },
      alVerDia: (fecha) => cambiarVista("dia", fecha),
    });
    return;
  }
  elementos.fechaLarga.textContent = fechaLarga(estado.fecha);
  renderizarGrilla(elementos.grilla, {
    alElegirLibre: (box, desde, hasta) => {
      if (!puedeEscribir()) return;
      abrirReservar(box, estado.fecha, desde, hasta, accionesReserva);
    },
    alElegirOcupada: (tramo) => abrirDetalle(tramo, () => void refrescar()),
  });
}

/** Reservas de lo que se ve: el día, o de lunes a viernes en la vista semanal. */
function pedirReservas(vista: Vista, fecha: string): Promise<Reserva[]> {
  if (vista === "dia") return api.listarReservas(fecha);
  const dias = diasHabilesDeSemana(fecha);
  return api.listarReservasRango(dias[0] ?? fecha, dias[dias.length - 1] ?? fecha);
}

/** Carga etiquetas y funcionarios registrados. No lanza si falla. */
async function cargarCatalogos(): Promise<void> {
  try {
    const [etiquetas, funcionarios] = await Promise.all([
      api.listarEtiquetas(true),
      api.listarFuncionarios(true),
    ]);
    estado.etiquetas = etiquetas;
    estado.funcionarios = funcionarios;
  } catch {
    // Si falla el catálogo, se conserva el último estado conocido.
  }
}

/** Pide boxes, catálogos y reservas de lo visible. Nunca lanza. */
async function refrescar(): Promise<void> {
  if (refrescando) return;
  refrescando = true;
  try {
    if (estado.conexion.tipo === "sin_conexion") {
      await reconectarEnSilencio();
    }
    const { vista, fecha } = estado;
    const [boxes, reservas] = await Promise.all([
      api.listarBoxes(false),
      pedirReservas(vista, fecha),
      cargarCatalogos(),
    ]);
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
  elementos.filtroEtiqueta.addEventListener("change", () => {
    estado.filtroEtiquetaId = elementos.filtroEtiqueta.value
      ? Number(elementos.filtroEtiqueta.value)
      : null;
    renderizar();
  });
  porId("btn-buscar").addEventListener("click", () => abrirBuscar(accionesBuscar));
  porId("btn-funcionarios").addEventListener("click", () => {
    if (!puedeEscribir()) return;
    abrirFuncionarios(() => void refrescar());
  });
  porId("btn-etiquetas").addEventListener("click", () => {
    if (!puedeEscribir()) return;
    abrirEtiquetas(() => void refrescar());
  });
  porId("btn-boxes").addEventListener("click", () => {
    if (!puedeEscribir()) return;
    abrirBoxes(() => void refrescar());
  });
  porId("btn-exportar").addEventListener("click", () => abrirExportar());
  document.addEventListener("keydown", (evento) => {
    if ((evento.ctrlKey || evento.metaKey) && evento.key.toLowerCase() === "f") {
      evento.preventDefault();
      if (!hayDialogoAbierto()) {
        abrirBuscar(accionesBuscar);
      }
    }
  });
  alCerrarDialogos(() => void refrescar());
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible" && !hayDialogoAbierto()) void refrescar();
  });
}

async function iniciar(): Promise<void> {
  enlazarEventos();
  renderizar();
  try {
    const [horario, preferencias, modoGuardado] = await Promise.all([
      api.horario(),
      api.preferencias(),
      api.modoEquipo(),
    ]);
    estado.horario = horario;
    estado.ultimoFuncionarioId = preferencias.ultimo_funcionario_id ?? null;

    let modo = modoGuardado;
    if (modo === null) {
      modo = await mostrarAsistente();
      if (modo === "servidor") {
        mostrarAviso(textos.asistente.servidorFaltaInstalar);
      }
    }
    estado.modo = modo;
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
