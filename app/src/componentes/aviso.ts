/** Aviso temporal en la parte superior (p. ej. "Ese bloque acaba de ser tomado"). */
import { porId } from "../dom";

const DURACION_MS = 6000;
let temporizador: number | undefined;

export function mostrarAviso(mensaje: string, tipo: "error" | "exito" = "error"): void {
  const aviso = porId<HTMLDivElement>("aviso");
  aviso.textContent = mensaje;
  aviso.className = `aviso aviso--${tipo}`;
  aviso.hidden = false;
  window.clearTimeout(temporizador);
  temporizador = window.setTimeout(() => {
    aviso.hidden = true;
  }, DURACION_MS);
}
