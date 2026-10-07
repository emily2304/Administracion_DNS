//! Parseo y serialización del header DNS (12 bytes iniciales, RFC 1035 / RFC 2929).
//!
//!  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                      ID                       |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    QDCOUNT                     |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    ANCOUNT                     |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    NSCOUNT                     |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    ARCOUNT                     |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

use anyhow::{bail, Result};

pub const HEADER_LEN: usize = 12;

#[derive(Debug, Clone, Copy)]
pub struct DnsHeader {
    pub id: u16,
    pub qr: bool,     // 0 = query, 1 = response
    pub opcode: u8,   // 4 bits. 0 = standard query
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub rcode: u8,    // 4 bits
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl DnsHeader {
    /// Es un query estándar si QR = 0 y OPCODE = 0.
    pub fn is_standard_query(&self) -> bool {
        !self.qr && self.opcode == 0
    }

    pub fn parse(buf: &[u8]) -> Result<Self> {
        if buf.len() < HEADER_LEN {
            bail!("paquete demasiado corto para tener header DNS ({} bytes)", buf.len());
        }

        let id = u16::from_be_bytes([buf[0], buf[1]]);

        let flags_hi = buf[2];
        let flags_lo = buf[3];

        let qr = (flags_hi & 0b1000_0000) != 0;
        let opcode = (flags_hi & 0b0111_1000) >> 3;
        let aa = (flags_hi & 0b0000_0100) != 0;
        let tc = (flags_hi & 0b0000_0010) != 0;
        let rd = (flags_hi & 0b0000_0001) != 0;

        let ra = (flags_lo & 0b1000_0000) != 0;
        let rcode = flags_lo & 0b0000_1111;

        let qdcount = u16::from_be_bytes([buf[4], buf[5]]);
        let ancount = u16::from_be_bytes([buf[6], buf[7]]);
        let nscount = u16::from_be_bytes([buf[8], buf[9]]);
        let arcount = u16::from_be_bytes([buf[10], buf[11]]);

        Ok(DnsHeader {
            id, qr, opcode, aa, tc, rd, ra, rcode,
            qdcount, ancount, nscount, arcount,
        })
    }

    /// Serializa el header a 12 bytes, listo para anteponer a las secciones del mensaje.
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut flags_hi = 0u8;
        if self.qr { flags_hi |= 0b1000_0000; }
        flags_hi |= (self.opcode & 0b0000_1111) << 3;
        if self.aa { flags_hi |= 0b0000_0100; }
        if self.tc { flags_hi |= 0b0000_0010; }
        if self.rd { flags_hi |= 0b0000_0001; }

        let mut flags_lo = 0u8;
        if self.ra { flags_lo |= 0b1000_0000; }
        flags_lo |= self.rcode & 0b0000_1111;

        let mut out = [0u8; HEADER_LEN];
        out[0..2].copy_from_slice(&self.id.to_be_bytes());
        out[2] = flags_hi;
        out[3] = flags_lo;
        out[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        out[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        out[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        out[10..12].copy_from_slice(&self.arcount.to_be_bytes());
        out
    }

    /// Arma un header de respuesta a partir del header de la query original.
    pub fn response_from_query(query: &DnsHeader, ancount: u16, rcode: u8) -> Self {
        DnsHeader {
            id: query.id,
            qr: true,
            opcode: query.opcode,
            aa: false,
            tc: false,
            rd: query.rd,
            ra: true,
            rcode,
            qdcount: query.qdcount,
            ancount,
            nscount: 0,
            arcount: 0,
        }
    }
}
