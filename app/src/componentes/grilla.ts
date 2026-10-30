/**
 * Grilla del día (RF-01, RF-14): boxes activos en filas y bloques en columnas,
 * separados en mañana y tarde como la planilla del CESFAM. Los bloques seguidos
 * de una misma reserva se dibujan como una sola barra con el color de la etiqueta
 * del funcionario.
 */
import { colorTexto } from "../colores";
import { el } from "../dom";
import {
  coincideFiltro,
  estado,
  etiquetaBloque,
  etiquetaDeBox,
  etiquetaDeFuncionario,
  etiquetasActivas,
  rangoHorario,
} from "../estado";
import { bloquesTerminados, hoy } from "../fechas";
import { textos } from "../textos";
import { agruparEnTramos, type Tramo } from "../tramos";
import type { BoxAtencion, Horario } from "../tipos";
import { chipEtiqueta } from "./formulario";

export interface AccionesGrilla {
  /** `hasta` es `null` en un clic simple (el modal propone el rango). */
  alElegirLibre: (box: BoxAtencion, desde: number, hasta: number | null) => void;
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

/** Leyenda compacta de etiquetas activas sobre la grilla (RF-14). */
export function renderizarLeyendaEtiquetas(contenedor: HTMLElement): void {
  const activas = etiquetasActivas();
  if (activas.length === 0) {
    contenedor.hidden = true;
    contenedor.replaceChildren();
    return;
  }
  contenedor.hidden = false;
  contenedor.replaceChildren(...activas.map((etiq) => chipEtiqueta(etiq, "chip-etiqueta--leyenda")));
}

/** Encabezado `<th>` de la fila de un box, con franja y chip del color de su etiqueta (RF-14). */
export function encabezadoBox(box: BoxAtencion): HTMLTableCellElement {
  const etiqueta = etiquetaDeBox(box);
  const titulo = etiqueta ? textos.grilla.boxConEtiqueta(box.nombre, etiqueta.nombre) : box.nombre;
  const th = el(
    "th",
    { clase: "grilla__box", atributos: { scope: "row", title: titulo } },
    el("span", { clase: "grilla__box-nombre", texto: box.nombre }),
    etiqueta ? chipEtiqueta(etiqueta, "chip-etiqueta--box") : null,
  );
  if (etiqueta) {
    th.style.borderLeft = `5px solid ${etiqueta.color}`;
  }
  return th;
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
  return el("tr", { clase: "grilla__fila" }, encabezadoBox(box), ...celdas);
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
  const etiqueta = etiquetaDeFuncionario(tramo.funcionario_id);
  const detalle = [tramo.funcionario, etiqueta?.nombre, horas, tramo.motivo].filter(Boolean).join(" · ");
  const coincide = coincideFiltro({ funcionario_id: tramo.funcionario_id, nombre: tramo.funcionario });
  const clases = ["celda", "celda--ocupada"];
  if (coincide === true) clases.push("celda--destacada");
  if (coincide === false) clases.push("celda--atenuada");
  const boton = el(
    "button",
    {
      clase: clases.join(" "),
      atributos: { type: "button", title: detalle, "aria-label": detalle },
      alHacerClic: () => contexto.acciones.alElegirOcupada(tramo),
    },
    el("span", { clase: "celda__nombre", texto: tramo.funcionario }),
    tramo.motivo ? el("span", { clase: "celda__motivo", texto: tramo.motivo }) : null,
  );
  if (etiqueta) {
    boton.style.backgroundColor = etiqueta.color;
    boton.style.borderColor = etiqueta.color;
    boton.style.color = colorTexto(etiqueta.color);
  }
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
    alHacerClic: () => contexto.acciones.alElegirLibre(box, bloque, null),
  });
  boton.dataset["bloque"] = String(bloque);
  boton.addEventListener("pointerdown", (evento) => {
    if (evento.button === 0 && !boton.disabled) iniciarArrastre(box, bloque, boton, contexto.acciones);
  });
  boton.addEventListener("pointerenter", () => extenderArrastre(box, bloque));
  boton.disabled = contexto.deshabilitada;
  const celda = td(bloque, 1, contexto);
  celda.append(boton);
  return celda;
}

// ─── Arrastre para elegir un rango ─────────────────────────────────────────

interface Arrastre {
  box: BoxAtencion;
  inicio: number;
  fin: number;
  fila: HTMLElement;
  acciones: AccionesGrilla;
}

let arrastre: Arrastre | null = null;

function iniciarArrastre(box: BoxAtencion, bloque: number, boton: HTMLElement, acciones: AccionesGrilla): void {
  const fila = boton.closest("tr");
  if (!fila) return;
  arrastre = { box, inicio: bloque, fin: bloque, fila, acciones };
  document.addEventListener("pointerup", terminarArrastre, { once: true });
}

/** Solo se extiende sobre celdas libres contiguas del mismo box. */
function extenderArrastre(box: BoxAtencion, bloque: number): void {
  if (!arrastre || arrastre.box.id !== box.id) return;
  const [a, b] = [Math.min(arrastre.inicio, bloque), Math.max(arrastre.inicio, bloque)];
  const libres = new Set(
    [...arrastre.fila.querySelectorAll<HTMLElement>(".celda--libre")].map((c) => Number(c.dataset["bloque"])),
  );
  for (let i = a; i <= b; i++) if (!libres.has(i)) return;
  arrastre.fin = bloque;
  pintarSeleccion(arrastre.fila, a, b);
}

function pintarSeleccion(fila: HTMLElement, a: number, b: number): void {
  for (const celda of fila.querySelectorAll<HTMLElement>(".celda--libre")) {
    const bloque = Number(celda.dataset["bloque"]);
    celda.classList.toggle("celda--seleccionada", bloque >= a && bloque <= b);
  }
}

function terminarArrastre(): void {
  const actual = arrastre;
  arrastre = null;
  if (!actual || actual.inicio === actual.fin) return;
  pintarSeleccion(actual.fila, -1, -1);
  const [a, b] = [Math.min(actual.inicio, actual.fin), Math.max(actual.inicio, actual.fin)];
  actual.acciones.alElegirLibre(actual.box, a, b);
}
