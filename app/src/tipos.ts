/** Tipos del contrato, espejo de `shared/src/modelo.rs` y de `comandos.rs`. */

export interface Etiqueta {
  id: number;
  nombre: string;
  /** Color hexadecimal `#rrggbb` en minúsculas. */
  color: string;
  activo: boolean;
}

export interface CambiosEtiqueta {
  nombre?: string;
  color?: string;
  activo?: boolean;
}

export interface Funcionario {
  id: number;
  nombre: string;
  etiqueta_id: number | null;
  activo: boolean;
}

export interface CambiosFuncionario {
  nombre?: string;
  etiqueta_id?: number | null;
  activo?: boolean;
}

export interface BoxAtencion {
  id: number;
  nombre: string;
  activo: boolean;
  orden: number;
  etiqueta_id: number | null;
}

export interface CambiosBox {
  nombre?: string;
  activo?: boolean;
  etiqueta_id?: number | null;
}

export interface Reserva {
  id: number;
  box_id: number;
  /** AAAA-MM-DD */
  fecha: string;
  bloque: number;
  funcionario_id: number | null;
  funcionario: string;
  motivo: string | null;
  creada_en: string;
  /** Reservas creadas juntas (rango o serie semanal) comparten grupo. */
  grupo: number | null;
}

/** Cuerpo de `crear_reservas`: bloques `desde..=hasta` del box en cada fecha. */
export interface NuevoLote {
  box_id: number;
  fechas: string[];
  desde: number;
  hasta: number;
  funcionario_id: number;
  motivo: string | null;
  omitir_ocupados: boolean;
}

export interface ResultadoLote {
  creadas: Reserva[];
  omitidos: Array<{ fecha: string; bloque: number }>;
}

export interface Horario {
  hora_inicio: number;
  minutos_por_bloque: number;
  cantidad_bloques: number;
  /** Primer bloque de la tarde (PM); los anteriores son la mañana (AM). */
  bloque_inicio_tarde: number;
  etiquetas: string[];
}

export interface Preferencias {
  ultimo_funcionario_id: number | null;
  servidor: string | null;
}

export type ResultadoConexion =
  | { tipo: "conectado"; direccion: string; equipo: string }
  | { tipo: "requiere_manual"; mensaje: string };

/** Error que devuelven los comandos: `{ codigo, mensaje }` (ver `errores.rs`). */
export interface ErrorUi {
  codigo: string;
  mensaje: string;
}

export type EstadoConexion =
  | { tipo: "conectado"; equipo: string }
  | { tipo: "buscando" }
  | { tipo: "sin_conexion"; mensaje: string };
