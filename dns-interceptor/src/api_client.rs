//! Cliente HTTP hacia el DNS API (contenedor Java).

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use crate::resolver::selection::{ClientLocation, HealthCheckerLocation, RttMeasurement};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    Single,
    Multi,
    #[serde(rename = "round-trip", alias = "roundtrip")]
    RoundTrip,
    Weight,
    Geo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpEntry {
    pub ip: Ipv4Addr,
    /// Usado solo para el tipo "weight". Ignorable en los demás tipos.
    #[serde(default)]
    pub weight: Option<u32>,
    /// Usado solo para el tipo "geo". Ignorable en los demás tipos.
    #[serde(default)]
    pub country: Option<String>,
    /// true si el Health Checker marcó esta IP como saludable.
    #[serde(default = "default_healthy")]
    pub healthy: bool,
}

fn default_healthy() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecord {
    pub record_type: RecordType,
    pub ips: Vec<IpEntry>,
    #[serde(default)]
    pub client_country: Option<String>,
    #[serde(default)]
    pub client_city: Option<String>,
    #[serde(default)]
    pub client_location: Option<ClientLocation>,
    #[serde(default)]
    pub health_checkers: Vec<HealthCheckerLocation>,
    #[serde(default)]
    pub measurements: Vec<RttMeasurement>,
    #[serde(default = "default_ttl", deserialize_with = "deserialize_ttl")]
    pub ttl: u32,
}

fn default_ttl() -> u32 {
    60
}

fn deserialize_ttl<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<u32>::deserialize(deserializer)?.unwrap_or_else(default_ttl))
}

pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String, timeout: Duration) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { http, base_url })
    }

    /// GET /api/exists?domain=<host>&client_ip=<ip>
    /// Devuelve Some(record) si existe un registro para ese host, None si no existe.
    pub async fn exists(
        &self,
        domain: &str,
        client_ip: IpAddr,
    ) -> anyhow::Result<Option<DnsRecord>> {
        let url = format!("{}/api/exists", self.base_url);
        let client_ip = client_ip.to_string();
        let resp = self
            .http
            .get(&url)
            .query(&[("domain", domain), ("client_ip", &client_ip)])
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let resp = resp.error_for_status()?;
        let record: DnsRecord = resp.json().await?;
        Ok(Some(record))
    }

    /// POST /api/dns_resolver
    /// Envía el paquete DNS crudo (codificado en BASE64) y devuelve la respuesta
    /// ya decodificada (bytes DNS listos para reenviar al cliente).
    pub async fn forward_raw(&self, raw_packet: &[u8]) -> anyhow::Result<Vec<u8>> {
        let url = format!("{}/api/dns_resolver", self.base_url);
        let encoded = B64.encode(raw_packet);

        #[derive(serde::Serialize)]
        struct Body<'a> {
            data: &'a str,
        }

        #[derive(Deserialize)]
        struct RespBody {
            data: String,
        }

        let resp = self
            .http
            .post(&url)
            .json(&Body { data: &encoded })
            .send()
            .await?
            .error_for_status()?;

        let body: RespBody = resp.json().await?;
        let decoded = B64.decode(body.data)?;
        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::{DnsRecord, RecordType};
    use std::net::Ipv4Addr;

    #[test]
    fn accepts_project_round_trip_wire_name() {
        assert!(matches!(
            serde_json::from_str::<RecordType>(r#""round-trip""#),
            Ok(RecordType::RoundTrip)
        ));
    }

    #[test]
    fn accepts_legacy_roundtrip_wire_name() {
        assert!(matches!(
            serde_json::from_str::<RecordType>(r#""roundtrip""#),
            Ok(RecordType::RoundTrip)
        ));
    }

    #[test]
    fn null_ttl_uses_default() {
        let record: DnsRecord =
            serde_json::from_str(r#"{"record_type":"single","ips":[],"ttl":null}"#)
                .expect("null TTL should use the default");

        assert_eq!(record.ttl, 60);
    }

    #[test]
    fn numeric_ttl_is_preserved() {
        let record: DnsRecord =
            serde_json::from_str(r#"{"record_type":"single","ips":[],"ttl":120}"#)
                .expect("numeric TTL should deserialize");

        assert_eq!(record.ttl, 120);
    }

    #[test]
    fn client_country_is_optional_and_deserializable() {
        let record: DnsRecord =
            serde_json::from_str(r#"{"record_type":"geo","ips":[],"client_country":"CR"}"#)
                .expect("client country should deserialize");

        assert_eq!(record.client_country.as_deref(), Some("CR"));

        let record: DnsRecord = serde_json::from_str(r#"{"record_type":"geo","ips":[]}"#)
            .expect("missing client country should remain compatible");
        assert_eq!(record.client_country, None);
    }

    #[test]
    fn deserializes_round_trip_runtime_data() {
        let record: DnsRecord = serde_json::from_str(
            r#"{
                "record_type":"round-trip",
                "ips":[{"ip":"192.0.2.10","healthy":true}],
                "client_country":"CR",
                "client_city":"San Jose",
                "client_location":{"latitude":9.9281,"longitude":-84.0907},
                "health_checkers":[{"checker_id":"cr","latitude":9.9,"longitude":-84.0}],
                "measurements":[{"checker_id":"cr","target_ip":"192.0.2.10","rtt_ms":12.5,"healthy":true}]
            }"#,
        )
        .expect("runtime Round Trip data should deserialize");

        assert_eq!(record.client_country.as_deref(), Some("CR"));
        assert_eq!(record.client_city.as_deref(), Some("San Jose"));
        assert_eq!(
            record.client_location.map(|location| location.latitude),
            Some(9.9281)
        );
        assert_eq!(record.health_checkers[0].id, "cr");
        assert_eq!(
            record.measurements[0].target_ip,
            Ipv4Addr::new(192, 0, 2, 10)
        );
    }
}
