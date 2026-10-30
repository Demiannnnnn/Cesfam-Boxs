/**
 * Modales de la app: detalle/cancelar (RF-04), boxes (RF-06, RF-14), conexión manual
 * (RF-09) y exportar (RF-12). Reserva, funcionarios y etiquetas viven en sus propios
 * módulos.
 */
import { save } from "@tauri-apps/plugin-dialog";

import { api } from "../api";
import { boton, campo, el } from "../dom";
import { estado, etiquetaDeFuncionario, nombreBox, rangoHorario } from "../estado";
import { fechaCorta, fechaHora, finDeMes, inicioDeMes } from "../fechas";
import { textos } from "../textos";
import type { Tramo } from "../tramos";
import type { BoxAtencion, ErrorUi, Etiqueta, ResultadoConexion } from "../tipos";
import { mostrarAviso } from "./aviso";
import { abrirDialogo, confirmar } from "./dialogo";
import { chipEtiqueta, crearSelectorEtiqueta, mientrasOcupado, soloLectura } from "./formulario";

// ─── Detalle y cancelación ─────────────────────────────────────────────────

function grupoDelTramo(tramo: Tramo): number | null {
  const grupo = tramo.reservas[0]?.grupo ?? null;
  return grupo !== null && tramo.reservas.every((r) => r.grupo === grupo) ? grupo : null;
}

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
  mostrarAviso(yaCanceladas === tramo.reservas.length ? textos.detalle.yaCancelada : textos.detalle.cancelada, yaCanceladas === tramo.reservas.length ? "error" : "exito");
  return true;
}

export function abrirDetalle(tramo: Tramo, alCambiar: () => void): void {
  const dialogo = abrirDialogo(textos.detalle.titulo);
  const horas = rangoHorario(tramo.desde, tramo.hasta);
  const primera = tramo.reservas[0];
  const fecha = primera?.fecha ?? estado.fecha;
  const grupo = grupoDelTramo(tramo);
  const etiqueta = etiquetaDeFuncionario(tramo.funcionario_id);
  const datoEtiqueta = etiqueta
    ? el("div", { clase: "dato" }, el("span", { clase: "dato__etiqueta", texto: textos.detalle.etiqueta }), el("span", { clase: "dato__valor" }, chipEtiqueta(etiqueta)))
    : null;
  dialogo.cuerpo.append(
    el(
      "div",
      { clase: "datos" },
      soloLectura(textos.reserva.box, nombreBox(tramo.box_id)),
      soloLectura(textos.reserva.fecha, fechaCorta(fecha)),
      soloLectura(textos.detalle.horario, horas),
      soloLectura(textos.detalle.funcionario, tramo.funcionario),
      datoEtiqueta,
      soloLectura(textos.detalle.motivo, tramo.motivo ?? textos.detalle.sinMotivo),
      soloLectura(textos.detalle.creada, primera ? fechaHora(primera.creada_en) : ""),
    ),
  );

  dialogo.pie.prepend(
    boton(
      grupo === null ? textos.detalle.cancelar : textos.detalle.cancelarDia,
      async () => {
        if (!(await confirmar(textos.detalle.confirmarCancelacion(tramo.funcionario, horas), textos.detalle.siCancelar))) return;
        if (await cancelarTramo(tramo, dialogo.mostrarError)) dialogo.cerrar();
        alCambiar();
      },
      "peligro",
    ),
  );

  if (grupo !== null) {
    dialogo.pie.prepend(
      boton(
        textos.detalle.cancelarSiguientes,
        async () => {
          const msg = textos.detalle.confirmarCancelacionSerie(tramo.funcionario, fechaCorta(fecha));
          if (!(await confirmar(msg, textos.detalle.siCancelar))) return;
          try {
            const { canceladas } = await api.cancelarGrupo(grupo, fecha);
            mostrarAviso(textos.detalle.serieCancelada(canceladas), "exito");
            dialogo.cerrar();
          } catch (error) {
            const e = error as ErrorUi;
            if (e.codigo !== "no_encontrado") return dialogo.mostrarError(e.mensaje);
            mostrarAviso(textos.detalle.yaCancelada);
            dialogo.cerrar();
          }
          alCambiar();
        },
        "peligro",
      ),
    );
  }
}

// ─── Configuración de boxes ────────────────────────────────────────────────

export function abrirBoxes(alCerrar: () => void): void {
  const dialogo = abrirDialogo(textos.boxes.titulo, alCerrar);
  const lista = el("div", { clase: "lista-admin" });
  const contenedorNuevo = el("div", {});

  const recargar = async () => {
    try {
      const [boxes, etiquetas] = await Promise.all([api.listarBoxes(true), api.listarEtiquetas(true)]);
      estado.etiquetas = etiquetas;
      lista.replaceChildren(
        ...(boxes.length ? boxes.map((b) => filaBox(b, etiquetas, recargar)) : [el("p", { texto: textos.boxes.vacio })]),
      );
      construirFormularioNuevo(etiquetas);
      dialogo.mostrarError(null);
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  };

  const construirFormularioNuevo = (etiquetas: Etiqueta[]) => {
    const nuevo = el("input", { atributos: { type: "text", maxlength: "40" } });
    const selectorEtiqueta = crearSelectorEtiqueta(etiquetas, null, textos.boxes.sinEtiqueta, textos.boxes.etiqueta);
    const agregar = el("button", { clase: "boton boton--primario", texto: textos.boxes.agregar, atributos: { type: "submit" } });
    const formulario = el(
      "form",
      { clase: "lista-admin__nuevo" },
      el(
        "div",
        { clase: "lista-admin__nuevo-controles" },
        campo(textos.boxes.nuevo, nuevo),
        campo(textos.boxes.etiqueta, selectorEtiqueta.raiz),
        agregar,
      ),
    );
    formulario.addEventListener("submit", async (evento) => {
      evento.preventDefault();
      try {
        await api.crearBox(nuevo.value, selectorEtiqueta.valor());
        await recargar();
      } catch (error) {
        dialogo.mostrarError((error as ErrorUi).mensaje);
      }
    });
    contenedorNuevo.replaceChildren(formulario);
  };

  dialogo.cuerpo.append(lista, contenedorNuevo);
  void recargar();
}

function filaBox(box: BoxAtencion, etiquetas: Etiqueta[], recargar: () => Promise<void>): HTMLElement {
  const nombre = el("input", { atributos: { type: "text", maxlength: "40", "aria-label": textos.boxes.nombre } });
  nombre.value = box.nombre;
  const selectorEtiqueta = crearSelectorEtiqueta(etiquetas, box.etiqueta_id, textos.boxes.sinEtiqueta, textos.boxes.etiqueta);
  const activo = el("input", { atributos: { type: "checkbox" } });
  activo.checked = box.activo;
  const errorFila = el("p", { clase: "fila-admin__error", atributos: { role: "alert" } });
  errorFila.hidden = true;

  const mostrarErrorFila = (mensaje: string | null) => {
    errorFila.textContent = mensaje ?? "";
    errorFila.hidden = mensaje === null;
  };

  const guardar = boton(textos.boxes.guardar, async () => {
    mostrarErrorFila(null);
    try {
      await api.actualizarBox(box.id, { nombre: nombre.value, etiqueta_id: selectorEtiqueta.valor() });
      await recargar();
    } catch (error) {
      mostrarErrorFila((error as ErrorUi).mensaje);
    }
  });

  activo.addEventListener("change", async () => {
    mostrarErrorFila(null);
    const quiereActivo = activo.checked;
    if (!quiereActivo && !(await confirmar(textos.boxes.confirmarDesactivar(box.nombre), textos.boxes.siDesactivar))) {
      activo.checked = true;
      return;
    }
    try {
      await api.actualizarBox(box.id, { activo: quiereActivo });
      await recargar();
    } catch (error) {
      activo.checked = !quiereActivo;
      mostrarErrorFila((error as ErrorUi).mensaje);
    }
  });

  return el(
    "div",
    { clase: `fila-admin${box.activo ? "" : " fila-admin--inactivo"}` },
    el(
      "div",
      { clase: "fila-admin__controles fila-admin__controles--box" },
      nombre,
      selectorEtiqueta.raiz,
      el("label", { clase: "casilla" }, activo, el("span", { texto: textos.boxes.activo })),
      guardar,
    ),
    errorFila,
  );
}

// ─── Conexión manual ───────────────────────────────────────────────────────

export function abrirConexionManual(mensaje: string, alConectar: (r: ResultadoConexion) => void): void {
  const dialogo = abrirDialogo(textos.conexionManual.titulo, undefined, false);
  const direccion = el("input", { atributos: { type: "text", autocomplete: "off", spellcheck: "false" } });
  const conectar = el("button", {
    clase: "boton boton--primario",
    texto: textos.conexionManual.conectar,
    atributos: { type: "submit", form: "formulario-conexion" },
  });
  const buscar = el("button", { clase: "boton boton--secundario", texto: textos.conexionManual.buscar, atributos: { type: "button" } });

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
