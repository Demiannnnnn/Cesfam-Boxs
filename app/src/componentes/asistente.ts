/**
 * Pantalla de primer inicio para configurar el modo del equipo (RF-16, RF-17, ADR-020).
 * Presenta tres opciones: Servidor, Escritura o Lectura.
 */
import { api } from "../api";
import { el } from "../dom";
import { textos } from "../textos";
import type { ErrorUi, ModoEquipo } from "../tipos";
import { abrirDialogo } from "./dialogo";

interface OpcionModo {
  modo: ModoEquipo;
  titulo: string;
  descripcion: string;
}

/**
 * Muestra el asistente de primer inicio con tres botones grandes para elegir el modo.
 * Guarda la elección a través de `api.elegirModo(modo)` y resuelve con el modo elegido.
 */
export function mostrarAsistente(): Promise<ModoEquipo> {
  return new Promise((resolver) => {
    const dialogo = abrirDialogo(textos.asistente.titulo, undefined, false);

    const opciones: OpcionModo[] = [
      {
        modo: "servidor",
        titulo: textos.asistente.servidorTitulo,
        descripcion: textos.asistente.servidorDescripcion,
      },
      {
        modo: "escritura",
        titulo: textos.asistente.escrituraTitulo,
        descripcion: textos.asistente.escrituraDescripcion,
      },
      {
        modo: "lectura",
        titulo: textos.asistente.lecturaTitulo,
        descripcion: textos.asistente.lecturaDescripcion,
      },
    ];

    const botones: HTMLButtonElement[] = [];

    const mensajeEstado = el("p", {
      clase: "asistente__estado",
      atributos: { role: "status" },
    });
    mensajeEstado.hidden = true;

    const elegir = async (modo: ModoEquipo) => {
      dialogo.mostrarError(null);
      for (const btn of botones) btn.disabled = true;
      try {
        if (modo === "servidor") {
          mensajeEstado.textContent = textos.asistente.instalandoServidor;
          mensajeEstado.hidden = false;
          await api.instalarServidor(null);
        } else {
          await api.elegirModo(modo);
        }
        dialogo.cerrar();
        resolver(modo);
      } catch (error) {
        mensajeEstado.hidden = true;
        dialogo.mostrarError((error as ErrorUi).mensaje);
        for (const btn of botones) btn.disabled = false;
      }
    };

    const contenedor = el(
      "div",
      { clase: "asistente" },
      ...opciones.map(({ modo, titulo, descripcion }) => {
        const btn = el(
          "button",
          {
            clase: "asistente__opcion",
            atributos: { type: "button" },
            alHacerClic: () => void elegir(modo),
          },
          el("span", { clase: "asistente__opcion-titulo", texto: titulo }),
          el("span", { clase: "asistente__opcion-descripcion", texto: descripcion }),
        );
        botones.push(btn);
        return btn;
      }),
    );

    dialogo.cuerpo.append(contenedor, mensajeEstado);
  });
}
