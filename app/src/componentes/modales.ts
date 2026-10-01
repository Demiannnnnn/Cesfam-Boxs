/**
 * Modales de la app: reservar (RF-03), detalle/cancelar (RF-04), boxes (RF-06),
 * conexión manual (RF-09) y exportar (RF-12).
 *
 * Cada modal recibe sus dependencias por parámetro y avisa al terminar mediante
 * callbacks; no conoce el ciclo de refresco.
 */
import { save } from "@tauri-apps/plugin-dialog";

import { api } from "../api";
import { boton, campo, el } from "../dom";
import { etiquetaBloque, estado, nombreBox } from "../estado";
import { fechaCorta, fechaHora, finDeMes, inicioDeMes } from "../fechas";
import { textos } from "../textos";
import type { BoxAtencion, ErrorUi, Reserva, ResultadoConexion } from "../tipos";
import { mostrarAviso } from "./aviso";
import { abrirDialogo, confirmar } from "./dialogo";

/** Deshabilita un botón mientras corre `tarea` (evita dobles envíos). */
async function mientrasOcupado<T>(
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

function soloLectura(etiqueta: string, valor: string): HTMLElement {
  return el(
    "div",
    { clase: "dato" },
    el("span", { clase: "dato__etiqueta", texto: etiqueta }),
    el("span", { clase: "dato__valor", texto: valor }),
  );
}

// ─── Reservar ──────────────────────────────────────────────────────────────

export interface AccionesReserva {
  alReservar: (reserva: Reserva) => void;
  /** Error que obliga a refrescar la grilla (409, box inactivo). */
  alConflicto: (error: ErrorUi) => void;
}

export function abrirReservar(box: BoxAtencion, bloque: number, acciones: AccionesReserva): void {
  const dialogo = abrirDialogo(textos.reserva.titulo);
  const fecha = estado.fecha;

  const funcionario = el("input", {
    atributos: { type: "text", maxlength: "80", required: "", autocomplete: "off" },
  });
  funcionario.value = estado.ultimoFuncionario;
  const motivo = el("input", { atributos: { type: "text", maxlength: "200", autocomplete: "off" } });

  const enviar = el("button", {
    clase: "boton boton--primario",
    texto: textos.reserva.confirmar,
    atributos: { type: "submit" },
  });
  const formulario = el(
    "form",
    { clase: "formulario" },
    el(
      "div",
      { clase: "datos" },
      soloLectura(textos.reserva.box, box.nombre),
      soloLectura(textos.reserva.fecha, fechaCorta(fecha)),
      soloLectura(textos.reserva.hora, etiquetaBloque(bloque)),
    ),
    campo(textos.reserva.funcionario, funcionario),
    campo(textos.reserva.motivo, motivo),
  );
  formulario.id = "formulario-reserva";
  enviar.setAttribute("form", formulario.id);

  formulario.addEventListener("submit", async (evento) => {
    evento.preventDefault();
    const nombre = funcionario.value.trim();
    if (!nombre) {
      dialogo.mostrarError(textos.reserva.funcionarioObligatorio);
      funcionario.focus();
      return;
    }
    dialogo.mostrarError(null);
    try {
      const reserva = await mientrasOcupado(enviar, textos.reserva.enviando, () =>
        api.crearReserva({
          box_id: box.id,
          fecha,
          bloque,
          funcionario: nombre,
          motivo: motivo.value.trim() || null,
        }),
      );
      estado.ultimoFuncionario = reserva.funcionario;
      dialogo.cerrar();
      acciones.alReservar(reserva);
    } catch (error) {
      const e = error as ErrorUi;
      if (e.codigo === "bloque_ocupado" || e.codigo === "box_inactivo") {
        dialogo.cerrar();
        acciones.alConflicto(e);
      } else {
        dialogo.mostrarError(e.mensaje);
      }
    }
  });

  dialogo.cuerpo.append(formulario);
  dialogo.pie.prepend(enviar);
  (funcionario.value ? motivo : funcionario).focus();
  if (funcionario.value) funcionario.select();
}

// ─── Detalle y cancelación ─────────────────────────────────────────────────

export function abrirDetalle(reserva: Reserva, alCambiar: () => void): void {
  const dialogo = abrirDialogo(textos.detalle.titulo);
  const hora = etiquetaBloque(reserva.bloque);
  dialogo.cuerpo.append(
    el(
      "div",
      { clase: "datos" },
      soloLectura(textos.reserva.box, nombreBox(reserva.box_id)),
      soloLectura(textos.reserva.fecha, fechaCorta(reserva.fecha)),
      soloLectura(textos.reserva.hora, hora),
      soloLectura(textos.detalle.funcionario, reserva.funcionario),
      soloLectura(textos.detalle.motivo, reserva.motivo ?? textos.detalle.sinMotivo),
      soloLectura(textos.detalle.creada, fechaHora(reserva.creada_en)),
    ),
  );

  const cancelar = boton(
    textos.detalle.cancelar,
    async () => {
      const seguro = await confirmar(
        textos.detalle.confirmarCancelacion(reserva.funcionario, hora),
        textos.detalle.siCancelar,
      );
      if (!seguro) return;
      try {
        await api.cancelarReserva(reserva.id);
        mostrarAviso(textos.detalle.cancelada, "exito");
        dialogo.cerrar();
      } catch (error) {
        const e = error as ErrorUi;
        if (e.codigo === "no_encontrado") {
          mostrarAviso(textos.detalle.yaCancelada);
          dialogo.cerrar();
        } else {
          dialogo.mostrarError(e.mensaje);
          return;
        }
      }
      alCambiar();
    },
    "peligro",
  );
  dialogo.pie.prepend(cancelar);
}

// ─── Configuración de boxes ────────────────────────────────────────────────

export function abrirBoxes(alCerrar: () => void): void {
  const dialogo = abrirDialogo(textos.boxes.titulo, alCerrar);
  const lista = el("div", { clase: "lista-boxes" });
  const nuevo = el("input", { atributos: { type: "text", maxlength: "40" } });

  const recargar = async () => {
    try {
      const boxes = await api.listarBoxes(true);
      lista.replaceChildren(
        ...(boxes.length ? boxes.map((b) => filaBox(b, dialogo.mostrarError, recargar)) : [
          el("p", { texto: textos.boxes.vacio }),
        ]),
      );
      dialogo.mostrarError(null);
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  };

  const agregar = el("button", {
    clase: "boton boton--primario",
    texto: textos.boxes.agregar,
    atributos: { type: "submit" },
  });
  const formulario = el("form", { clase: "lista-boxes__nuevo" }, campo(textos.boxes.nuevo, nuevo), agregar);
  formulario.addEventListener("submit", async (evento) => {
    evento.preventDefault();
    try {
      await api.crearBox(nuevo.value);
      nuevo.value = "";
      await recargar();
      nuevo.focus();
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  });

  dialogo.cuerpo.append(lista, formulario);
  void recargar();
  nuevo.focus();
}

function filaBox(
  box: BoxAtencion,
  mostrarError: (mensaje: string | null) => void,
  recargar: () => Promise<void>,
): HTMLElement {
  const nombre = el("input", {
    atributos: { type: "text", maxlength: "40", "aria-label": textos.boxes.nombre },
  });
  nombre.value = box.nombre;
  const activo = el("input", { atributos: { type: "checkbox" } });
  activo.checked = box.activo;

  const guardar = boton(textos.boxes.guardar, async () => {
    try {
      await api.actualizarBox(box.id, { nombre: nombre.value });
      await recargar();
    } catch (error) {
      mostrarError((error as ErrorUi).mensaje);
    }
  });

  activo.addEventListener("change", async () => {
    const quiereActivo = activo.checked;
    if (!quiereActivo) {
      const seguro = await confirmar(textos.boxes.confirmarDesactivar(box.nombre), textos.boxes.siDesactivar);
      if (!seguro) {
        activo.checked = true;
        return;
      }
    }
    try {
      await api.actualizarBox(box.id, { activo: quiereActivo });
      await recargar();
    } catch (error) {
      activo.checked = !quiereActivo;
      mostrarError((error as ErrorUi).mensaje);
    }
  });

  return el(
    "div",
    { clase: `lista-boxes__fila${box.activo ? "" : " lista-boxes__fila--inactivo"}` },
    nombre,
    guardar,
    el("label", { clase: "casilla" }, activo, el("span", { texto: textos.boxes.activo })),
  );
}

// ─── Conexión manual ───────────────────────────────────────────────────────

/**
 * Pide la dirección del servidor hasta lograr conectar. No se puede cerrar sin
 * conectar: sin servidor la app no tiene nada que mostrar.
 */
export function abrirConexionManual(mensaje: string, alConectar: (r: ResultadoConexion) => void): void {
  const dialogo = abrirDialogo(textos.conexionManual.titulo, undefined, false);
  const direccion = el("input", { atributos: { type: "text", autocomplete: "off", spellcheck: "false" } });

  const conectar = el("button", {
    clase: "boton boton--primario",
    texto: textos.conexionManual.conectar,
    atributos: { type: "submit", form: "formulario-conexion" },
  });
  const buscar = el("button", {
    clase: "boton boton--secundario",
    texto: textos.conexionManual.buscar,
    atributos: { type: "button" },
  });

  const terminar = (resultado: ResultadoConexion) => {
    if (resultado.tipo === "conectado") {
      dialogo.cerrar();
      alConectar(resultado);
    } else {
      dialogo.mostrarError(resultado.mensaje);
    }
  };

  const formulario = el(
    "form",
    { clase: "formulario" },
    el("p", { texto: mensaje }),
    el("p", { clase: "ayuda", texto: textos.conexionManual.explicacion }),
    campo(textos.conexionManual.campo, direccion),
  );
  formulario.id = "formulario-conexion";
  formulario.addEventListener("submit", async (evento) => {
    evento.preventDefault();
    dialogo.mostrarError(null);
    try {
      terminar(await mientrasOcupado(conectar, textos.conexionManual.buscando, () => api.conectarManual(direccion.value)));
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
      direccion.focus();
    }
  });
  buscar.addEventListener("click", async () => {
    dialogo.mostrarError(null);
    try {
      terminar(await mientrasOcupado(buscar, textos.conexionManual.buscando, () => api.conectar()));
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  });

  dialogo.cuerpo.append(formulario);
  dialogo.pie.append(buscar, conectar);
  direccion.focus();
}

// ─── Exportar ──────────────────────────────────────────────────────────────

export function abrirExportar(): void {
  const dialogo = abrirDialogo(textos.exportar.titulo);
  const desde = el("input", { atributos: { type: "date", required: "" } });
  const hasta = el("input", { atributos: { type: "date", required: "" } });
  desde.value = inicioDeMes(estado.fecha);
  hasta.value = finDeMes(estado.fecha);

  const exportar = boton(
    textos.exportar.confirmar,
    async () => {
      dialogo.mostrarError(null);
      const ruta = await save({
        defaultPath: `reservas-${desde.value}-a-${hasta.value}.csv`,
        filters: [{ name: textos.exportar.filtro, extensions: ["csv"] }],
      });
      if (!ruta) return;
      try {
        const filas = await mientrasOcupado(exportar, "…", () => api.exportarCsv(desde.value, hasta.value, ruta));
        mostrarAviso(textos.exportar.listo(filas), "exito");
        dialogo.cerrar();
      } catch (error) {
        dialogo.mostrarError((error as ErrorUi).mensaje);
      }
    },
    "primario",
  );

  dialogo.cuerpo.append(
    el("div", { clase: "formulario formulario--fila" }, campo(textos.exportar.desde, desde), campo(textos.exportar.hasta, hasta)),
  );
  dialogo.pie.prepend(exportar);
}
