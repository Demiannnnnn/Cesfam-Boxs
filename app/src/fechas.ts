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

/** `true` si la fecha cae sábado o domingo. */
export function esFinDeSemana(fecha: string): boolean {
  const dia = aDate(fecha).getDay();
  return dia === 0 || dia === 6;
}

/**
 * Día hábil siguiente (`sentido` 1) o anterior (`sentido` -1), saltando sábados
 * y domingos: la planilla del CESFAM solo planifica de lunes a viernes.
 */
export function diaHabil(fecha: string, sentido: 1 | -1): string {
  let resultado = sumarDias(fecha, sentido);
  while (esFinDeSemana(resultado)) resultado = sumarDias(resultado, sentido);
  return resultado;
}

/** Lunes de la semana de `fecha`. */
export function inicioSemana(fecha: string): string {
  const dia = aDate(fecha).getDay(); // 0 = domingo
  return sumarDias(fecha, dia === 0 ? -6 : 1 - dia);
}

/** Lunes a viernes de la semana de `fecha`. */
export function diasHabilesDeSemana(fecha: string): string[] {
  const lunes = inicioSemana(fecha);
  return [0, 1, 2, 3, 4].map((i) => sumarDias(lunes, i));
}

/** "Lun 19" para encabezados de la vista semanal. */
export function diaCorto(fecha: string): string {
  const d = aDate(fecha);
  const nombre = d.toLocaleDateString("es-CL", { weekday: "short" }).replace(".", "");
  return `${nombre.charAt(0).toUpperCase()}${nombre.slice(1)} ${d.getDate()}`;
}

/** "Semana del 19 al 23 de octubre de 2026" (o "del 29 de septiembre al 3 de octubre…"). */
export function tituloSemana(fecha: string): string {
  const dias = diasHabilesDeSemana(fecha);
  const inicio = aDate(dias[0] ?? fecha);
  const fin = aDate(dias[4] ?? fecha);
  const mes = (d: Date) => d.toLocaleDateString("es-CL", { month: "long" });
  const desde = inicio.getMonth() === fin.getMonth() ? `${inicio.getDate()}` : `${inicio.getDate()} de ${mes(inicio)}`;
  return `Semana del ${desde} al ${fin.getDate()} de ${mes(fin)} de ${fin.getFullYear()}`;
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
