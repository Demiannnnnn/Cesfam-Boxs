/**
 * Modales de la app: detalle/cancelar (RF-04), boxes (RF-06), conexión manual
 * (RF-09) y exportar (RF-12). El de reserva vive en `reservar.ts`.
 *
 * Cada modal recibe sus dependencias por parámetro y avisa al terminar mediante
 * callbacks; no conoce el ciclo de refresco.
 */
import { save } from "@tauri-apps/plugin-dialog";

import { api } from "../api";
import { boton, campo, el } from "../dom";
import { estado, nombreBox, rangoHorario } from "../estado";
import { fechaCorta, fechaHora, finDeMes, inicioDeMes } from "../fechas";
import { textos } from "../textos";
import type { Tramo } from "../tramos";
import type { BoxAtencion, ErrorUi, ResultadoConexion } from "../tipos";
import { mostrarAviso } from "./aviso";
import { abrirDialogo, confirmar } from "./dialogo";
import { mientrasOcupado, soloLectura } from "./formulario";

// ─── Detalle y cancelación ─────────────────────────────────────────────────

/** Grupo común a todas las reservas del tramo, si lo hay (rango o serie). */
function grupoDelTramo(tramo: Tramo): number | null {
  const grupo = tramo.reservas[0]?.grupo ?? null;
  return grupo !== null && tramo.reservas.every((r) => r.grupo === grupo) ? grupo : null;
}

/** Cancela cada bloque del tramo. Devuelve `false` si falló algo distinto de "ya cancelada". */
async function cancelarTramo(tramo: Tramo, mostrarError: (m: string) => void): Promise<boolean> {
  let yaCanceladas = 0;
  for (const reserva of tramo.reservas) {
    try {
      await api.cancelarReserva(reserva.id);
    } catch (error) {
      const e = error as ErrorUi;
      if (e.codigo !== "no_encontrado") {
        mostrarError(e.mensaje);
        return false;
      }
      yaCanceladas += 1;
    }
  }
  if (yaCanceladas === tramo.reservas.length) mostrarAviso(textos.detalle.yaCancelada);
  else mostrarAviso(textos.detalle.cancelada, "exito");
  return true;
}

/**
 * Detalle de un tramo (bloques seguidos de una misma reserva). Cancelar libera
 * todos sus bloques: el usuario ve una sola barra y espera que desaparezca entera.
 * Si el tramo es parte de una serie, ofrece cancelar también las semanas siguientes.
 */
export function abrirDetalle(tramo: Tramo, alCambiar: () => void): void {
  const dialogo = abrirDialogo(textos.detalle.titulo);
  const horas = rangoHorario(tramo.desde, tramo.hasta);
  const primera = tramo.reservas[0];
  const fecha = primera?.fecha ?? estado.fecha;
  const grupo = grupoDelTramo(tramo);
  dialogo.cuerpo.append(
    el(
      "div",
      { clase: "datos" },
      soloLectura(textos.reserva.box, nombreBox(tramo.box_id)),
      soloLectura(textos.reserva.fecha, fechaCorta(fecha)),
      soloLectura(textos.detalle.horario, horas),
      soloLectura(textos.detalle.funcionario, tramo.funcionario),
      soloLectura(textos.detalle.motivo, tramo.motivo ?? textos.detalle.sinMotivo),
      soloLectura(textos.detalle.creada, primera ? fechaHora(primera.creada_en) : ""),
    ),
  );

  const cancelarDia = boton(
    grupo === null ? textos.detalle.cancelar : textos.detalle.cancelarDia,
    async () => {
      const seguro = await confirmar(textos.detalle.confirmarCancelacion(tramo.funcionario, horas), textos.detalle.siCancelar);
      if (!seguro) return;
      if (await cancelarTramo(tramo, dialogo.mostrarError)) dialogo.cerrar();
      alCambiar();
    },
    "peligro",
  );
  dialogo.pie.prepend(cancelarDia);

  if (grupo !== null) {
    const cancelarSerie = boton(
      textos.detalle.cancelarSiguientes,
      async () => {
        const seguro = await confirmar(
          textos.detalle.confirmarCancelacionSerie(tramo.funcionario, fechaCorta(fecha)),
          textos.detalle.siCancelar,
        );
        if (!seguro) return;
        try {
          const { canceladas } = await api.cancelarGrupo(grupo, fecha);
          mostrarAviso(textos.detalle.serieCancelada(canceladas), "exito");
          dialogo.cerrar();
        } catch (error) {
          const e = error as ErrorUi;
          if (e.codigo !== "no_encontrado") {
            dialogo.mostrarError(e.mensaje);
            return;
          }
          mostrarAviso(textos.detalle.yaCancelada);
          dialogo.cerrar();
        }
        alCambiar();
      },
      "peligro",
    );
    dialogo.pie.prepend(cancelarSerie);
  }
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
