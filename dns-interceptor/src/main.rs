use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use dns_interceptor::api_client::ApiClient;
use dns_interceptor::config::Config;
use dns_interceptor::dns::{DnsMessage, RequestCase};
use dns_interceptor::resolver::{self, Resolution, ResolverState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    tracing::info!(listen_addr = %config.listen_addr, dns_api_url = %config.dns_api_url, "iniciando DNS Interceptor");

    let socket = Arc::new(UdpSocket::bind(config.listen_addr).await?);
    let api = Arc::new(ApiClient::new(config.dns_api_url.clone(), config.api_timeout)?);
    let state = ResolverState::new();

    // Buffer clásico de mensaje DNS por UDP (RFC 1035 recomienda 512 bytes sin EDNS).
    let mut buf = [0u8; 512];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let data = buf[..len].to_vec();

        let socket = Arc::clone(&socket);
        let api = Arc::clone(&api);
        let state = Arc::clone(&state);

        // Cada paquete se procesa en su propia task -> soporta múltiples
        // solicitudes simultáneas sin bloquear el loop principal.
        tokio::spawn(async move {
            if let Err(e) = handle_packet(socket, api, state, data, src).await {
                tracing::error!(error = %e, %src, "error manejando paquete DNS");
            }
        });
    }
}

async fn handle_packet(
    socket: Arc<UdpSocket>,
    api: Arc<ApiClient>,
    state: Arc<ResolverState>,
    data: Vec<u8>,
    src: SocketAddr,
) -> anyhow::Result<()> {
    let msg = match DnsMessage::parse(&data) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(error = %e, %src, "paquete no parseable como DNS, se descarta");
            return Ok(());
        }
    };

    // --- Identificación del tipo de request (Caso 1 vs Caso 2 del enunciado) ---
    let case = msg.case();
    tracing::info!(
        %src,
        %case,
        id = msg.header.id,
        qr = msg.header.qr,
        opcode = msg.header.opcode,
        domain = msg.domain().unwrap_or("<sin-question>"),
        "paquete DNS recibido y clasificado"
    );

    match case {
        RequestCase::Other => {
            // Caso 1: no es query estándar -> forward crudo (BASE64) al DNS API.
            tracing::debug!(%src, "Caso 1: forward crudo al DNS API vía POST /api/dns_resolver");
            let response = api.forward_raw(&data).await?;
            socket.send_to(&response, src).await?;
        }

        RequestCase::StandardQuery => {
            // Caso 2: query estándar -> consultar al DNS API si existe el registro
            // (GET /api/exists) y resolver localmente según el tipo de registro.
            let domain = msg.domain().unwrap_or("<sin-question>").to_string();
            tracing::debug!(%src, %domain, "Caso 2: consultando DNS API (/api/exists)");

            match resolver::resolve(&msg, &api, &state, src.ip()).await? {
                Resolution::Answered(response) => {
                    socket.send_to(&response, src).await?;
                }
                Resolution::Fallback => {
                    // No existe el registro (o quedó unhealthy) -> se trata igual que el Caso 1.
                    tracing::debug!(%src, %domain, "sin registro en DNS API, cae a forward crudo");
                    let response = api.forward_raw(&data).await?;
                    socket.send_to(&response, src).await?;
                }
            }
        }
    }

    Ok(())
}
