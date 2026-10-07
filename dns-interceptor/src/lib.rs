//! Se expone como librería (además del binario en main.rs) para poder
//! testear los módulos desde tests/ (integration tests) sin duplicar código.

pub mod api_client;
pub mod config;
pub mod dns;
pub mod resolver;
