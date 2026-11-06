/**
 * Utilidades de color para etiquetas (RF-14): paleta sugerida y cálculo de color
 * de texto (blanco o negro) según razón de contraste WCAG 2.1.
 */

/**
 * Paleta de 8 colores sugeridos para etiquetas, distinguibles entre sí y con buen
 * contraste sobre el fondo blanco de la interfaz.
 */
export const PALETA_ETIQUETAS: readonly string[] = [
  "#2563eb", // Azul
  "#16a34a", // Verde
  "#dc2626", // Rojo
  "#7c3aed", // Violeta
  "#d97706", // Ámbar
  "#0d9488", // Turquesa
  "#db2777", // Rosa
  "#4f46e5", // Índigo
] as const;

function canalLineal(hexPar: string): number {
  const srgb = Number.parseInt(hexPar, 16) / 255;
  return srgb <= 0.04045 ? srgb / 12.92 : Math.pow((srgb + 0.055) / 1.055, 2.4);
}

/** Luminancia relativa WCAG (0 = negro, 1 = blanco) para un color `#rrggbb`. */
export function luminancia(fondo: string): number {
  const limpio = fondo.trim().replace(/^#/, "");
  if (limpio.length !== 6) return 0;
  const r = canalLineal(limpio.slice(0, 2));
  const g = canalLineal(limpio.slice(2, 4));
  const b = canalLineal(limpio.slice(4, 6));
  if (Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b)) return 0;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Devuelve `"#000000"` o `"#ffffff"` según cuál tenga mayor contraste WCAG contra
 * el color de fondo `#rrggbb`.
 */
export function colorTexto(fondo: string): "#000000" | "#ffffff" {
  // El contraste con negro (L=0) supera al contraste con blanco (L=1) cuando
  // (L + 0.05) / 0.05 > 1.05 / (L + 0.05) => L > sqrt(0.0525) - 0.05 ≈ 0.1791.
  return luminancia(fondo) > 0.179 ? "#000000" : "#ffffff";
}
