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
    columnaBox: "Box",
    manana: "Mañana",
    tarde: "Tarde",
    reservarEn: (box: string, hora: string) => `Reservar ${box} a las ${hora}`,
    sinBoxes: "No hay boxes activos. Agrégalos con el botón «⚙ Boxes».",
    cargando: "Cargando…",
  },
  reserva: {
    titulo: "Reservar box",
    box: "Box",
    fecha: "Fecha",
    horario: "Horario",
    desde: "Desde",
    hasta: "Hasta",
    manana: "Mañana",
    tarde: "Tarde",
    diaCompleto: "Día completo",
    funcionario: "Nombre del funcionario",
    motivo: "Motivo (opcional)",
    repetir: "Repetir cada semana",
    repetirHasta: "hasta el",
    resumenDia: (horas: string) => `Se reservará de ${horas}.`,
    resumenSerie: (horas: string, fechas: string[]) =>
      `Se reservará de ${horas} los ${fechas.length} días: ${fechas.join(", ")}.`,
    rangoInvertido: "La hora de término debe ser posterior a la de inicio.",
    demasiadasFechas: (max: number) => `La serie no puede tener más de ${max} días. Elige una fecha de término más cercana.`,
    algunosOcupados:
      "Algunos de esos bloques ya están ocupados, así que no se reservó nada. Puedes reservar solo los libres o cambiar el horario.",
    soloLibres: "Reservar solo los libres",
    confirmar: "Reservar",
    enviando: "Reservando…",
    funcionarioObligatorio: "Escribe tu nombre para reservar.",
    listo: (dias: number) => (dias > 1 ? `Reserva creada para ${dias} días.` : "Reserva creada."),
    parcial: (omitidos: number) =>
      `Reserva creada. ${omitidos} ${omitidos === 1 ? "bloque ya estaba ocupado" : "bloques ya estaban ocupados"} y se omitieron.`,
  },
  detalle: {
    titulo: "Detalle de la reserva",
    funcionario: "Funcionario",
    horario: "Horario",
    motivo: "Motivo",
    sinMotivo: "—",
    creada: "Reservada el",
    cancelar: "Cancelar reserva",
    cancelarDia: "Cancelar solo este día",
    cancelarSiguientes: "Cancelar este día y los siguientes",
    confirmarCancelacionSerie: (nombre: string, fecha: string) =>
      `¿Cancelar la serie de ${nombre} desde el ${fecha} en adelante? Los días anteriores se conservan.`,
    serieCancelada: (n: number) => `Se cancelaron ${n} bloques de la serie.`,
    confirmarCancelacion: (nombre: string, horas: string) =>
      `¿Cancelar la reserva de ${nombre} (${horas})?`,
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
