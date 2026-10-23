/**
 * Grilla del día (RF-01): boxes activos en filas y bloques en columnas, separados
 * en mañana y tarde como la planilla del CESFAM. Los bloques seguidos de una misma
 * reserva se dibujan como una sola barra. Se reconstruye completa desde el estado
 * en cada render.
 */
import { el } from "../dom";
import { bloquesTerminados, hoy } from "../fechas";
import { estado, etiquetaBloque, rangoHorario } from "../estado";
import { textos } from "../textos";
import { agruparEnTramos, type Tramo } from "../tramos";
import type { BoxAtencion, Horario } from "../tipos";

export interface AccionesGrilla {
  alElegirLibre: (box: BoxAtencion, bloque: number) => void;
  alElegirOcupada: (tramo: Tramo) => void;
}

/** Contexto de pintado de una fila, calculado una vez por render. */
interface Contexto {
  inicioTarde: number;
  terminados: number;
  enCurso: number | null;
  deshabilitada: boolean;
  acciones: AccionesGrilla;
}

export function renderizarGrilla(contenedor: HTMLElement, acciones: AccionesGrilla): void {
  const { horario, boxes, reservas, fecha, conexion } = estado;
  contenedor.classList.toggle("grilla--sin-conexion", conexion.tipo !== "conectado");

  if (!horario) {
    contenedor.replaceChildren(el("p", { clase: "grilla__mensaje", texto: textos.grilla.cargando }));
    return;
  }
  if (boxes.length === 0) {
    contenedor.replaceChildren(el("p", { clase: "grilla__mensaje", texto: textos.grilla.sinBoxes }));
    return;
  }

  // Para hoy: bloques ya terminados se atenúan y el bloque en curso se resalta.
  const terminados =
    fecha === hoy()
      ? bloquesTerminados(horario.hora_inicio, horario.minutos_por_bloque, horario.cantidad_bloques)
      : 0;
  const contexto: Contexto = {
    inicioTarde: horario.bloque_inicio_tarde,
    terminados,
    enCurso: fecha === hoy() && dentroDelHorario(horario.hora_inicio) ? terminados : null,
    deshabilitada: conexion.tipo !== "conectado",
    acciones,
  };
  const tramos = agruparEnTramos(reservas);

  const tabla = el(
    "table",
    { clase: "grilla__tabla" },
    columnas(horario),
    encabezado(horario, contexto),
    el("tbody", {}, ...boxes.map((box) => fila(box, horario, tramos, contexto))),
  );
  contenedor.replaceChildren(tabla);
}

function dentroDelHorario(horaInicio: number): boolean {
  return new Date().getHours() >= horaInicio;
}

/** Anchos fijos: columna de boxes ancha, bloques iguales. */
function columnas(horario: Horario): HTMLTableColElement {
  const grupo = el("colgroup", {}, el("col", { clase: "grilla__col-box" }));
  for (let bloque = 0; bloque < horario.cantidad_bloques; bloque++) grupo.append(el("col"));
  return grupo;
}

/** Dos filas: "Mañana | Tarde" y una etiqueta por hora (cada hora abarca sus bloques). */
function encabezado(horario: Horario, contexto: Contexto): HTMLTableSectionElement {
  const { cantidad_bloques: total, minutos_por_bloque: minutos } = horario;
  const tarde = contexto.inicioTarde;
  const periodos = el(
    "tr",
    {},
    el("th", {
      clase: "grilla__esquina",
      texto: textos.grilla.columnaBox,
      atributos: { scope: "col", rowspan: "2" },
    }),
    el("th", { clase: "grilla__periodo", texto: textos.grilla.manana, atributos: { colspan: String(tarde) } }),
    el("th", {
      clase: "grilla__periodo grilla__inicio-tarde",
      texto: textos.grilla.tarde,
      atributos: { colspan: String(total - tarde) },
    }),
  );

  const porHora = 60 % minutos === 0 ? 60 / minutos : 1;
  const horas = el("tr", {});
  for (let bloque = 0; bloque < total; bloque += porHora) {
    const abarca = Math.min(porHora, total - bloque);
    const clases = ["grilla__hora"];
    if (bloque === tarde) clases.push("grilla__inicio-tarde");
    const enCurso = contexto.enCurso;
    if (enCurso !== null && enCurso >= bloque && enCurso < bloque + abarca) clases.push("grilla__hora--actual");
    horas.append(
      el("th", {
        clase: clases.join(" "),
        texto: etiquetaBloque(bloque),
        atributos: { scope: "colgroup", colspan: String(abarca) },
      }),
    );
  }
  return el("thead", {}, periodos, horas);
}

function fila(box: BoxAtencion, horario: Horario, tramos: Map<string, Tramo>, contexto: Contexto): HTMLTableRowElement {
  const celdas: HTMLTableCellElement[] = [];
  let bloque = 0;
  while (bloque < horario.cantidad_bloques) {
    const tramo = tramos.get(`${box.id}-${bloque}`);
    if (tramo) {
      celdas.push(celdaOcupada(tramo, contexto));
      bloque = tramo.hasta + 1;
    } else {
      celdas.push(celdaLibre(box, bloque, contexto));
      bloque += 1;
    }
  }
  return el(
    "tr",
    { clase: "grilla__fila" },
    el("th", { clase: "grilla__box", texto: box.nombre, atributos: { scope: "row", title: box.nombre } }),
    ...celdas,
  );
}

function td(desde: number, abarca: number, contexto: Contexto, clasesExtra: string[] = []): HTMLTableCellElement {
  const clases = [...clasesExtra];
  if (desde === contexto.inicioTarde) clases.push("grilla__inicio-tarde");
  const celda = el("td", { clase: clases.join(" ") });
  if (abarca > 1) celda.colSpan = abarca;
  return celda;
}

function celdaOcupada(tramo: Tramo, contexto: Contexto): HTMLTableCellElement {
  const horas = rangoHorario(tramo.desde, tramo.hasta);
  const detalle = [tramo.funcionario, horas, tramo.motivo].filter(Boolean).join(" · ");
  const boton = el(
    "button",
    {
      clase: "celda celda--ocupada",
      atributos: { type: "button", title: detalle, "aria-label": detalle },
      alHacerClic: () => contexto.acciones.alElegirOcupada(tramo),
    },
    el("span", { clase: "celda__nombre", texto: tramo.funcionario }),
    tramo.motivo ? el("span", { clase: "celda__motivo", texto: tramo.motivo }) : null,
  );
  boton.disabled = contexto.deshabilitada;
  const celda = td(tramo.desde, tramo.hasta - tramo.desde + 1, contexto);
  celda.append(boton);
  return celda;
}

function celdaLibre(box: BoxAtencion, bloque: number, contexto: Contexto): HTMLTableCellElement {
  const clases = ["celda", "celda--libre"];
  if (bloque < contexto.terminados) clases.push("celda--pasada");
  if (bloque === contexto.enCurso) clases.push("celda--actual");
  const boton = el("button", {
    clase: clases.join(" "),
    atributos: {
      type: "button",
      title: textos.grilla.reservarEn(box.nombre, etiquetaBloque(bloque)),
      "aria-label": textos.grilla.reservarEn(box.nombre, etiquetaBloque(bloque)),
      "data-texto": "+",
    },
    alHacerClic: () => contexto.acciones.alElegirLibre(box, bloque),
  });
  boton.disabled = contexto.deshabilitada;
  const celda = td(bloque, 1, contexto);
  celda.append(boton);
  return celda;
}
