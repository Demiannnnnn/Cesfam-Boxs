//! Tipos compartidos entre `gestion-box-api` y la app Tauri.
//!
//! Mantener el contrato en un solo crate garantiza que API y app no se
//! desincronicen: un cambio incompatible rompe la compilación de ambos.

pub mod descubrimiento;
pub mod error;
pub mod horario;
pub mod modelo;
pub mod tipos;

pub use descubrimiento::RespuestaDescubrimiento;
pub use error::{CodigoError, ErrorRespuesta};
pub use modelo::{
    BloqueOmitido, BoxAtencion, CambiosBox, CambiosEtiqueta, CambiosFuncionario,
    ConsultaBoxesLibres, ConsultaBuscarReservas, EstadoSalud, Etiqueta, Funcionario, NuevaEtiqueta,
    NuevaReserva, NuevoBox, NuevoFuncionario, NuevoLote, Reserva, ResultadoCancelacion,
    ResultadoLote,
};
pub use tipos::{Bloque, Color, ErrorTipo, Fecha};
