/**
 * Modal de administración de funcionarios registrados (RF-13, RF-14):
 * buscador por nombre arriba (filtra en vivo), lista con nombre, selector de
 * etiqueta con muestra de color, casilla Activo, Guardar y errores bajo cada fila;
 * abajo formulario para registrar un nuevo funcionario.
 */
import { api } from "../api";
import { boton, campo, el } from "../dom";
import { estado, normalizarTexto } from "../estado";
import { textos } from "../textos";
import type { ErrorUi, Etiqueta, Funcionario } from "../tipos";
import { abrirDialogo } from "./dialogo";
import { crearSelectorEtiqueta } from "./formulario";

export function abrirFuncionarios(alCerrar: () => void): void {
  const dialogo = abrirDialogo(textos.funcionarios.titulo, alCerrar);
  const buscador = el("input", {
    clase: "buscador-modal",
    atributos: {
      type: "search",
      placeholder: textos.funcionarios.buscar,
      "aria-label": textos.funcionarios.buscar,
      autocomplete: "off",
    },
  });
  const lista = el("div", { clase: "lista-admin" });
  const contenedorNuevo = el("div", {});

  let funcionariosCargados: Funcionario[] = [];
  let etiquetasCargadas: Etiqueta[] = [];

  const pintarLista = () => {
    const filtro = normalizarTexto(buscador.value);
    const filtrados = filtro
      ? funcionariosCargados.filter((f) => normalizarTexto(f.nombre).includes(filtro))
      : funcionariosCargados;

    if (funcionariosCargados.length === 0) {
      lista.replaceChildren(el("p", { texto: textos.funcionarios.vacio }));
      return;
    }
    if (filtrados.length === 0) {
      lista.replaceChildren(el("p", { texto: textos.funcionarios.sinResultados }));
      return;
    }
    lista.replaceChildren(
      ...filtrados.map((f) => filaFuncionario(f, etiquetasCargadas, recargar)),
    );
  };

  const construirFormularioNuevo = () => {
    const inputNombre = el("input", { atributos: { type: "text", maxlength: "80" } });
    const selectorEtiqueta = crearSelectorEtiqueta(
      etiquetasCargadas,
      null,
      textos.funcionarios.sinEtiqueta,
      textos.funcionarios.etiqueta,
    );
    const errorNuevo = el("p", { clase: "fila-admin__error", atributos: { role: "alert" } });
    errorNuevo.hidden = true;

    const agregar = el("button", {
      clase: "boton boton--primario",
      texto: textos.funcionarios.agregar,
      atributos: { type: "submit" },
    });
    const formulario = el(
      "form",
      { clase: "lista-admin__nuevo" },
      el(
        "div",
        { clase: "lista-admin__nuevo-controles" },
        campo(textos.funcionarios.nuevo, inputNombre),
        campo(textos.funcionarios.etiqueta, selectorEtiqueta.raiz),
        agregar,
      ),
      errorNuevo,
    );
    formulario.addEventListener("submit", async (evento) => {
      evento.preventDefault();
      errorNuevo.hidden = true;
      try {
        await api.crearFuncionario(inputNombre.value, selectorEtiqueta.valor());
        inputNombre.value = "";
        selectorEtiqueta.reiniciar();
        await recargar();
        inputNombre.focus();
      } catch (error) {
        errorNuevo.textContent = (error as ErrorUi).mensaje;
        errorNuevo.hidden = false;
      }
    });
    contenedorNuevo.replaceChildren(formulario);
  };

  const recargar = async () => {
    try {
      const [funcionarios, etiquetas] = await Promise.all([
        api.listarFuncionarios(true),
        api.listarEtiquetas(true),
      ]);
      funcionariosCargados = funcionarios;
      etiquetasCargadas = etiquetas;
      estado.funcionarios = funcionarios;
      estado.etiquetas = etiquetas;
      pintarLista();
      construirFormularioNuevo();
      dialogo.mostrarError(null);
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
    }
  };

  buscador.addEventListener("input", pintarLista);
  dialogo.cuerpo.append(buscador, lista, contenedorNuevo);
  void recargar();
  buscador.focus();
}

function filaFuncionario(
  funcionario: Funcionario,
  etiquetas: Etiqueta[],
  recargar: () => Promise<void>,
): HTMLElement {
  const nombre = el("input", {
    atributos: { type: "text", maxlength: "80", "aria-label": textos.funcionarios.nombre },
  });
  nombre.value = funcionario.nombre;
  const selectorEtiqueta = crearSelectorEtiqueta(
    etiquetas,
    funcionario.etiqueta_id,
    textos.funcionarios.sinEtiqueta,
    textos.funcionarios.etiqueta,
  );
  const activo = el("input", { atributos: { type: "checkbox" } });
  activo.checked = funcionario.activo;
  const errorFila = el("p", { clase: "fila-admin__error", atributos: { role: "alert" } });
  errorFila.hidden = true;

  const mostrarErrorFila = (mensaje: string | null) => {
    errorFila.textContent = mensaje ?? "";
    errorFila.hidden = mensaje === null;
  };

  const guardar = boton(textos.funcionarios.guardar, async () => {
    mostrarErrorFila(null);
    try {
      await api.actualizarFuncionario(funcionario.id, {
        nombre: nombre.value,
        etiqueta_id: selectorEtiqueta.valor(),
        activo: activo.checked,
      });
      await recargar();
    } catch (error) {
      mostrarErrorFila((error as ErrorUi).mensaje);
    }
  });

  activo.addEventListener("change", async () => {
    mostrarErrorFila(null);
    try {
      await api.actualizarFuncionario(funcionario.id, { activo: activo.checked });
      await recargar();
    } catch (error) {
      activo.checked = !activo.checked;
      mostrarErrorFila((error as ErrorUi).mensaje);
    }
  });

  return el(
    "div",
    { clase: `fila-admin${funcionario.activo ? "" : " fila-admin--inactivo"}` },
    el(
      "div",
      { clase: "fila-admin__controles fila-admin__controles--funcionario" },
      nombre,
      selectorEtiqueta.raiz,
      el("label", { clase: "casilla" }, activo, el("span", { texto: textos.funcionarios.activo })),
      guardar,
    ),
    errorFila,
  );
}
