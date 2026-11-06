/**
 * Opciones avanzadas protegidas por clave (RF-19, ADR-022).
 *
 * Permite cambiar de modo entre escritura y lectura, y en caso de falla
 * irrecuperable del servidor anterior, promover un equipo de escritura a servidor.
 */
import { api } from "../api";
import { boton, campo, el } from "../dom";
import { estado } from "../estado";
import { textos } from "../textos";
import type { ErrorUi, EstadoCopias, ModoEquipo } from "../tipos";
import { abrirDialogo } from "./dialogo";

export interface AccionesAvanzado {
  alCambiarModo: (modo: ModoEquipo) => void;
  alPromoverAServidor: () => Promise<void>;
}

/**
 * Abre el diálogo de opciones avanzadas. Primero solicita la clave; si es correcta,
 * presenta el panel de configuración según el modo del equipo.
 */
export function abrirAvanzado(acciones: AccionesAvanzado): void {
  pedirClave(acciones);
}

function pedirClave(acciones: AccionesAvanzado): void {
  const dialogo = abrirDialogo(textos.avanzado.titulo);

  const inputClave = el("input", {
    clase: "control",
    atributos: {
      type: "password",
      placeholder: textos.avanzado.clavePlaceholder,
      autofocus: "true",
      autocomplete: "current-password",
    },
  });

  const formulario = el(
    "form",
    { clase: "formulario" },
    el("p", { texto: textos.avanzado.pedirClave }),
    campo(textos.avanzado.clavePlaceholder, inputClave),
  );

  const enviar = boton(
    textos.avanzado.entrar,
    () => void procesarClave(),
    "primario",
  );

  formulario.addEventListener("submit", (evento) => {
    evento.preventDefault();
    void procesarClave();
  });

  const procesarClave = async () => {
    dialogo.mostrarError(null);
    enviar.disabled = true;
    try {
      const correcta = await api.abrirAvanzado(inputClave.value);
      if (!correcta) {
        dialogo.mostrarError(textos.avanzado.claveIncorrecta);
        enviar.disabled = false;
        inputClave.select();
        return;
      }
      dialogo.cerrar();
      void mostrarOpciones(acciones);
    } catch (error) {
      dialogo.mostrarError((error as ErrorUi).mensaje);
      enviar.disabled = false;
    }
  };

  dialogo.cuerpo.append(formulario);
  dialogo.pie.prepend(enviar);
}

async function mostrarOpciones(acciones: AccionesAvanzado): Promise<void> {
  const dialogo = abrirDialogo(textos.avanzado.titulo);

  let copias: EstadoCopias | null = null;
  try {
    copias = await api.estadoCopias();
  } catch {
    copias = { ultima: null, cantidad: 0 };
  }

  const contenedor = el("div", { clase: "formulario" });
  dialogo.cuerpo.append(contenedor);

  const renderizarOpciones = () => {
    contenedor.replaceChildren();

    // 1. Estado de copias locales
    const seccionCopias = el("div", { clase: "campo" });
    const tituloCopias = el("h3", {
      clase: "campo__etiqueta",
      texto: textos.avanzado.seccionCopias,
    });
    const textoCopias = el("p", {
      texto:
        copias && copias.cantidad > 0 && copias.ultima
          ? textos.avanzado.resumenCopias(copias.ultima, copias.cantidad)
          : textos.avanzado.sinCopias,
    });
    seccionCopias.append(tituloCopias, textoCopias);
    contenedor.append(seccionCopias);

    // 2. Opciones según el modo del equipo
    const modo = estado.modo;

    if (modo === "servidor") {
      const avisoServidor = el("p", { texto: textos.avanzado.esServidor });
      contenedor.append(avisoServidor);
      return;
    }

    if (modo === "lectura") {
      const btnCambiarAEscritura = boton(
        textos.avanzado.cambiarAEscritura,
        async () => {
          dialogo.mostrarError(null);
          btnCambiarAEscritura.disabled = true;
          try {
            await api.cambiarModo("escritura");
            acciones.alCambiarModo("escritura");
            renderizarOpciones();
          } catch (error) {
            dialogo.mostrarError((error as ErrorUi).mensaje);
            btnCambiarAEscritura.disabled = false;
          }
        },
        "secundario",
      );
      contenedor.append(btnCambiarAEscritura);
      return;
    }

    if (modo === "escritura") {
      // Botón para cambiar a solo consulta
      const btnCambiarAConsulta = boton(
        textos.avanzado.cambiarAConsulta,
        async () => {
          dialogo.mostrarError(null);
          btnCambiarAConsulta.disabled = true;
          try {
            await api.cambiarModo("lectura");
            acciones.alCambiarModo("lectura");
            renderizarOpciones();
          } catch (error) {
            dialogo.mostrarError((error as ErrorUi).mensaje);
            btnCambiarAConsulta.disabled = false;
          }
        },
        "secundario",
      );

      // Sección Usar este equipo como servidor
      const seccionPromover = el("div", { clase: "formulario" });
      const tituloPromover = el("h3", {
        clase: "campo__etiqueta",
        texto: textos.avanzado.promoverTitulo,
      });
      const advertenciaPromover = el("p", {
        texto: textos.avanzado.promoverTexto,
      });

      const inputConfirmar = el("input", {
        clase: "control",
        atributos: {
          type: "text",
          placeholder: textos.avanzado.promoverPlaceholder,
          autocomplete: "off",
        },
      });

      const btnConfirmar = boton(
        textos.avanzado.confirmar,
        () => void ejecutarPromocion(),
        "peligro",
      );
      btnConfirmar.disabled = true;

      inputConfirmar.addEventListener("input", () => {
        btnConfirmar.disabled = inputConfirmar.value !== "SERVIDOR";
      });

      const mensajeEstado = el("p", {
        clase: "asistente__estado",
        atributos: { role: "status" },
      });
      mensajeEstado.hidden = true;

      const ejecutarPromocion = async () => {
        dialogo.mostrarError(null);
        btnConfirmar.disabled = true;
        inputConfirmar.disabled = true;
        btnCambiarAConsulta.disabled = true;

        mensajeEstado.textContent = textos.avanzado.buscandoServidores;
        mensajeEstado.hidden = false;

        const temporizador = window.setTimeout(() => {
          mensajeEstado.textContent = textos.avanzado.instalandoServidor;
        }, 3000);

        try {
          await api.promoverAServidor();
          window.clearTimeout(temporizador);
          dialogo.cerrar();
          await acciones.alPromoverAServidor();
        } catch (error) {
          window.clearTimeout(temporizador);
          mensajeEstado.hidden = true;
          dialogo.mostrarError((error as ErrorUi).mensaje);
          btnCambiarAConsulta.disabled = false;
          inputConfirmar.disabled = false;
          btnConfirmar.disabled = inputConfirmar.value !== "SERVIDOR";
        }
      };

      seccionPromover.append(
        tituloPromover,
        advertenciaPromover,
        campo(textos.avanzado.promoverPlaceholder, inputConfirmar),
        btnConfirmar,
        mensajeEstado,
      );

      contenedor.append(btnCambiarAConsulta, seccionPromover);
    }
  };

  renderizarOpciones();
}
