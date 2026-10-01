/**
 * Utilidades de fechas. Las fechas viajan como texto `AAAA-MM-DD`; aquí se
 * convierten con el constructor local para evitar desfases por zona horaria
 * (`new Date("2026-10-01")` se interpreta en UTC).
 */

function aDate(fecha: string): Date {
  const [anio, mes, dia] = fecha.split("-").map(Number);
  return new Date(anio ?? 1970, (mes ?? 1) - 1, dia ?? 1);
}

function aTexto(fecha: Date): string {
  const mes = String(fecha.getMonth() + 1).padStart(2, "0");
  const dia = String(fecha.getDate()).padStart(2, "0");
  return `${fecha.getFullYear()}-${mes}-${dia}`;
}

/** Hoy en formato `AAAA-MM-DD` (hora local). */
export function hoy(): string {
  return aTexto(new Date());
}

/** Suma `dias` (puede ser negativo) a una fecha `AAAA-MM-DD`. */
export function sumarDias(fecha: string, dias: number): string {
  const resultado = aDate(fecha);
  resultado.setDate(resultado.getDate() + dias);
  return aTexto(resultado);
}

/** Primer día del mes de `fecha`. */
export function inicioDeMes(fecha: string): string {
  return `${fecha.slice(0, 8)}01`;
}

/** Último día del mes de `fecha`. */
export function finDeMes(fecha: string): string {
  const d = aDate(fecha);
  return aTexto(new Date(d.getFullYear(), d.getMonth() + 1, 0));
}

/** "miércoles, 1 de octubre de 2026" con mayúscula inicial. */
export function fechaLarga(fecha: string): string {
  const texto = aDate(fecha).toLocaleDateString("es-CL", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
  return texto.charAt(0).toUpperCase() + texto.slice(1);
}

/** "01-10-2026" para mostrar en modales. */
export function fechaCorta(fecha: string): string {
  const [anio, mes, dia] = fecha.split("-");
  return `${dia}-${mes}-${anio}`;
}

/** "30-09-2026 16:12" a partir de `AAAA-MM-DDTHH:MM:SS`. */
export function fechaHora(marca: string): string {
  const [fecha = "", hora = ""] = marca.split("T");
  return `${fechaCorta(fecha)} ${hora.slice(0, 5)}`;
}

/**
 * Cuántos bloques del día ya terminaron, entre 0 y `cantidad`.
 * Antes de abrir → 0; después de cerrar → `cantidad`.
 */
export function bloquesTerminados(
  horaInicio: number,
  minutosPorBloque: number,
  cantidad: number,
  ahora: Date = new Date(),
): number {
  const minutos = ahora.getHours() * 60 + ahora.getMinutes() - horaInicio * 60;
  if (minutos < 0) return 0;
  return Math.min(Math.floor(minutos / minutosPorBloque), cantidad);
}
