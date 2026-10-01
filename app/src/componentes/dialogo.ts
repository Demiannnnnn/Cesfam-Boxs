/**
 * Diálogo modal genérico sobre `<dialog>`. Lleva la cuenta de diálogos abiertos
 * para pausar el refresco automático mientras el usuario interactúa (RF-07).
 */
import { el } from "../dom";
import { textos } from "../textos";

let abiertos = 0;
const alCerrarTodos: Array<() => void> = [];

/** `true` si hay algún modal abierto. */
export function hayDialogoAbierto(): boolean {
  return abiertos > 0;
}

/** Registra una función que se llama cuando se cierra el último modal. */
export function alCerrarDialogos(funcion: () => void): void {
  alCerrarTodos.push(funcion);
}

export interface Dialogo {
  /** Contenedor del cuerpo, para agregar contenido. */
  cuerpo: HTMLElement;
  /** Zona de botones al pie. */
  pie: HTMLElement;
  /** Muestra un mensaje de error dentro del diálogo. */
  mostrarError: (mensaje: string | null) => void;
  cerrar: () => void;
}

/**
 * Abre un diálogo con título. Escape o el botón Cerrar lo cierran.
 * `alCerrar` se llama siempre al cerrarse.
 */
export function abrirDialogo(titulo: string, alCerrar?: () => void, permitirCerrar = true): Dialogo {
  const error = el("p", { clase: "dialogo__error", atributos: { role: "alert" } });
  error.hidden = true;
  const cuerpo = el("div", { clase: "dialogo__cuerpo" });
  const pie = el("div", { clase: "dialogo__pie" });
  const dialogo = el(
    "dialog",
    { clase: "dialogo" },
    el("h2", { clase: "dialogo__titulo", texto: titulo }),
    cuerpo,
    error,
    pie,
  );

  let cerrado = false;
  const cerrar = () => {
    if (cerrado) return;
    cerrado = true;
    dialogo.close();
    dialogo.remove();
    abiertos -= 1;
    alCerrar?.();
    if (abiertos === 0) alCerrarTodos.forEach((f) => f());
  };

  dialogo.addEventListener("cancel", (evento) => {
    evento.preventDefault();
    if (permitirCerrar) cerrar();
  });

  if (permitirCerrar) {
    const botonCerrar = el("button", {
      clase: "boton boton--secundario",
      texto: textos.comunes.cerrar,
      atributos: { type: "button" },
      alHacerClic: cerrar,
    });
    pie.append(botonCerrar);
  }

  document.body.append(dialogo);
  abiertos += 1;
  dialogo.showModal();

  return {
    cuerpo,
    pie,
    cerrar,
    mostrarError: (mensaje) => {
      error.textContent = mensaje ?? "";
      error.hidden = mensaje === null;
    },
  };
}

/** Pide confirmación con un diálogo propio (no `window.confirm`). */
export function confirmar(mensaje: string, textoConfirmar: string): Promise<boolean> {
  return new Promise((resolver) => {
    let respuesta = false;
    const dialogo = abrirDialogo(mensaje, () => resolver(respuesta));
    dialogo.cuerpo.remove();
    const si = el("button", {
      clase: "boton boton--peligro",
      texto: textoConfirmar,
      atributos: { type: "button" },
      alHacerClic: () => {
        respuesta = true;
        dialogo.cerrar();
      },
    });
    dialogo.pie.prepend(si);
    si.focus();
  });
}
