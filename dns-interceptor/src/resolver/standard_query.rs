//! Lógica de resolución para queries estándar (QR=0, OPCODE=0).
//! Dado un DnsMessage ya parseado, decide qué IP(s) devolver según el tipo
//! de registro que reporte el DNS API, o cae al forward crudo si no hay
//! registro / la IP asignada está unhealthy.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api_client::{ApiClient, IpEntry, RecordType};
use crate::dns::DnsMessage;
use crate::resolver::selection::{pick_geo, pick_round_trip};

/// Estado compartido entre requests, necesario para round-robin (hay que
/// recordar cuál fue la última IP servida por dominio).
#[derive(Default)]
pub struct ResolverState {
    round_robin_counters: Mutex<HashMap<String, AtomicUsize>>,
}

impl ResolverState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    async fn next_round_robin(&self, domain: &str, len: usize) -> usize {
        let mut map = self.round_robin_counters.lock().await;
        let counter = map
            .entry(domain.to_string())
            .or_insert_with(|| AtomicUsize::new(0));
        counter.fetch_add(1, Ordering::Relaxed) % len
    }
}

/// Resultado de intentar resolver localmente un query estándar.
pub enum Resolution {
    /// Se encontró registro y se armó la respuesta DNS lista para enviar.
    Answered(Vec<u8>),
    /// No hay registro (o quedó unhealthy) -> el caller debe hacer forward_raw.
    Fallback,
}

pub async fn resolve(
    msg: &DnsMessage,
    api: &ApiClient,
    state: &ResolverState,
    client_ip: IpAddr,
) -> anyhow::Result<Resolution> {
    let domain = match msg.domain() {
        Some(d) => d,
        None => return Ok(Resolution::Fallback),
    };

    let record = match api.exists(domain, client_ip).await? {
        Some(r) => r,
        None => return Ok(Resolution::Fallback),
    };

    let healthy_ips: Vec<&IpEntry> = record.ips.iter().filter(|e| e.healthy).collect();
    if healthy_ips.is_empty() {
        // Todas unhealthy -> se trata como si no existiera el registro.
        return Ok(Resolution::Fallback);
    }

    let chosen_ip: Ipv4Addr = match record.record_type {
        RecordType::Single => healthy_ips[0].ip,

        RecordType::Multi => {
            let idx = state.next_round_robin(domain, healthy_ips.len()).await;
            healthy_ips[idx].ip
        }

        RecordType::Weight => pick_weighted(&healthy_ips),

        RecordType::Geo => match pick_geo(&record.ips, record.client_country.as_deref()) {
            Some(ip) => ip,
            None => return Ok(Resolution::Fallback),
        },

        RecordType::RoundTrip => {
            let Some(client_location) = record.client_location else {
                return Ok(Resolution::Fallback);
            };
            let selected = pick_round_trip(
                client_location,
                &record.health_checkers,
                &record.measurements,
            );
            match selected.filter(|ip| healthy_ips.iter().any(|entry| entry.ip == *ip)) {
                Some(ip) => ip,
                None => return Ok(Resolution::Fallback),
            }
        }
    };

    let response = msg.build_response(&[chosen_ip], record.ttl)?;
    Ok(Resolution::Answered(response))
}

fn pick_weighted(ips: &[&IpEntry]) -> Ipv4Addr {
    use rand::Rng;

    let total_weight: u32 = ips.iter().map(|e| e.weight.unwrap_or(1)).sum();
    if total_weight == 0 {
        return ips[0].ip;
    }

    let mut roll = rand::thread_rng().gen_range(0..total_weight);
    for entry in ips {
        let w = entry.weight.unwrap_or(1);
        if roll < w {
            return entry.ip;
        }
        roll -= w;
    }

    ips[0].ip
}
