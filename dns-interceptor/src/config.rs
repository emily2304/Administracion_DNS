use std::net::SocketAddr;
use std::time::Duration;

/// Configuración de la aplicación, cargada desde variables de entorno.
#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub dns_api_url: String,
    pub api_timeout: Duration,
}

impl Config {
    /// Carga la configuración desde el entorno (y desde un archivo .env si existe).
    pub fn from_env() -> anyhow::Result<Self> {
        // Si hay un .env en el directorio actual, lo carga (no falla si no existe).
        let _ = dotenvy::dotenv();

        let listen_addr = std::env::var("LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:5353".to_string())
            .parse()?;

        let dns_api_url = std::env::var("DNS_API_URL")
            .unwrap_or_else(|_| "http://dns-api:8080".to_string());

        let api_timeout_ms: u64 = std::env::var("API_TIMEOUT_MS")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?;

        Ok(Config {
            listen_addr,
            dns_api_url,
            api_timeout: Duration::from_millis(api_timeout_ms),
        })
    }
}
