/**
 * Modal de reserva (RF-03, RF-05, RF-13, RF-14): un rango de bloques del día, con
 * atajos Mañana / Tarde / Día completo, y opcionalmente repetido cada semana.
 * Se elige un funcionario registrado y activo (enviando `funcionario_id`); si el
 * nombre escrito no está registrado, ofrece registrarlo sin salir del modal.
 */
import { api } from "../api";
import { campo, el } from "../dom";
import {
  buscarFuncionarioActivoPorNombre,
  estado,
  etiquetaBloque,
  etiquetaDeFuncionario,
  etiquetaFinBloque,
  funcionariosActivos,
} from "../estado";
import { fechaCorta, finDeMes, sumarDias } from "../fechas";
import { textos } from "../textos";
import type { BoxAtencion, ErrorUi, Horario, ResultadoLote } from "../tipos";
import { abrirDialogo } from "./dialogo";
import { chipEtiqueta, mientrasOcupado, soloLectura } from "./formulario";

/** Igual que `MAX_FECHAS_LOTE` en la API. */
const MAX_FECHAS = 46;
const DIAS_POR_SEMANA = 7;

export interface AccionesReserva {
  alReservar: (resultado: ResultadoLote) => void;
  /** Error que obliga a refrescar la grilla (409 de un solo bloque, box inactivo). */
  alConflicto: (error: ErrorUi) => void;
}

/**
 * Hasta dónde llega por defecto una reserva que empieza en `desde`: el fin de su
 * medio día (como "BOX AM" / "BOX PM"), sin pisar la siguiente reserva del box.
 */
export function hastaPorDefecto(box: BoxAtencion, fecha: string, desde: number, horario: Horario): number {
  const finPeriodo = desde < horario.bloque_inicio_tarde ? horario.bloque_inicio_tarde - 1 : horario.cantidad_bloques - 1;
  const ocupados = new Set(
    estado.reservas.filter((r) => r.box_id === box.id && r.fecha === fecha).map((r) => r.bloque),
  );
  let hasta = desde;
  while (hasta < finPeriodo && !ocupados.has(hasta + 1)) hasta += 1;
  return hasta;
}

/** Fechas de la serie: `inicio` y cada 7 días hasta `fin` inclusive. */
export function fechasSemanales(inicio: string, fin: string): string[] {
  const fechas: string[] = [];
  for (let fecha = inicio; fecha <= fin && fechas.length <= MAX_FECHAS; fecha = sumarDias(fecha, DIAS_POR_SEMANA)) {
    fechas.push(fecha);
  }
  return fechas;
}

function selector(opciones: Array<[number, string]>, valor: number, etiqueta: string): HTMLSelectElement {
  const select = el("select", { atributos: { "aria-label": etiqueta } });
  for (const [v, texto] of opciones) select.append(el("option", { texto, atributos: { value: String(v) } }));
  select.value = String(valor);
  return select;
}

export function abrirReservar(
  box: BoxAtencion,
  fecha: string,
  desde: number,
  hasta: number | null,
  acciones: AccionesReserva,
): void {
  const horario = estado.horario;
  if (!horario) return;
  const dialogo = abrirDialogo(textos.reserva.titulo);
  const total = horario.cantidad_bloques;
  const tarde = horario.bloque_inicio_tarde;
  const indices = Array.from({ length: total }, (_, i) => i);

  // ── Horario ──
  const selDesde = selector(indices.map((i) => [i, etiquetaBloque(i)]), desde, textos.reserva.desde);
  const selHasta = selector(
    indices.map((i) => [i, etiquetaFinBloque(i)]),
    hasta ?? hastaPorDefecto(box, fecha, desde, horario),
    textos.reserva.hasta,
  );
  const fijar = (a: number, b: number) => {
    selDesde.value = String(a);
    selHasta.value = String(b);
    actualizarResumen();
  };
  const atajos = el(
    "div",
    { clase: "atajos" },
    el("button", { clase: "boton boton--chico", texto: textos.reserva.manana, atributos: { type: "button" }, alHacerClic: () => fijar(0, tarde - 1) }),
    el("button", { clase: "boton boton--chico", texto: textos.reserva.tarde, atributos: { type: "button" }, alHacerClic: () => fijar(tarde, total - 1) }),
    el("button", { clase: "boton boton--chico", texto: textos.reserva.diaCompleto, atributos: { type: "button" }, alHacerClic: () => fijar(0, total - 1) }),
  );

  // ── Funcionario registrado (RF-13, RF-14) ──
  const inputFuncionario = el("input", {
    atributos: { type: "text", maxlength: "80", required: "", autocomplete: "off", list: "lista-funcionarios" },
  });
  const ultimoActivo =
    estado.ultimoFuncionarioId !== null
      ? funcionariosActivos().find((f) => f.id === estado.ultimoFuncionarioId) ?? null
      : null;
  inputFuncionario.value = ultimoActivo?.nombre ?? "";

  const estadoFuncionario = el("div", { clase: "reserva-funcionario__estado" });

  const registrarEnLinea = async (nombre: string, btnRegistrar: HTMLButtonElement) => {
    dialogo.mostrarError(null);
    try {
      const creado = await mientrasOcupado(btnRegistrar, textos.reserva.registrandoFuncionario, () =>
        api.crearFuncionario(nombre, null),
      );
      estado.funcionarios = [...estado.funcionarios.filter((f) => f.id !== creado.id), creado].sort((a, b) =>
        a.nombre.localeCompare(b.nombre, "es"),
      );
      inputFuncionario.value = creado.nombre;
      actualizarEstadoFuncionario();
      motivo.focus();
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  };

  function actualizarEstadoFuncionario(): void {
    const escrito = inputFuncionario.value.trim();
    if (!escrito) {
      estadoFuncionario.replaceChildren();
      return;
    }
    const coincidencia = buscarFuncionarioActivoPorNombre(escrito);
    if (coincidencia) {
      const etiq = etiquetaDeFuncionario(coincidencia.id);
      estadoFuncionario.replaceChildren(...(etiq ? [chipEtiqueta(etiq)] : []));
      return;
    }
    const btnRegistrar = el("button", {
      clase: "boton boton--chico boton--secundario",
      texto: textos.reserva.registrarFuncionario(escrito),
      atributos: { type: "button" },
      alHacerClic: () => void registrarEnLinea(escrito, btnRegistrar),
    });
    estadoFuncionario.replaceChildren(btnRegistrar);
  }

  inputFuncionario.addEventListener("input", () => {
    dialogo.mostrarError(null);
    actualizarEstadoFuncionario();
  });
  actualizarEstadoFuncionario();

  const motivo = el("input", { atributos: { type: "text", maxlength: "200", autocomplete: "off" } });

  // ── Repetición semanal ──
  const repetir = el("input", { atributos: { type: "checkbox" } });
  const repetirHasta = el("input", { atributos: { type: "date", min: fecha } });
  repetirHasta.value = finDeMes(fecha);
  repetirHasta.disabled = true;
  const resumen = el("p", { clase: "ayuda resumen-reserva", atributos: { "aria-live": "polite" } });

  const leerRango = (): [number, number] => [Number(selDesde.value), Number(selHasta.value)];
  const leerFechas = (): string[] => (repetir.checked ? fechasSemanales(fecha, repetirHasta.value || fecha) : [fecha]);

  function actualizarResumen(): void {
    repetirHasta.disabled = !repetir.checked;
    const [a, b] = leerRango();
    const fechas = leerFechas();
    if (a > b) {
      resumen.textContent = textos.reserva.rangoInvertido;
      return;
    }
    const horas = `${etiquetaBloque(a)} – ${etiquetaFinBloque(b)}`;
    resumen.textContent =
      fechas.length > 1 ? textos.reserva.resumenSerie(horas, fechas.map(fechaCorta)) : textos.reserva.resumenDia(horas);
  }
  for (const control of [selDesde, selHasta, repetir, repetirHasta]) control.addEventListener("change", actualizarResumen);
  actualizarResumen();

  const enviar = el("button", {
    clase: "boton boton--primario",
    texto: textos.reserva.confirmar,
    atributos: { type: "submit", form: "formulario-reserva" },
  });
  const soloLibres = el("button", {
    clase: "boton boton--secundario",
    texto: textos.reserva.soloLibres,
    atributos: { type: "button" },
  });
  soloLibres.hidden = true;

  const formulario = el(
    "form",
    { clase: "formulario" },
    el("div", { clase: "datos" }, soloLectura(textos.reserva.box, box.nombre), soloLectura(textos.reserva.fecha, fechaCorta(fecha))),
    el(
      "div",
      { clase: "campo" },
      el("span", { clase: "campo__etiqueta", texto: textos.reserva.horario }),
      el("div", { clase: "rango" }, selDesde, el("span", { texto: "–" }), selHasta, atajos),
    ),
    campo(textos.reserva.funcionario, inputFuncionario, estadoFuncionario),
    campo(textos.reserva.motivo, motivo),
    el(
      "div",
      { clase: "repetir" },
      el("label", { clase: "casilla" }, repetir, el("span", { texto: textos.reserva.repetir })),
      el("label", { clase: "repetir__hasta" }, el("span", { texto: textos.reserva.repetirHasta }), repetirHasta),
    ),
    resumen,
  );
  formulario.id = "formulario-reserva";

  async function reservar(omitirOcupados: boolean, botonAccion: HTMLButtonElement): Promise<void> {
    const nombreEscrito = inputFuncionario.value.trim();
    if (!nombreEscrito) {
      dialogo.mostrarError(textos.reserva.funcionarioObligatorio);
      inputFuncionario.focus();
      return;
    }
    const elegido = buscarFuncionarioActivoPorNombre(nombreEscrito);
    if (!elegido) {
      actualizarEstadoFuncionario();
      dialogo.mostrarError(textos.reserva.funcionarioNoRegistrado);
      inputFuncionario.focus();
      return;
    }
    const [a, b] = leerRango();
    const fechas = leerFechas();
    if (a > b) {
      dialogo.mostrarError(textos.reserva.rangoInvertido);
      return;
    }
    if (fechas.length > MAX_FECHAS) {
      dialogo.mostrarError(textos.reserva.demasiadasFechas(MAX_FECHAS));
      return;
    }
    dialogo.mostrarError(null);
    try {
      const resultado = await mientrasOcupado(botonAccion, textos.reserva.enviando, () =>
        api.crearReservas({
          box_id: box.id,
          fechas,
          desde: a,
          hasta: b,
          funcionario_id: elegido.id,
          motivo: motivo.value.trim() || null,
          omitir_ocupados: omitirOcupados,
        }),
      );
      estado.ultimoFuncionarioId = elegido.id;
      dialogo.cerrar();
      acciones.alReservar(resultado);
    } catch (error) {
      const e = error as ErrorUi;
      const unSoloBloque = a === b && fechas.length === 1;
      if (e.codigo === "box_inactivo" || (e.codigo === "bloque_ocupado" && (unSoloBloque || omitirOcupados))) {
        dialogo.cerrar();
        acciones.alConflicto(e);
      } else if (e.codigo === "bloque_ocupado") {
        dialogo.mostrarError(textos.reserva.algunosOcupados);
        soloLibres.hidden = false;
      } else {
        dialogo.mostrarError(e.mensaje);
      }
    }
  }

  formulario.addEventListener("submit", (evento) => {
    evento.preventDefault();
    void reservar(false, enviar);
  });
  soloLibres.addEventListener("click", () => void reservar(true, soloLibres));

  dialogo.cuerpo.append(formulario);
  dialogo.pie.prepend(enviar, soloLibres);
  (inputFuncionario.value ? motivo : inputFuncionario).focus();
}
