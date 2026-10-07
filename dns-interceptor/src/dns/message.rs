//! Wrapper de alto nivel: junta header + question + bytes crudos, y ofrece helpers para decidir qué hacer con el paquete.

use anyhow::Result;
use std::net::Ipv4Addr;

use super::header::DnsHeader;
use super::question::Question;

/// Clasificación del paquete según el enunciado del proyecto:
/// - `StandardQuery` = "Caso 2" (QR=0, OPCODE=0) -> se intenta resolver localmente.
/// - `Other`         = "Caso 1" (cualquier otra cosa) -> forward crudo en BASE64 al DNS API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestCase {
    StandardQuery,
    Other,
}

impl std::fmt::Display for RequestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestCase::StandardQuery => write!(f, "Caso 2 (query estándar)"),
            RequestCase::Other => write!(f, "Caso 1 (no-estándar)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DnsMessage {
    pub header: DnsHeader,
    pub question: Option<Question>,
    pub raw: Vec<u8>, // paquete original, tal cual llegó por UDP
}

impl DnsMessage {
    pub fn parse(buf: &[u8]) -> Result<Self> {
        let header = DnsHeader::parse(buf)?;

        // Solo intentamos parsear la Question si el header dice que hay al menos una
        // y si el paquete es lo bastante largo.
        let question = if header.qdcount >= 1 {
            Question::parse(buf).ok().map(|(q, _)| q)
        } else {
            None
        };

        Ok(DnsMessage {
            header,
            question,
            raw: buf.to_vec(),
        })
    }

    /// true si es un query estándar (QR=0, OPCODE=0) — el único caso donde el
    /// interceptor intenta resolver localmente antes de hacer forward.
    pub fn is_standard_query(&self) -> bool {
        self.header.is_standard_query()
    }

    /// Clasifica el paquete en Caso 1 / Caso 2 según el enunciado.
    pub fn case(&self) -> RequestCase {
        if self.is_standard_query() {
            RequestCase::StandardQuery
        } else {
            RequestCase::Other
        }
    }

    /// Nombre de dominio solicitado, si se pudo parsear la Question.
    pub fn domain(&self) -> Option<&str> {
        self.question.as_ref().map(|q| q.qname.as_str())
    }

    /// Arma una respuesta DNS válida con una o más IPs tipo A, reusando el
    /// header/question original de la query.
    pub fn build_response(&self, ips: &[Ipv4Addr], ttl: u32) -> Result<Vec<u8>> {
        let question = self
            .question
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no hay Question para construir la respuesta"))?;

        let response_header =
            DnsHeader::response_from_query(&self.header, ips.len() as u16, 0 /* NOERROR */);

        let mut out = Vec::new();
        out.extend_from_slice(&response_header.to_bytes());
        out.extend_from_slice(&question.to_bytes());
        out.extend_from_slice(&super::record::build_a_records(ips, ttl));

        Ok(out)
    }

    /// Arma una respuesta de error (ej. NXDOMAIN) reusando el header/question original.
    pub fn build_error_response(&self, rcode: u8) -> Result<Vec<u8>> {
        let question = self
            .question
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no hay Question para construir la respuesta"))?;

        let response_header = DnsHeader::response_from_query(&self.header, 0, rcode);

        let mut out = Vec::new();
        out.extend_from_slice(&response_header.to_bytes());
        out.extend_from_slice(&question.to_bytes());

        Ok(out)
    }
}
