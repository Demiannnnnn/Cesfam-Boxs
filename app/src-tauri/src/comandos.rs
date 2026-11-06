//! Comandos `invoke` expuestos a la interfaz. Cada comando es delgado: obtiene el
//! cliente, llama a la API y traduce el error a [`ErrorUi`].

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use gestion_box_shared::horario::Horario;
use gestion_box_shared::{
    Bloque, BoxAtencion, CambiosBox, CambiosEtiqueta, CambiosFuncionario, Color,
    ConsultaBoxesLibres, ConsultaBuscarReservas, EstadoSalud, Etiqueta, Fecha, Funcionario,
    NuevaEtiqueta, NuevaReserva, NuevoBox, NuevoFuncionario, NuevoLote, Reserva,
    ResultadoCancelacion, ResultadoLote,
};
use serde::Serialize;
use tauri::State;

use crate::cliente_api::{host_de, normalizar_direccion, ClienteApi, ErrorCliente};
use crate::conexion::{self, Servidor};
use crate::config_local::{ConfigLocal, ModoEquipo};
use crate::errores::{error_para_ui, ErrorUi, MENSAJE_NO_ENCONTRADO_EN_RED};
use crate::exportar;

type Resultado<T> = Result<T, ErrorUi>;

/// Estado de la app compartido entre comandos y la tarea de respaldo.
#[derive(Clone)]
pub struct EstadoApp {
    ruta_config: PathBuf,
    config: Arc<Mutex<ConfigLocal>>,
    cliente: Arc<Mutex<Option<ClienteApi>>>,
    notificador_copia: Arc<tokio::sync::Notify>,
    en_descarga: Arc<AtomicBool>,
}

impl EstadoApp {
    /// Carga `cliente.json` (o valores por defecto).
    pub fn cargar(ruta_config: PathBuf) -> Self {
        let config = ConfigLocal::cargar(&ruta_config);
        Self {
            ruta_config,
            config: Arc::new(Mutex::new(config)),
            cliente: Arc::new(Mutex::new(None)),
            notificador_copia: Arc::new(tokio::sync::Notify::new()),
            en_descarga: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn config(&self) -> ConfigLocal {
        self.config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn modificar_config(&self, cambio: impl FnOnce(&mut ConfigLocal)) {
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let antes = config.clone();
        cambio(&mut config);
        if *config != antes {
            if let Err(error) = config.guardar(&self.ruta_config) {
                eprintln!("[gestion-box] no se pudo guardar cliente.json: {error}");
            }
        }
    }

    fn fijar_modo(&self, modo: ModoEquipo) -> Resultado<()> {
        let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        config
            .fijar_modo_inicial(modo)
            .map_err(ErrorUi::datos_invalidos)?;
        if let Err(error) = config.guardar(&self.ruta_config) {
            eprintln!("[gestion-box] no se pudo guardar cliente.json: {error}");
            return Err(ErrorUi::new(
                "interno",
                "No se pudo guardar la configuración del equipo.",
            ));
        }
        Ok(())
    }

    fn cliente(&self) -> Resultado<ClienteApi> {
        self.cliente
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or_else(ErrorUi::sin_servidor)
    }

    /// Fija el servidor actual y lo recuerda (RF-10). Notifica a la tarea de respaldo.
    fn usar_servidor(&self, servidor: &Servidor) -> Resultado<()> {
        let cliente =
            ClienteApi::new(&servidor.direccion).map_err(|e| error_para_ui(&e, None, None))?;
        *self.cliente.lock().unwrap_or_else(|e| e.into_inner()) = Some(cliente);
        self.modificar_config(|config| {
            config.servidor = Some(servidor.direccion.clone());
            config.equipo_servidor = Some(servidor.equipo.clone());
        });
        self.notificador_copia.notify_one();
        Ok(())
    }

    /// Obtiene el cliente de la API si ya está conectado.
    pub fn obtener_cliente(&self) -> Option<ClienteApi> {
        self.cliente
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Carpeta donde este equipo almacena sus copias de respaldo (RF-18, ADR-021).
    pub fn carpeta_copias(&self) -> PathBuf {
        let dir = self.ruta_config.parent().unwrap_or(Path::new("."));
        crate::copia_diaria::carpeta_copias(dir)
    }

    /// Notificador para despertar la tarea de respaldo al conectar.
    pub fn notificador_copia(&self) -> Arc<tokio::sync::Notify> {
        self.notificador_copia.clone()
    }

    /// Bandera atómica que evita descargas simultáneas.
    pub fn en_descarga(&self) -> Arc<AtomicBool> {
        self.en_descarga.clone()
    }

    /// Traduce un error de red usando el nombre del servidor conocido.
    fn error(&self, error: ErrorCliente) -> ErrorUi {
        let config = self.config();
        let host = config.servidor.as_deref().map(host_de);
        error_para_ui(&error, config.equipo_servidor.as_deref().or(host), host)
    }
}

/// Resultado de un intento de conexión.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum ResultadoConexion {
    Conectado { direccion: String, equipo: String },
    RequiereManual { mensaje: String },
}

impl From<Servidor> for ResultadoConexion {
    fn from(servidor: Servidor) -> Self {
        Self::Conectado {
            direccion: servidor.direccion,
            equipo: servidor.equipo,
        }
    }
}

/// Preferencias locales que la interfaz necesita al iniciar (RF-10, RF-13).
#[derive(Debug, Clone, Serialize)]
pub struct Preferencias {
    pub ultimo_funcionario_id: Option<i64>,
    pub servidor: Option<String>,
}

/// Horario de bloques (fuente única en `shared::horario`). Cubre RF-01.
#[tauri::command]
pub fn horario() -> Horario {
    Horario::actual()
}

/// Preferencias guardadas en este PC. Cubre RF-10.
#[tauri::command]
pub fn preferencias(estado: State<'_, EstadoApp>) -> Preferencias {
    let config = estado.config();
    Preferencias {
        ultimo_funcionario_id: config.ultimo_funcionario_id,
        servidor: config.servidor,
    }
}

/// Modo de operación configurado en este equipo, o `None` en el primer inicio. Cubre RF-16 y RF-17.
#[tauri::command]
pub fn modo_equipo(estado: State<'_, EstadoApp>) -> Option<ModoEquipo> {
    estado.config().modo
}

/// Fija el modo de operación en el primer inicio. Falla si ya tiene un modo (RF-16).
#[tauri::command]
pub fn elegir_modo(estado: State<'_, EstadoApp>, modo: ModoEquipo) -> Resultado<()> {
    estado.fijar_modo(modo)
}

/// Consulta el estado de las copias locales de respaldo (RF-18, ADR-021).
#[tauri::command]
pub fn estado_copias(estado: State<'_, EstadoApp>) -> crate::copia_diaria::EstadoCopias {
    crate::copia_diaria::estado_copias(&estado.carpeta_copias())
}

/// Descubre el servidor: guardada → localhost → broadcast (RF-09, RNF-08).
#[tauri::command]
pub async fn conectar(estado: State<'_, EstadoApp>) -> Resultado<ResultadoConexion> {
    let guardada = estado.config().servidor;
    let estrategias = conexion::estrategias_por_defecto(guardada.as_deref());
    match conexion::descubrir(&estrategias).await {
        Some(servidor) => {
            estado.usar_servidor(&servidor)?;
            Ok(servidor.into())
        }
        None => Ok(ResultadoConexion::RequiereManual {
            mensaje: MENSAJE_NO_ENCONTRADO_EN_RED.to_owned(),
        }),
    }
}

/// Conecta a la dirección escrita por el usuario (nombre del PC o IP). Cubre RF-09.
#[tauri::command]
pub async fn conectar_manual(
    estado: State<'_, EstadoApp>,
    direccion: String,
) -> Resultado<ResultadoConexion> {
    let normalizada = normalizar_direccion(&direccion).ok_or_else(|| {
        ErrorUi::datos_invalidos("Escribe el nombre del PC servidor o su dirección IP.")
    })?;
    let cliente = ClienteApi::new(&normalizada).map_err(|e| error_para_ui(&e, None, None))?;
    let host = host_de(&normalizada);
    let salud = cliente
        .salud()
        .await
        .map_err(|e| error_para_ui(&e, Some(host), Some(host)))?;
    let servidor = Servidor {
        direccion: normalizada,
        equipo: salud.equipo,
    };
    estado.usar_servidor(&servidor)?;
    Ok(servidor.into())
}

/// `GET /health` del servidor actual (RF-08).
#[tauri::command]
pub async fn estado_salud(estado: State<'_, EstadoApp>) -> Resultado<EstadoSalud> {
    estado.cliente()?.salud().await.map_err(|e| estado.error(e))
}

/// Lista los boxes de atención. Cubre RF-01, RF-06 y RF-14.
#[tauri::command]
pub async fn listar_boxes(
    estado: State<'_, EstadoApp>,
    incluir_inactivos: bool,
) -> Resultado<Vec<BoxAtencion>> {
    estado
        .cliente()?
        .listar_boxes(incluir_inactivos)
        .await
        .map_err(|e| estado.error(e))
}

/// Crea un nuevo box con etiqueta opcional. Cubre RF-06 y RF-14.
#[tauri::command]
pub async fn crear_box(
    estado: State<'_, EstadoApp>,
    nombre: String,
    etiqueta_id: Option<i64>,
) -> Resultado<BoxAtencion> {
    estado
        .cliente()?
        .crear_box(&NuevoBox {
            nombre,
            etiqueta_id,
        })
        .await
        .map_err(|e| estado.error(e))
}

/// Actualiza nombre, etiqueta o estado activo de un box. Cubre RF-06 y RF-14.
#[tauri::command]
pub async fn actualizar_box(
    estado: State<'_, EstadoApp>,
    id: i64,
    cambios: CambiosBox,
) -> Resultado<BoxAtencion> {
    estado
        .cliente()?
        .actualizar_box(id, &cambios)
        .await
        .map_err(|e| estado.error(e))
}

/// Lista las etiquetas registradas. Cubre RF-14.
#[tauri::command]
pub async fn listar_etiquetas(
    estado: State<'_, EstadoApp>,
    incluir_inactivos: bool,
) -> Resultado<Vec<Etiqueta>> {
    estado
        .cliente()?
        .listar_etiquetas(incluir_inactivos)
        .await
        .map_err(|e| estado.error(e))
}

fn parsear_color(texto: &str) -> Resultado<Color> {
    texto
        .parse()
        .map_err(|_| ErrorUi::datos_invalidos("Elige un color válido en formato #RRGGBB."))
}

/// Crea una nueva etiqueta con nombre y color. Cubre RF-14.
#[tauri::command]
pub async fn crear_etiqueta(
    estado: State<'_, EstadoApp>,
    nombre: String,
    color: String,
) -> Resultado<Etiqueta> {
    let color = parsear_color(&color)?;
    estado
        .cliente()?
        .crear_etiqueta(&NuevaEtiqueta { nombre, color })
        .await
        .map_err(|e| estado.error(e))
}

/// Actualiza nombre, color o estado activo de una etiqueta. Cubre RF-14.
#[tauri::command]
pub async fn actualizar_etiqueta(
    estado: State<'_, EstadoApp>,
    id: i64,
    cambios: CambiosEtiqueta,
) -> Resultado<Etiqueta> {
    estado
        .cliente()?
        .actualizar_etiqueta(id, &cambios)
        .await
        .map_err(|e| estado.error(e))
}

/// Lista los funcionarios registrados. Cubre RF-13 y RF-14.
#[tauri::command]
pub async fn listar_funcionarios(
    estado: State<'_, EstadoApp>,
    incluir_inactivos: bool,
) -> Resultado<Vec<Funcionario>> {
    estado
        .cliente()?
        .listar_funcionarios(incluir_inactivos)
        .await
        .map_err(|e| estado.error(e))
}

/// Registra un nuevo funcionario con etiqueta opcional. Cubre RF-13 y RF-14.
#[tauri::command]
pub async fn crear_funcionario(
    estado: State<'_, EstadoApp>,
    nombre: String,
    etiqueta_id: Option<i64>,
) -> Resultado<Funcionario> {
    estado
        .cliente()?
        .crear_funcionario(&NuevoFuncionario {
            nombre,
            etiqueta_id,
        })
        .await
        .map_err(|e| estado.error(e))
}

/// Actualiza nombre, etiqueta o estado activo de un funcionario. Cubre RF-13 y RF-14.
#[tauri::command]
pub async fn actualizar_funcionario(
    estado: State<'_, EstadoApp>,
    id: i64,
    cambios: CambiosFuncionario,
) -> Resultado<Funcionario> {
    estado
        .cliente()?
        .actualizar_funcionario(id, &cambios)
        .await
        .map_err(|e| estado.error(e))
}

fn parsear_fecha(texto: &str) -> Resultado<Fecha> {
    texto
        .parse()
        .map_err(|_| ErrorUi::datos_invalidos(format!("La fecha \"{texto}\" no es válida.")))
}

fn parsear_bloque(valor: u8) -> Resultado<Bloque> {
    Bloque::new(valor).map_err(|e| ErrorUi::datos_invalidos(e.to_string()))
}

/// Lista los boxes activos libres en `desde..=hasta` de `fecha`, con filtro opcional
/// de etiqueta del box. Cubre RF-15.
#[tauri::command]
pub async fn boxes_libres(
    estado: State<'_, EstadoApp>,
    fecha: String,
    desde: u8,
    hasta: u8,
    etiqueta_id: Option<i64>,
) -> Resultado<Vec<BoxAtencion>> {
    let consulta = ConsultaBoxesLibres {
        fecha: parsear_fecha(&fecha)?,
        desde: parsear_bloque(desde)?,
        hasta: parsear_bloque(hasta)?,
        etiqueta_id,
    };
    estado
        .cliente()?
        .boxes_libres(&consulta)
        .await
        .map_err(|e| estado.error(e))
}

/// Busca reservas en `desde..=hasta` filtrando por funcionario, etiqueta y/o box. Cubre RF-15.
#[tauri::command]
pub async fn buscar_reservas(
    estado: State<'_, EstadoApp>,
    desde: String,
    hasta: String,
    funcionario_id: Option<i64>,
    etiqueta_id: Option<i64>,
    box_id: Option<i64>,
) -> Resultado<Vec<Reserva>> {
    let consulta = ConsultaBuscarReservas {
        desde: parsear_fecha(&desde)?,
        hasta: parsear_fecha(&hasta)?,
        funcionario_id,
        etiqueta_id,
        box_id,
    };
    estado
        .cliente()?
        .buscar_reservas(&consulta)
        .await
        .map_err(|e| estado.error(e))
}

/// Lista las reservas de un día. Cubre RF-01.
#[tauri::command]
pub async fn listar_reservas(
    estado: State<'_, EstadoApp>,
    fecha: String,
) -> Resultado<Vec<Reserva>> {
    let fecha = parsear_fecha(&fecha)?;
    estado
        .cliente()?
        .reservas_del_dia(fecha)
        .await
        .map_err(|e| estado.error(e))
}

/// Reservas de un rango de fechas (vista semanal). Cubre RF-01.
#[tauri::command]
pub async fn listar_reservas_rango(
    estado: State<'_, EstadoApp>,
    desde: String,
    hasta: String,
) -> Resultado<Vec<Reserva>> {
    let (desde, hasta) = (parsear_fecha(&desde)?, parsear_fecha(&hasta)?);
    estado
        .cliente()?
        .reservas_del_rango(desde, hasta)
        .await
        .map_err(|e| estado.error(e))
}

/// Crea una reserva y recuerda el identificador del funcionario en este PC. Cubre RF-03 y RF-13.
#[tauri::command]
pub async fn crear_reserva(
    estado: State<'_, EstadoApp>,
    nueva: NuevaReserva,
) -> Resultado<Reserva> {
    let reserva = estado
        .cliente()?
        .crear_reserva(&nueva)
        .await
        .map_err(|e| estado.error(e))?;
    estado.modificar_config(|config| {
        config.ultimo_funcionario_id = Some(nueva.funcionario_id);
    });
    Ok(reserva)
}

/// Crea un rango de bloques en una o varias fechas (serie semanal) y recuerda el
/// funcionario en este PC. Cubre RF-03, RF-05 y RF-13.
#[tauri::command]
pub async fn crear_reservas(
    estado: State<'_, EstadoApp>,
    lote: NuevoLote,
) -> Resultado<ResultadoLote> {
    let resultado = estado
        .cliente()?
        .crear_lote(&lote)
        .await
        .map_err(|e| estado.error(e))?;
    if !resultado.creadas.is_empty() {
        estado.modificar_config(|config| config.ultimo_funcionario_id = Some(lote.funcionario_id));
    }
    Ok(resultado)
}

/// Cancela las reservas de un grupo desde `desde` (este día y los siguientes de la serie). Cubre RF-04.
#[tauri::command]
pub async fn cancelar_grupo(
    estado: State<'_, EstadoApp>,
    grupo: i64,
    desde: String,
) -> Resultado<ResultadoCancelacion> {
    let desde = parsear_fecha(&desde)?;
    estado
        .cliente()?
        .cancelar_grupo(grupo, desde)
        .await
        .map_err(|e| estado.error(e))
}

/// Cancela una reserva individual. Cubre RF-04.
#[tauri::command]
pub async fn cancelar_reserva(estado: State<'_, EstadoApp>, id: i64) -> Resultado<()> {
    estado
        .cliente()?
        .cancelar_reserva(id)
        .await
        .map_err(|e| estado.error(e))
}

/// Exporta las reservas del rango a un CSV en `ruta`. Devuelve cuántas filas escribió (RF-12).
#[tauri::command]
pub async fn exportar_csv(
    estado: State<'_, EstadoApp>,
    desde: String,
    hasta: String,
    ruta: String,
) -> Resultado<usize> {
    let (desde, hasta) = (parsear_fecha(&desde)?, parsear_fecha(&hasta)?);
    let cliente = estado.cliente()?;
    let reservas = cliente
        .reservas_del_rango(desde, hasta)
        .await
        .map_err(|e| estado.error(e))?;
    let boxes = cliente
        .listar_boxes(true)
        .await
        .map_err(|e| estado.error(e))?;
    let funcionarios = cliente.listar_funcionarios(true).await.unwrap_or_default();
    let etiquetas = cliente.listar_etiquetas(true).await.unwrap_or_default();
    let csv = exportar::generar_csv(&reservas, &boxes, &funcionarios, &etiquetas);
    tokio::fs::write(&ruta, csv).await.map_err(|error| {
        eprintln!("[gestion-box] no se pudo escribir {ruta}: {error}");
        ErrorUi::new(
            "interno",
            "No se pudo guardar el archivo. Revisa que la carpeta exista y que el archivo no esté abierto en Excel.",
        )
    })?;
    Ok(reservas.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsear_color_acepta_hex_y_rechaza_invalido() {
        assert_eq!(parsear_color("#2563EB").unwrap().as_str(), "#2563eb");

        let error = parsear_color("azul").unwrap_err();
        assert_eq!(error.codigo, "datos_invalidos");
        assert!(error.mensaje.contains("#RRGGBB"));
    }

    #[test]
    fn parsear_fecha_rechaza_formato_invalido() {
        assert!(parsear_fecha("2026-10-01").is_ok());

        let error = parsear_fecha("01-10-2026").unwrap_err();
        assert_eq!(error.codigo, "datos_invalidos");
    }

    #[test]
    fn parsear_bloque_rechaza_fuera_de_rango() {
        assert!(parsear_bloque(0).is_ok());
        assert!(parsear_bloque(23).is_ok());

        let error = parsear_bloque(24).unwrap_err();
        assert_eq!(error.codigo, "datos_invalidos");
    }

    #[test]
    fn elegir_modo_funciona_con_none_y_falla_si_ya_existe() {
        let carpeta = tempfile::tempdir().unwrap();
        let ruta = carpeta.path().join("cliente.json");
        let estado = EstadoApp::cargar(ruta);

        assert_eq!(estado.config().modo, None);

        // Primer inicio: fijar modo exitosamente
        assert!(estado.fijar_modo(ModoEquipo::Escritura).is_ok());
        assert_eq!(estado.config().modo, Some(ModoEquipo::Escritura));

        // Intento posterior: rechazar con mensaje en español
        let error = estado.fijar_modo(ModoEquipo::Lectura).unwrap_err();
        assert_eq!(error.codigo, "datos_invalidos");
        assert_eq!(error.mensaje, "El modo del equipo ya fue configurado.");
        assert_eq!(estado.config().modo, Some(ModoEquipo::Escritura));
    }
}
