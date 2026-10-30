/** Piezas comunes de los formularios de los modales. */
import { colorTexto, PALETA_ETIQUETAS } from "../colores";
import { el } from "../dom";
import { textos } from "../textos";
import type { Etiqueta } from "../tipos";

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

/** Chip coloreado con el nombre de la etiqueta y texto negro/blanco por contraste WCAG. */
export function chipEtiqueta(etiqueta: Etiqueta, claseExtra = ""): HTMLElement {
  const chip = el("span", {
    clase: `chip-etiqueta${claseExtra ? ` ${claseExtra}` : ""}`,
    texto: etiqueta.nombre,
    atributos: { title: etiqueta.nombre },
  });
  chip.style.backgroundColor = etiqueta.color;
  chip.style.color = colorTexto(etiqueta.color);
  return chip;
}

export interface SelectorEtiqueta {
  raiz: HTMLElement;
  valor: () => number | null;
  reiniciar: () => void;
}

/**
 * Selector `<select>` de etiqueta ("Sin etiqueta" + etiquetas activas) acompañado de
 * una muestra del color elegido. Cubre RF-13 y RF-14.
 */
export function crearSelectorEtiqueta(
  etiquetas: Etiqueta[],
  actual: number | null,
  textoSinEtiqueta: string,
  etiquetaAria: string,
): SelectorEtiqueta {
  const muestra = el("span", {
    clase: "selector-etiqueta__muestra",
    atributos: { "aria-hidden": "true" },
  });
  const select = el("select", { atributos: { "aria-label": etiquetaAria } });
  select.append(el("option", { texto: textoSinEtiqueta, atributos: { value: "" } }));

  const visibles = etiquetas.filter((e) => e.activo || e.id === actual);
  for (const etiq of visibles) {
    select.append(el("option", { texto: etiq.nombre, atributos: { value: String(etiq.id) } }));
  }
  select.value = actual !== null && visibles.some((e) => e.id === actual) ? String(actual) : "";

  const actualizarMuestra = () => {
    const id = select.value ? Number(select.value) : null;
    const elegida = id !== null ? etiquetas.find((e) => e.id === id) : undefined;
    if (elegida) {
      muestra.style.backgroundColor = elegida.color;
      muestra.classList.remove("selector-etiqueta__muestra--vacia");
    } else {
      muestra.style.backgroundColor = "transparent";
      muestra.classList.add("selector-etiqueta__muestra--vacia");
    }
  };
  select.addEventListener("change", actualizarMuestra);
  actualizarMuestra();

  return {
    raiz: el("div", { clase: "selector-etiqueta" }, muestra, select),
    valor: () => (select.value ? Number(select.value) : null),
    reiniciar: () => {
      select.value = "";
      actualizarMuestra();
    },
  };
}

export interface SelectorColor {
  raiz: HTMLElement;
  valor: () => string;
  fijar: (color: string) => void;
}

/** `<input type="color">` más paleta de 8 colores sugeridos (RF-14). */
export function crearSelectorColor(valorInicial: string): SelectorColor {
  const input = el("input", {
    clase: "selector-color__input",
    atributos: { type: "color", "aria-label": textos.etiquetas.color },
  });
  input.value = valorInicial.toLowerCase();

  const botones = PALETA_ETIQUETAS.map((color) => {
    const btn = el("button", {
      clase: "paleta-colores__muestra",
      atributos: {
        type: "button",
        title: color,
        "aria-label": textos.etiquetas.elegirColor(color),
      },
      alHacerClic: () => fijar(color),
    });
    btn.style.backgroundColor = color;
    btn.dataset["color"] = color;
    return btn;
  });

  const sincronizarPaleta = () => {
    const actual = input.value.toLowerCase();
    for (const btn of botones) {
      btn.classList.toggle("paleta-colores__muestra--activa", btn.dataset["color"] === actual);
    }
  };
  const fijar = (color: string) => {
    input.value = color.toLowerCase();
    sincronizarPaleta();
  };
  input.addEventListener("input", sincronizarPaleta);
  sincronizarPaleta();

  return {
    raiz: el("div", { clase: "selector-color" }, input, el("div", { clase: "paleta-colores" }, ...botones)),
    valor: () => input.value.toLowerCase(),
    fijar,
  };
}
