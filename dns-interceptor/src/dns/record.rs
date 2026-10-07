//! Construcción de registros de respuesta (sección Answer, RFC 1035 sección 4.1.3).
//! Formato de un Resource Record:
//! NAME | TYPE(2) | CLASS(2) | TTL(4) | RDLENGTH(2) | RDATA(RDLENGTH)

use std::net::Ipv4Addr;

pub const TYPE_A: u16 = 1;
pub const CLASS_IN: u16 = 1;

/// Puntero de compresión estándar hacia el nombre en la sección Question
/// (offset 0x0C = 12, justo después del header). Ahorra tener que repetir
/// el QNAME completo en cada Answer.
const NAME_POINTER: [u8; 2] = [0xC0, 0x0C];

/// Arma un Resource Record tipo A (IPv4) apuntando al QNAME original.
pub fn build_a_record(ip: Ipv4Addr, ttl: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&NAME_POINTER);
    out.extend_from_slice(&TYPE_A.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
    out.extend_from_slice(&ttl.to_be_bytes());

    let rdata = ip.octets();
    out.extend_from_slice(&(rdata.len() as u16).to_be_bytes()); // RDLENGTH = 4
    out.extend_from_slice(&rdata);

    out
}

/// Arma varios registros A (útil para el tipo "multi", donde se podrían
/// devolver todas las IPs, o solo la elegida por round-robin según se decida).
pub fn build_a_records(ips: &[Ipv4Addr], ttl: u32) -> Vec<u8> {
    let mut out = Vec::new();
    for ip in ips {
        out.extend_from_slice(&build_a_record(*ip, ttl));
    }
    out
}
