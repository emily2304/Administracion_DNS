// Pruebas unitarias del parseo DNS. Corren contra los tipos públicos del crate,
// así que hay que exponerlos como lib (ver nota abajo) o mover estos tests a
// src/dns/*.rs como #[cfg(test)] mod tests si se deja el crate como binario puro.

// Ejemplo de query DNS estándar para "example.com" tipo A, clase IN.
fn sample_standard_query() -> Vec<u8> {
    let mut pkt = Vec::new();
    // Header: ID=0x1234, flags=0x0100 (QR=0, OPCODE=0, RD=1), QDCOUNT=1, resto 0
    pkt.extend_from_slice(&[0x12, 0x34]); // ID
    pkt.extend_from_slice(&[0x01, 0x00]); // flags
    pkt.extend_from_slice(&[0x00, 0x01]); // QDCOUNT
    pkt.extend_from_slice(&[0x00, 0x00]); // ANCOUNT
    pkt.extend_from_slice(&[0x00, 0x00]); // NSCOUNT
    pkt.extend_from_slice(&[0x00, 0x00]); // ARCOUNT

    // Question: example.com, tipo A (1), clase IN (1)
    for label in ["example", "com"] {
        pkt.push(label.len() as u8);
        pkt.extend_from_slice(label.as_bytes());
    }
    pkt.push(0x00); // fin de QNAME
    pkt.extend_from_slice(&[0x00, 0x01]); // QTYPE = A
    pkt.extend_from_slice(&[0x00, 0x01]); // QCLASS = IN

    pkt
}

#[test]
fn header_detects_standard_query() {
    let pkt = sample_standard_query();
    let header = dns_interceptor::dns::DnsHeader::parse(&pkt).unwrap();
    assert!(header.is_standard_query());
    assert_eq!(header.id, 0x1234);
    assert_eq!(header.qdcount, 1);
}

#[test]
fn question_parses_domain_name() {
    let pkt = sample_standard_query();
    let (question, _) = dns_interceptor::dns::Question::parse(&pkt).unwrap();
    assert_eq!(question.qname, "example.com");
    assert_eq!(question.qtype, 1);
    assert_eq!(question.qclass, 1);
}

#[test]
fn message_roundtrip_builds_valid_response() {
    let pkt = sample_standard_query();
    let msg = dns_interceptor::dns::DnsMessage::parse(&pkt).unwrap();
    assert_eq!(msg.domain(), Some("example.com"));

    let ip: std::net::Ipv4Addr = "93.184.216.34".parse().unwrap();
    let response = msg.build_response(&[ip], 60).unwrap();

    // El header de la respuesta debe tener QR=1 y ANCOUNT=1.
    let resp_header = dns_interceptor::dns::DnsHeader::parse(&response).unwrap();
    assert!(resp_header.qr);
    assert_eq!(resp_header.ancount, 1);
}
