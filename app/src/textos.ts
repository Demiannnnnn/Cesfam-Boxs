/**
 * Catálogo único de textos visibles (RNF-11). Ningún otro archivo debe tener
 * texto para el usuario escrito a mano.
 */
export const textos = {
  conexion: {
    conectado: (equipo: string) => `Conectado a ${equipo}`,
    buscando: "Buscando servidor…",
    sinConexion: "Sin conexión",
  },
  grilla: {
    columnaHora: "Hora",
    reservar: "Reservar",
    sinBoxes: "No hay boxes activos. Agrégalos con el botón «⚙ Boxes».",
    cargando: "Cargando…",
  },
  reserva: {
    titulo: "Reservar box",
    box: "Box",
    fecha: "Fecha",
    hora: "Hora",
    funcionario: "Nombre del funcionario",
    motivo: "Motivo (opcional)",
    confirmar: "Reservar",
    enviando: "Reservando…",
    funcionarioObligatorio: "Escribe tu nombre para reservar.",
  },
  detalle: {
    titulo: "Detalle de la reserva",
    funcionario: "Funcionario",
    motivo: "Motivo",
    sinMotivo: "—",
    creada: "Reservada el",
    cancelar: "Cancelar reserva",
    confirmarCancelacion: (nombre: string, hora: string) =>
      `¿Cancelar la reserva de ${nombre} a las ${hora}?`,
    siCancelar: "Sí, cancelar",
    yaCancelada: "Esa reserva ya había sido cancelada.",
    cancelada: "Reserva cancelada.",
  },
  boxes: {
    titulo: "Configuración de boxes",
    nombre: "Nombre",
    activo: "Activo",
    guardar: "Guardar nombre",
    nuevo: "Nuevo box",
    agregar: "Agregar",
    vacio: "Todavía no hay boxes.",
    confirmarDesactivar: (nombre: string) =>
      `Al desactivar «${nombre}» desaparecerá de la grilla. Sus reservas se conservan y vuelven a verse si lo reactivas. ¿Continuar?`,
    siDesactivar: "Sí, desactivar",
  },
  conexionManual: {
    titulo: "Conectar con el servidor",
    explicacion:
      "No se encontró el servidor automáticamente. Escribe el nombre del PC servidor (por ejemplo PC-SOME) o su dirección IP.",
    campo: "Nombre del PC servidor o IP",
    conectar: "Conectar",
    buscar: "Buscar de nuevo",
    buscando: "Buscando servidor…",
  },
  exportar: {
    titulo: "Exportar reservas",
    desde: "Desde",
    hasta: "Hasta",
    confirmar: "Exportar a CSV",
    filtro: "Planilla CSV",
    listo: (filas: number) => `Exportación lista: ${filas} reservas.`,
  },
  comunes: {
    cerrar: "Cerrar",
    volver: "Volver",
    errorDesconocido: "Ocurrió un error inesperado. Si se repite, avisa a TI y menciona la hora.",
  },
} as const;
