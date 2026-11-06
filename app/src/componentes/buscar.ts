/**
 * Modal de búsqueda con dos pestañas (RF-15):
 * 1. Reservas: por funcionario registrado, etiqueta y/o box en un rango de fechas,
 *    agrupando resultados en tramos y permitiendo saltar al día.
 * 2. Box libre: por fecha, rango horario (con atajos Mañana / Tarde / Día completo)
 *    y etiqueta opcional, con botón directo para reservar cada box libre.
 */
import { api } from "../api";
import { campo, el } from "../dom";
import {
  buscarFuncionarioActivoPorNombre,
  estado,
  etiquetaDeBox,
  etiquetaDeFuncionario,
  nombreBox,
  normalizarTexto,
  rangoHorario,
} from "../estado";
import { fechaCorta, hoy, sumarDias } from "../fechas";
import { textos } from "../textos";
import { listarTramos, type Tramo } from "../tramos";
import type { BoxAtencion, ErrorUi, Funcionario } from "../tipos";
import { abrirDialogo, type Dialogo } from "./dialogo";
import {
  chipEtiqueta,
  crearSelectorEtiqueta,
  crearSelectorRangoHorario,
  mientrasOcupado,
} from "./formulario";

const DIAS_CUATRO_SEMANAS = 28;

export interface AccionesBuscar {
  alIrADia: (fecha: string) => void;
  alReservarBox: (box: BoxAtencion, fecha: string, desde: number, hasta: number) => void;
}

function buscarFuncionarioPorNombre(nombre: string): Funcionario | null {
  const activo = buscarFuncionarioActivoPorNombre(nombre);
  if (activo) return activo;
  const buscado = normalizarTexto(nombre);
  if (!buscado) return null;
  return estado.funcionarios.find((f) => normalizarTexto(f.nombre) === buscado) ?? null;
}

export function abrirBuscar(acciones: AccionesBuscar): void {
  const dialogo = abrirDialogo(textos.buscar.titulo);

  const btnReservas = el("button", {
    clase: "boton boton--vista",
    texto: textos.buscar.pestanaReservas,
    atributos: { type: "button", "aria-pressed": "true" },
  });
  const btnBoxLibre = el("button", {
    clase: "boton boton--vista",
    texto: textos.buscar.pestanaBoxLibre,
    atributos: { type: "button", "aria-pressed": "false" },
  });
  const pestanas = el(
    "div",
    {
      clase: "vistas buscar__pestanas",
      atributos: { role: "group", "aria-label": textos.buscar.pestanasGrupo },
    },
    btnReservas,
    btnBoxLibre,
  );

  const seccionReservas = construirPestanaReservas(dialogo, acciones);
  const seccionBoxLibre = construirPestanaBoxLibre(dialogo, acciones);
  seccionBoxLibre.raiz.hidden = true;

  let cargoBoxLibreInicial = false;

  const activar = (cual: "reservas" | "box_libre") => {
    dialogo.mostrarError(null);
    const esReservas = cual === "reservas";
    btnReservas.setAttribute("aria-pressed", String(esReservas));
    btnBoxLibre.setAttribute("aria-pressed", String(!esReservas));
    seccionReservas.raiz.hidden = !esReservas;
    seccionBoxLibre.raiz.hidden = esReservas;
    if (!esReservas && !cargoBoxLibreInicial) {
      cargoBoxLibreInicial = true;
      void seccionBoxLibre.ejecutarBusqueda();
    }
  };

  btnReservas.addEventListener("click", () => activar("reservas"));
  btnBoxLibre.addEventListener("click", () => activar("box_libre"));

  dialogo.cuerpo.append(pestanas, seccionReservas.raiz, seccionBoxLibre.raiz);
  seccionReservas.enfocar();
}

function construirPestanaReservas(
  dialogo: Dialogo,
  acciones: AccionesBuscar,
): { raiz: HTMLElement; enfocar: () => void } {
  const inputFuncionario = el("input", {
    atributos: {
      type: "search",
      maxlength: "80",
      autocomplete: "off",
      list: "lista-funcionarios",
      placeholder: textos.buscar.funcionarioPlaceholder,
    },
  });
  const estadoFuncionario = el("div", { clase: "reserva-funcionario__estado" });

  const actualizarChipFuncionario = () => {
    const coincidencia = buscarFuncionarioPorNombre(inputFuncionario.value);
    const etiq = coincidencia ? etiquetaDeFuncionario(coincidencia.id) : null;
    estadoFuncionario.replaceChildren(...(etiq ? [chipEtiqueta(etiq)] : []));
  };
  inputFuncionario.addEventListener("input", () => {
    dialogo.mostrarError(null);
    actualizarChipFuncionario();
  });

  const selectorEtiqueta = crearSelectorEtiqueta(
    estado.etiquetas,
    null,
    textos.buscar.todasLasEtiquetas,
    textos.buscar.etiqueta,
  );

  const selectBox = el("select", { atributos: { "aria-label": textos.buscar.box } });
  selectBox.append(el("option", { texto: textos.buscar.todosLosBoxes, atributos: { value: "" } }));
  for (const box of estado.boxes) {
    selectBox.append(el("option", { texto: box.nombre, atributos: { value: String(box.id) } }));
  }

  const desde = el("input", { atributos: { type: "date", required: "" } });
  const hasta = el("input", { atributos: { type: "date", required: "" } });
  const fechaHoy = hoy();
  desde.value = fechaHoy;
  hasta.value = sumarDias(fechaHoy, DIAS_CUATRO_SEMANAS);

  const btnBuscar = el("button", {
    clase: "boton boton--primario",
    texto: textos.buscar.accionBuscar,
    atributos: { type: "submit" },
  });

  const contador = el("p", { clase: "buscar__contador", atributos: { "aria-live": "polite" } });
  contador.hidden = true;
  const contenedorResultados = el("div", { clase: "buscar__resultados" });

  const formulario = el(
    "form",
    { clase: "formulario buscar__formulario" },
    campo(textos.buscar.funcionario, inputFuncionario, estadoFuncionario),
    el(
      "div",
      { clase: "formulario--fila buscar__fila" },
      campo(textos.buscar.etiqueta, selectorEtiqueta.raiz),
      campo(textos.buscar.box, selectBox),
    ),
    el(
      "div",
      { clase: "formulario--fila buscar__fila buscar__fila--fechas" },
      campo(textos.buscar.desde, desde),
      campo(textos.buscar.hasta, hasta),
      btnBuscar,
    ),
  );

  formulario.addEventListener("submit", async (evento) => {
    evento.preventDefault();
    dialogo.mostrarError(null);

    const textoFunc = inputFuncionario.value.trim();
    let funcionarioId: number | null = null;
    if (textoFunc) {
      const encontrado = buscarFuncionarioPorNombre(textoFunc);
      if (!encontrado) {
        dialogo.mostrarError(textos.reserva.funcionarioNoRegistrado);
        inputFuncionario.focus();
        return;
      }
      funcionarioId = encontrado.id;
    }

    const etiquetaId = selectorEtiqueta.valor();
    const boxId = selectBox.value ? Number(selectBox.value) : null;

    if (funcionarioId === null && etiquetaId === null && boxId === null) {
      dialogo.mostrarError(textos.buscar.sinFiltros);
      return;
    }
    if (desde.value > hasta.value) {
      dialogo.mostrarError(textos.buscar.fechasInvertidas);
      return;
    }

    try {
      const reservas = await mientrasOcupado(btnBuscar, textos.buscar.buscando, () =>
        api.buscarReservas({
          desde: desde.value,
          hasta: hasta.value,
          funcionario_id: funcionarioId,
          etiqueta_id: etiquetaId,
          box_id: boxId,
        }),
      );
      const tramos = listarTramos(reservas);
      contador.textContent = textos.buscar.contadorResultados(tramos.length);
      contador.hidden = false;
      if (tramos.length === 0) {
        contenedorResultados.replaceChildren(
          el("p", { clase: "buscar__vacio", texto: textos.buscar.sinResultados }),
        );
        return;
      }
      contenedorResultados.replaceChildren(
        ...tramos.map((t) => itemResultadoReserva(t, dialogo, acciones)),
      );
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  });

  return {
    raiz: el("div", { clase: "buscar__panel" }, formulario, contador, contenedorResultados),
    enfocar: () => inputFuncionario.focus(),
  };
}

function itemResultadoReserva(
  tramo: Tramo,
  dialogo: Dialogo,
  acciones: AccionesBuscar,
): HTMLButtonElement {
  const fecha = tramo.reservas[0]?.fecha ?? estado.fecha;
  const fechaTexto = fechaCorta(fecha);
  const horas = rangoHorario(tramo.desde, tramo.hasta);
  const boxTexto = nombreBox(tramo.box_id);
  const etiqueta = etiquetaDeFuncionario(tramo.funcionario_id);

  return el(
    "button",
    {
      clase: "buscar__item buscar__item--reserva",
      atributos: {
        type: "button",
        title: textos.buscar.irAlDia(fechaTexto),
      },
      alHacerClic: () => {
        dialogo.cerrar();
        acciones.alIrADia(fecha);
      },
    },
    el("span", { clase: "buscar__fecha", texto: fechaTexto }),
    el("span", { clase: "buscar__horario", texto: horas }),
    el("span", { clase: "buscar__box", texto: boxTexto }),
    el(
      "span",
      { clase: "buscar__funcionario" },
      el("span", { clase: "buscar__funcionario-nombre", texto: tramo.funcionario }),
      etiqueta ? chipEtiqueta(etiqueta) : null,
      tramo.motivo ? el("span", { clase: "buscar__motivo", texto: tramo.motivo }) : null,
    ),
  );
}

function construirPestanaBoxLibre(
  dialogo: Dialogo,
  acciones: AccionesBuscar,
): { raiz: HTMLElement; ejecutarBusqueda: () => Promise<void> } {
  const horario = estado.horario;
  const inputFecha = el("input", { atributos: { type: "date", required: "" } });
  inputFecha.value = estado.fecha;

  const selectorHorario = horario
    ? crearSelectorRangoHorario(horario, 0, horario.bloque_inicio_tarde - 1)
    : null;

  const selectorEtiqueta = crearSelectorEtiqueta(
    estado.etiquetas,
    null,
    textos.buscar.todasLasEtiquetas,
    textos.buscar.etiqueta,
  );

  const btnBuscar = el("button", {
    clase: "boton boton--primario",
    texto: textos.buscar.accionBuscar,
    atributos: { type: "submit" },
  });

  const contador = el("p", { clase: "buscar__contador", atributos: { "aria-live": "polite" } });
  contador.hidden = true;
  const contenedorResultados = el("div", { clase: "buscar__resultados" });

  const ejecutarBusqueda = async () => {
    if (!selectorHorario) return;
    dialogo.mostrarError(null);
    const [desde, hasta] = selectorHorario.rango();
    if (desde > hasta) {
      dialogo.mostrarError(textos.reserva.rangoInvertido);
      return;
    }
    const fechaConsultada = inputFecha.value;
    const etiquetaId = selectorEtiqueta.valor();

    try {
      const libres = await mientrasOcupado(btnBuscar, textos.buscar.buscando, () =>
        api.boxesLibres({
          fecha: fechaConsultada,
          desde,
          hasta,
          etiqueta_id: etiquetaId,
        }),
      );
      contador.textContent = textos.buscar.contadorBoxesLibres(libres.length);
      contador.hidden = false;
      if (libres.length === 0) {
        contenedorResultados.replaceChildren(
          el("p", { clase: "buscar__vacio", texto: textos.buscar.sinResultados }),
        );
        return;
      }
      contenedorResultados.replaceChildren(
        ...libres.map((box) =>
          filaBoxLibre(box, fechaConsultada, desde, hasta, dialogo, acciones),
        ),
      );
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  };

  const formulario = el(
    "form",
    { clase: "formulario buscar__formulario" },
    el(
      "div",
      { clase: "formulario--fila buscar__fila" },
      campo(textos.buscar.fecha, inputFecha),
      campo(textos.buscar.etiqueta, selectorEtiqueta.raiz),
    ),
    selectorHorario
      ? el(
          "div",
          { clase: "campo" },
          el("span", { clase: "campo__etiqueta", texto: textos.buscar.horario }),
          selectorHorario.raiz,
        )
      : null,
    el("div", { clase: "buscar__acciones" }, btnBuscar),
  );

  formulario.addEventListener("submit", (evento) => {
    evento.preventDefault();
    void ejecutarBusqueda();
  });

  return {
    raiz: el("div", { clase: "buscar__panel" }, formulario, contador, contenedorResultados),
    ejecutarBusqueda,
  };
}

function filaBoxLibre(
  box: BoxAtencion,
  fecha: string,
  desde: number,
  hasta: number,
  dialogo: Dialogo,
  acciones: AccionesBuscar,
): HTMLElement {
  const etiqueta = etiquetaDeBox(box);
  const btnReservar = el("button", {
    clase: "boton boton--primario boton--chico",
    texto: textos.buscar.reservar,
    atributos: { type: "button" },
    alHacerClic: () => {
      dialogo.cerrar();
      acciones.alReservarBox(box, fecha, desde, hasta);
    },
  });

  return el(
    "div",
    { clase: "buscar__item buscar__item--box" },
    el(
      "div",
      { clase: "buscar__box-info" },
      el("span", { clase: "buscar__box-nombre", texto: box.nombre }),
      etiqueta ? chipEtiqueta(etiqueta) : null,
    ),
    btnReservar,
  );
}
