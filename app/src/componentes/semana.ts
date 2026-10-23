/**
 * Vista semanal: boxes en filas y, por cada día hábil, dos columnas (mañana y
 * tarde), igual que "BOX AM / BOX PM" en la planilla del CESFAM. Cada celda dice
 * quién tiene el box en ese medio día; la última fila lista los boxes libres de
 * cada medio día (la hoja "BOX DISPONIBLE" que antes se armaba a mano).
 */
import { el } from "../dom";
import { coincideFiltro, estado, rangoHorario } from "../estado";
import { diaCorto, diasHabilesDeSemana, hoy } from "../fechas";
import { textos } from "../textos";
import type { BoxAtencion, Horario, Reserva } from "../tipos";

export interface AccionesSemana {
  /** Medio día completamente libre: reservar desde el inicio de ese periodo. */
  alReservar: (box: BoxAtencion, fecha: string, desde: number) => void;
  /** Medio día con reservas: abrir el día para ver el detalle. */
  alVerDia: (fecha: string) => void;
}

/** Un medio día: bloques `[desde, hasta]` inclusive. */
interface Periodo {
  desde: number;
  hasta: number;
  etiqueta: string;
}

function periodos(horario: Horario): [Periodo, Periodo] {
  const tarde = horario.bloque_inicio_tarde;
  return [
    { desde: 0, hasta: tarde - 1, etiqueta: textos.semana.am },
    { desde: tarde, hasta: horario.cantidad_bloques - 1, etiqueta: textos.semana.pm },
  ];
}

/** Reservas por `"boxId|fecha"`, ordenadas por bloque. */
function indicePorBoxYDia(reservas: Reserva[]): Map<string, Reserva[]> {
  const indice = new Map<string, Reserva[]>();
  for (const r of [...reservas].sort((a, b) => a.bloque - b.bloque)) {
    const clave = `${r.box_id}|${r.fecha}`;
    const lista = indice.get(clave) ?? [];
    lista.push(r);
    indice.set(clave, lista);
  }
  return indice;
}

/** Funcionarios del periodo, en orden de aparición, con su horario para el tooltip. */
function resumir(reservas: Reserva[]): Array<{ nombre: string; horas: string }> {
  const porNombre = new Map<string, { desde: number; hasta: number }>();
  for (const r of reservas) {
    const actual = porNombre.get(r.funcionario);
    if (actual) actual.hasta = Math.max(actual.hasta, r.bloque);
    else porNombre.set(r.funcionario, { desde: r.bloque, hasta: r.bloque });
  }
  return [...porNombre].map(([nombre, { desde, hasta }]) => ({ nombre, horas: rangoHorario(desde, hasta) }));
}

export function renderizarSemana(contenedor: HTMLElement, acciones: AccionesSemana): void {
  const { horario, boxes, reservas, conexion } = estado;
  contenedor.classList.toggle("grilla--sin-conexion", conexion.tipo !== "conectado");
  if (!horario) {
    contenedor.replaceChildren(el("p", { clase: "grilla__mensaje", texto: textos.grilla.cargando }));
    return;
  }
  if (boxes.length === 0) {
    contenedor.replaceChildren(el("p", { clase: "grilla__mensaje", texto: textos.grilla.sinBoxes }));
    return;
  }

  const dias = diasHabilesDeSemana(estado.fecha);
  const medios = periodos(horario);
  const indice = indicePorBoxYDia(reservas);
  const deshabilitada = conexion.tipo !== "conectado";
  const libres = new Map<string, string[]>(); // "fecha|periodo" → boxes libres

  const filas = boxes.map((box) => {
    const celdas = dias.flatMap((fecha) =>
      medios.map((periodo, p) => {
        const delPeriodo = (indice.get(`${box.id}|${fecha}`) ?? []).filter(
          (r) => r.bloque >= periodo.desde && r.bloque <= periodo.hasta,
        );
        if (delPeriodo.length === 0) {
          const clave = `${fecha}|${p}`;
          libres.set(clave, [...(libres.get(clave) ?? []), box.nombre]);
        }
        const td = el("td", { clase: p === 0 ? "semana__inicio-dia" : "" });
        td.append(celda(box, fecha, periodo, delPeriodo, deshabilitada, acciones));
        return td;
      }),
    );
    return el(
      "tr",
      {},
      el("th", { clase: "grilla__box", texto: box.nombre, atributos: { scope: "row", title: box.nombre } }),
      ...celdas,
    );
  });

  const tabla = el(
    "table",
    { clase: "grilla__tabla semana" },
    columnas(dias.length * 2),
    encabezado(dias, medios),
    el("tbody", {}, ...filas),
    pie(dias, medios, libres, boxes.length),
  );
  contenedor.replaceChildren(tabla);
}

function columnas(cantidad: number): HTMLTableColElement {
  const grupo = el("colgroup", {}, el("col", { clase: "grilla__col-box" }));
  for (let i = 0; i < cantidad; i++) grupo.append(el("col"));
  return grupo;
}

function encabezado(dias: string[], medios: Periodo[]): HTMLTableSectionElement {
  const fechas = el(
    "tr",
    {},
    el("th", { clase: "grilla__esquina", texto: textos.grilla.columnaBox, atributos: { scope: "col", rowspan: "2" } }),
    ...dias.map((fecha) =>
      el("th", {
        clase: `grilla__periodo semana__inicio-dia${fecha === hoy() ? " grilla__hora--actual" : ""}`,
        texto: diaCorto(fecha),
        atributos: { scope: "colgroup", colspan: String(medios.length) },
      }),
    ),
  );
  const amPm = el(
    "tr",
    {},
    ...dias.flatMap(() =>
      medios.map((periodo, p) =>
        el("th", {
          clase: `grilla__hora${p === 0 ? " semana__inicio-dia" : ""}`,
          texto: periodo.etiqueta,
          atributos: { scope: "col", title: rangoHorario(periodo.desde, periodo.hasta) },
        }),
      ),
    ),
  );
  return el("thead", {}, fechas, amPm);
}

function celda(
  box: BoxAtencion,
  fecha: string,
  periodo: Periodo,
  reservas: Reserva[],
  deshabilitada: boolean,
  acciones: AccionesSemana,
): HTMLButtonElement {
  if (reservas.length === 0) {
    const etiqueta = textos.semana.reservarEn(box.nombre, diaCorto(fecha), periodo.etiqueta);
    const boton = el("button", {
      clase: "celda celda--libre",
      atributos: { type: "button", title: etiqueta, "aria-label": etiqueta, "data-texto": "+" },
      alHacerClic: () => acciones.alReservar(box, fecha, periodo.desde),
    });
    boton.disabled = deshabilitada;
    return boton;
  }

  const personas = resumir(reservas);
  const completo = reservas.length === periodo.hasta - periodo.desde + 1;
  const coincidencias = personas.map((p) => coincideFiltro(p.nombre));
  const clases = ["celda", completo ? "celda--ocupada" : "celda--parcial"];
  if (coincidencias.some((c) => c === true)) clases.push("celda--destacada");
  else if (coincidencias.some((c) => c === false)) clases.push("celda--atenuada");

  const detalle = personas.map((p) => `${p.nombre} (${p.horas})`).join(" · ");
  const visibles = personas.slice(0, 2);
  const boton = el(
    "button",
    {
      clase: clases.join(" "),
      atributos: { type: "button", title: detalle, "aria-label": `${diaCorto(fecha)} ${periodo.etiqueta}: ${detalle}` },
      alHacerClic: () => acciones.alVerDia(fecha),
    },
    ...visibles.map((p) => el("span", { clase: "celda__nombre", texto: p.nombre })),
    personas.length > visibles.length
      ? el("span", { clase: "celda__motivo", texto: textos.semana.yOtros(personas.length - visibles.length) })
      : null,
  );
  boton.disabled = deshabilitada;
  return boton;
}

/** Fila "Libres": boxes sin ninguna reserva en cada medio día. */
function pie(dias: string[], medios: Periodo[], libres: Map<string, string[]>, totalBoxes: number): HTMLTableSectionElement {
  const corto = (nombre: string) => nombre.replace(/^box\s*/i, "");
  return el(
    "tfoot",
    {},
    el(
      "tr",
      {},
      el("th", { clase: "grilla__box semana__libres-titulo", texto: textos.semana.libres, atributos: { scope: "row" } }),
      ...dias.flatMap((fecha) =>
        medios.map((_, p) => {
          const nombres = libres.get(`${fecha}|${p}`) ?? [];
          let texto = nombres.map(corto).join(" · ");
          if (nombres.length === 0) texto = textos.semana.ninguno;
          else if (nombres.length === totalBoxes) texto = textos.semana.todos;
          return el("td", {
            clase: `semana__libres${p === 0 ? " semana__inicio-dia" : ""}`,
            texto,
            atributos: { title: nombres.join(", ") || textos.semana.ninguno },
          });
        }),
      ),
    ),
  );
}
