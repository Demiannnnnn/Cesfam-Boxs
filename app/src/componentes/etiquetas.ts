/**
 * Modal de administración de etiquetas con color (RF-14):
 * lista (incluyendo inactivas) con nombre, selector de color (input + paleta de 8),
 * casilla Activa, botón Guardar y errores bajo cada fila; abajo formulario para
 * crear una etiqueta nueva.
 */
import { api } from "../api";
import { PALETA_ETIQUETAS } from "../colores";
import { boton, campo, el } from "../dom";
import { estado, puedeEscribir } from "../estado";
import { textos } from "../textos";
import type { ErrorUi, Etiqueta } from "../tipos";
import { abrirDialogo } from "./dialogo";
import { crearSelectorColor } from "./formulario";

export function abrirEtiquetas(alCerrar: () => void): void {
  if (!puedeEscribir()) return;
  const dialogo = abrirDialogo(textos.etiquetas.titulo, alCerrar);
  const lista = el("div", { clase: "lista-admin" });
  const inputNombre = el("input", { atributos: { type: "text", maxlength: "40" } });
  const selectorColorNuevo = crearSelectorColor(PALETA_ETIQUETAS[0] ?? "#2563eb");
  const errorNuevo = el("p", { clase: "fila-admin__error", atributos: { role: "alert" } });
  errorNuevo.hidden = true;

  const mostrarErrorNuevo = (mensaje: string | null) => {
    errorNuevo.textContent = mensaje ?? "";
    errorNuevo.hidden = mensaje === null;
  };

  const recargar = async () => {
    try {
      const etiquetas = await api.listarEtiquetas(true);
      estado.etiquetas = etiquetas;
      lista.replaceChildren(
        ...(etiquetas.length
          ? etiquetas.map((etiq) => filaEtiqueta(etiq, recargar))
          : [el("p", { texto: textos.etiquetas.vacio })]),
      );
      dialogo.mostrarError(null);
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  };

  const agregar = el("button", {
    clase: "boton boton--primario",
    texto: textos.etiquetas.agregar,
    atributos: { type: "submit" },
  });
  const formulario = el(
    "form",
    { clase: "lista-admin__nuevo" },
    el(
      "div",
      { clase: "lista-admin__nuevo-controles" },
      campo(textos.etiquetas.nueva, inputNombre),
      campo(textos.etiquetas.color, selectorColorNuevo.raiz),
      agregar,
    ),
    errorNuevo,
  );

  formulario.addEventListener("submit", async (evento) => {
    evento.preventDefault();
    mostrarErrorNuevo(null);
    try {
      await api.crearEtiqueta(inputNombre.value, selectorColorNuevo.valor());
      inputNombre.value = "";
      await recargar();
      inputNombre.focus();
    } catch (error) {
      mostrarErrorNuevo((error as ErrorUi).mensaje);
    }
  });

  dialogo.cuerpo.append(lista, formulario);
  void recargar();
  inputNombre.focus();
}

function filaEtiqueta(etiqueta: Etiqueta, recargar: () => Promise<void>): HTMLElement {
  const nombre = el("input", {
    atributos: { type: "text", maxlength: "40", "aria-label": textos.etiquetas.nombre },
  });
  nombre.value = etiqueta.nombre;
  const selectorColor = crearSelectorColor(etiqueta.color);
  const activa = el("input", { atributos: { type: "checkbox" } });
  activa.checked = etiqueta.activo;
  const errorFila = el("p", { clase: "fila-admin__error", atributos: { role: "alert" } });
  errorFila.hidden = true;

  const mostrarErrorFila = (mensaje: string | null) => {
    errorFila.textContent = mensaje ?? "";
    errorFila.hidden = mensaje === null;
  };

  const guardar = boton(textos.etiquetas.guardar, async () => {
    mostrarErrorFila(null);
    try {
      await api.actualizarEtiqueta(etiqueta.id, {
        nombre: nombre.value,
        color: selectorColor.valor(),
        activo: activa.checked,
      });
      await recargar();
    } catch (error) {
      mostrarErrorFila((error as ErrorUi).mensaje);
    }
  });

  activa.addEventListener("change", async () => {
    mostrarErrorFila(null);
    try {
      await api.actualizarEtiqueta(etiqueta.id, { activo: activa.checked });
      await recargar();
    } catch (error) {
      activa.checked = !activa.checked;
      mostrarErrorFila((error as ErrorUi).mensaje);
    }
  });

  return el(
    "div",
    { clase: `fila-admin${etiqueta.activo ? "" : " fila-admin--inactivo"}` },
    el(
      "div",
      { clase: "fila-admin__controles fila-admin__controles--etiqueta" },
      nombre,
      selectorColor.raiz,
      el("label", { clase: "casilla" }, activa, el("span", { texto: textos.etiquetas.activa })),
      guardar,
    ),
    errorFila,
  );
}
