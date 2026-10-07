//! Parseo de la sección Question de un mensaje DNS (RFC 1035, sección 4.1.2).
//! QNAME es una secuencia de "labels" con formato: [longitud][bytes...] y termina en un byte 0x00.

use anyhow::{bail, Result};

use super::header::HEADER_LEN;

#[derive(Debug, Clone)]
pub struct Question {
    pub qname: String, 
    pub qtype: u16,     
    pub qclass: u16,    
}

impl Question {
    /// Parsea la primera Question del mensaje (asumiendo QDCOUNT >= 1).
    /// Devuelve la Question y el offset (en bytes, desde el inicio del mensaje)
    /// donde termina la sección Question — útil si más adelante se necesita leer
    /// secciones posteriores.
    pub fn parse(buf: &[u8]) -> Result<(Self, usize)> {
        let mut pos = HEADER_LEN;
        let mut labels = Vec::new();

        loop {
            if pos >= buf.len() {
                bail!("mensaje truncado leyendo QNAME");
            }
            let len = buf[pos] as usize;
            pos += 1;

            if len == 0 {
                break; // fin del QNAME
            }

            // Nota: no manejamos compresión de nombres (bits 0xC0) aquí porque
            // en la sección Question de una query estándar no debería aparecer.
            if pos + len > buf.len() {
                bail!("mensaje truncado leyendo un label de QNAME");
            }

            let label = std::str::from_utf8(&buf[pos..pos + len])?;
            labels.push(label.to_string());
            pos += len;
        }

        if pos + 4 > buf.len() {
            bail!("mensaje truncado leyendo QTYPE/QCLASS");
        }

        let qtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let qclass = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
        pos += 4;

        Ok((
            Question {
                qname: labels.join("."),
                qtype,
                qclass,
            },
            pos,
        ))
    }

    /// Serializa esta Question de vuelta a bytes (para reusarla en la respuesta).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for label in self.qname.split('.') {
            out.push(label.len() as u8);
            out.extend_from_slice(label.as_bytes());
        }
        out.push(0); // terminador
        out.extend_from_slice(&self.qtype.to_be_bytes());
        out.extend_from_slice(&self.qclass.to_be_bytes());
        out
    }
}
