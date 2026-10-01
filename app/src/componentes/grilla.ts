/**
 * Grilla del día (RF-01): boxes activos en columnas, bloques en filas.
 * Se reconstruye completa desde el estado en cada render.
 */
import { el } from "../dom";
import { bloquesTerminados, hoy } from "../fechas";
import { estado, indiceReservas } from "../estado";
import { textos } from "../textos";
import type { BoxAtencion, Reserva } from "../tipos";

export interface AccionesGrilla {
  alElegirLibre: (box: BoxAtencion, bloque: number) => void;
  alElegirOcupada: (reserva: Reserva) => void;
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
  const enCurso = fecha === hoy() && dentroDelHorario(horario.hora_inicio) ? terminados : null;
  const indice = indiceReservas(reservas);
  const deshabilitada = conexion.tipo !== "conectado";

  const encabezado = el(
    "tr",
    {},
    el("th", { clase: "grilla__hora", texto: textos.grilla.columnaHora, atributos: { scope: "col" } }),
    ...boxes.map((box) => el("th", { texto: box.nombre, atributos: { scope: "col" } })),
  );

  const filas = horario.etiquetas.map((etiqueta, bloque) => {
    const clases = ["grilla__fila"];
    if (enCurso === bloque) clases.push("grilla__fila--actual");
    if (bloque < terminados) clases.push("grilla__fila--pasada");
    return el(
      "tr",
      { clase: clases.join(" ") },
      el("th", { clase: "grilla__hora", texto: etiqueta, atributos: { scope: "row" } }),
      ...boxes.map((box) => celda(box, bloque, indice.get(`${box.id}-${bloque}`), deshabilitada, acciones)),
    );
  });

  const tabla = el(
    "table",
    { clase: "grilla__tabla" },
    el("thead", {}, encabezado),
    el("tbody", {}, ...filas),
  );
  contenedor.replaceChildren(tabla);
}

function dentroDelHorario(horaInicio: number): boolean {
  return new Date().getHours() >= horaInicio;
}

function celda(
  box: BoxAtencion,
  bloque: number,
  reserva: Reserva | undefined,
  deshabilitada: boolean,
  acciones: AccionesGrilla,
): HTMLTableCellElement {
  const boton = reserva
    ? el(
        "button",
        {
          clase: "celda celda--ocupada",
          atributos: { type: "button", title: reserva.motivo ?? reserva.funcionario },
          alHacerClic: () => acciones.alElegirOcupada(reserva),
        },
        el("span", { clase: "celda__nombre", texto: reserva.funcionario }),
        reserva.motivo ? el("span", { clase: "celda__motivo", texto: reserva.motivo }) : null,
      )
    : el("button", {
        clase: "celda celda--libre",
        atributos: {
          type: "button",
          "aria-label": `${textos.grilla.reservar} ${box.nombre}`,
          "data-texto": textos.grilla.reservar,
        },
        alHacerClic: () => acciones.alElegirLibre(box, bloque),
      });
  boton.disabled = deshabilitada;
  return el("td", {}, boton);
}
