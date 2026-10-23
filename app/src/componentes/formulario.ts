/** Piezas comunes de los formularios de los modales. */
import { el } from "../dom";

/** Deshabilita un botón mientras corre `tarea` (evita dobles envíos). */
export async function mientrasOcupado<T>(
  boton: HTMLButtonElement,
  textoOcupado: string,
  tarea: () => Promise<T>,
): Promise<T> {
  const original = boton.textContent;
  boton.disabled = true;
  boton.textContent = textoOcupado;
  try {
    return await tarea();
  } finally {
    boton.disabled = false;
    boton.textContent = original;
  }
}

export function soloLectura(etiqueta: string, valor: string): HTMLElement {
  return el(
    "div",
    { clase: "dato" },
    el("span", { clase: "dato__etiqueta", texto: etiqueta }),
    el("span", { clase: "dato__valor", texto: valor }),
  );
}
