//! API de Gestión de Boxes CESFAM.
//!
//! Se expone como librería para que los tests de integración (`api/tests/`)
//! usen el mismo router y los mismos módulos que el ejecutable.

pub mod arranque;
pub mod config;
pub mod db;
pub mod descubrimiento;
pub mod estado;
pub mod http;
pub mod log;
pub mod repositorio;
pub mod respaldo;
pub mod servicio;
