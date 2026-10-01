/**
 * Creación segura de nodos. Todo texto se asigna con `textContent`: nunca se usa
 * `innerHTML` con datos, así un nombre como `<b>x</b>` se muestra literal.
 */

type Hijo = Node | string | null | undefined | false;

interface Propiedades {
  clase?: string;
  texto?: string;
  atributos?: Record<string, string>;
  alHacerClic?: (evento: Event) => void;
}

/** Crea un elemento con propiedades e hijos. */
export function el<K extends keyof HTMLElementTagNameMap>(
  etiqueta: K,
  propiedades: Propiedades = {},
  ...hijos: Hijo[]
): HTMLElementTagNameMap[K] {
  const nodo = document.createElement(etiqueta);
  if (propiedades.clase) nodo.className = propiedades.clase;
  if (propiedades.texto !== undefined) nodo.textContent = propiedades.texto;
  for (const [nombre, valor] of Object.entries(propiedades.atributos ?? {})) {
    nodo.setAttribute(nombre, valor);
  }
  if (propiedades.alHacerClic) nodo.addEventListener("click", propiedades.alHacerClic);
  for (const hijo of hijos) {
    if (hijo) nodo.append(hijo);
  }
  return nodo;
}

/** Busca un elemento por id y falla en voz alta si no existe (error de programación). */
export function porId<T extends HTMLElement>(id: string): T {
  const nodo = document.getElementById(id);
  if (!nodo) throw new Error(`Falta el elemento #${id} en index.html`);
  return nodo as T;
}

/** Botón estándar. */
export function boton(
  texto: string,
  alHacerClic: () => void,
  variante: "primario" | "secundario" | "peligro" = "secundario",
): HTMLButtonElement {
  return el("button", {
    clase: `boton boton--${variante}`,
    texto,
    atributos: { type: "button" },
    alHacerClic: () => alHacerClic(),
  });
}

/** Campo de formulario con etiqueta asociada. */
export function campo(etiqueta: string, control: HTMLElement, ayuda?: HTMLElement): HTMLLabelElement {
  return el("label", { clase: "campo" }, el("span", { clase: "campo__etiqueta", texto: etiqueta }), control, ayuda);
}
