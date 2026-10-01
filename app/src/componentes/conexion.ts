/** Indicador de estado de conexión (RF-08): texto + color, nunca solo color. */
import { el } from "../dom";
import { textos } from "../textos";
import type { EstadoConexion } from "../tipos";

export function renderizarConexion(contenedor: HTMLElement, conexion: EstadoConexion): void {
  const punto = el("span", { clase: "conexion__punto", atributos: { "aria-hidden": "true" } });
  let etiqueta: string;
  let detalle: string | null = null;
  switch (conexion.tipo) {
    case "conectado":
      etiqueta = textos.conexion.conectado(conexion.equipo);
      break;
    case "buscando":
      etiqueta = textos.conexion.buscando;
      break;
    case "sin_conexion":
      etiqueta = textos.conexion.sinConexion;
      detalle = conexion.mensaje;
      break;
  }
  contenedor.className = `conexion conexion--${conexion.tipo}`;
  contenedor.replaceChildren(punto, el("span", { clase: "conexion__texto", texto: etiqueta }));
  if (detalle) contenedor.append(el("span", { clase: "conexion__detalle", texto: detalle }));
  contenedor.title = detalle ?? etiqueta;
}
