// crates/varex-wisp/src/protocol.rs
//! Wisp v1.2 wire codec, verified against
//! https://github.com/MercuryWorkshop/wisp-protocol/blob/v1/protocol.md
//!
//! §Packet format (lines 10–14): every packet is
//!     type:u8 | stream_id:u32 | payload
//! and all integers are little-endian.
//!
//! §0x01 CONNECT (lines 15–19): payload is
//!     stream_type:u8 | port:u16 | hostname (rest of packet, UTF-8)
//! There is no length prefix on the hostname.
//!
//! §0x02 DATA (lines 32–42): payload is the raw byte slice.
//!
//! §0x03 CONTINUE (lines 43–46): payload is buffer_remaining:u32, and
//! the server must send an initial CONTINUE on stream 0 immediately
//! after the WebSocket opens (lines 17–18). UDP streams must not use
//! CONTINUE.
//!
//! §0x04 CLOSE: payload is reason:u8.
//! §0x05 INFO: payload is version:u8 followed by extension bytes.
use bytes::{Buf, BufMut, Bytes, BytesMut};
use varex_core::{Result, VarexError};

pub const TYPE_CONNECT: u8 = 0x01;
pub const TYPE_DATA: u8 = 0x02;
pub const TYPE_CONTINUE: u8 = 0x03;
pub const TYPE_CLOSE: u8 = 0x04;
pub const TYPE_INFO: u8 = 0x05;

pub const STREAM_TCP: u8 = 0x01;
pub const STREAM_UDP: u8 = 0x02;

pub const REASON_UNSPECIFIED: u8 = 0x01;
pub const REASON_TIMEOUT: u8 = 0x02;
pub const REASON_REFUSED: u8 = 0x03;
pub const REASON_NETWORK: u8 = 0x04;

pub const HEADER_LEN: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Packet {
    Connect { stream_id: u32, stream_type: u8, port: u16, host: String },
    Data { stream_id: u32, payload: Bytes },
    Continue { stream_id: u32, buffer_remaining: u32 },
    Close { stream_id: u32, reason: u8 },
    Info { stream_id: u32, version: u8, extensions: Vec<u8> },
}

impl Packet {
    pub fn parse(mut src: Bytes) -> Result<Self> {
        if src.len() < HEADER_LEN {
            return Err(VarexError::Protocol(format!("short header: {}", src.len())));
        }
        let ptype = src.get_u8();
        let stream_id = src.get_u32_le();
        match ptype {
            TYPE_CONNECT => {
                if src.len() < 3 { return Err(VarexError::Protocol("CONNECT truncated".into())); }
                let stream_type = src.get_u8();
                let port = src.get_u16_le();
                if stream_type != STREAM_TCP && stream_type != STREAM_UDP {
                    return Err(VarexError::Protocol(format!("bad stream_type 0x{stream_type:02x}")));
                }
                if src.is_empty() { return Err(VarexError::BadConnect("empty hostname".into())); }
                let host = std::str::from_utf8(&src)
                    .map_err(|_| VarexError::BadConnect("hostname not UTF-8".into()))?.to_owned();
                if host.len() > 253 || host.contains('\0') {
                    return Err(VarexError::BadConnect("invalid hostname".into()));
                }
                Ok(Packet::Connect { stream_id, stream_type, port, host })
            }
            TYPE_DATA => Ok(Packet::Data { stream_id, payload: src }),
            TYPE_CONTINUE => {
                if src.len() < 4 { return Err(VarexError::Protocol("CONTINUE truncated".into())); }
                Ok(Packet::Continue { stream_id, buffer_remaining: src.get_u32_le() })
            }
            TYPE_CLOSE => {
                if src.is_empty() { return Err(VarexError::Protocol("CLOSE missing reason".into())); }
                Ok(Packet::Close { stream_id, reason: src.get_u8() })
            }
            TYPE_INFO => {
                if src.is_empty() { return Err(VarexError::Protocol("INFO missing version".into())); }
                let version = src.get_u8();
                Ok(Packet::Info { stream_id, version, extensions: src.to_vec() })
            }
            other => Err(VarexError::Protocol(format!("unknown type 0x{other:02x}"))),
        }
    }
}

pub struct Encoder;
impl Encoder {
    pub fn data(buf: &mut BytesMut, stream_id: u32, payload: &[u8]) {
        buf.reserve(HEADER_LEN + payload.len());
        buf.put_u8(TYPE_DATA); buf.put_u32_le(stream_id); buf.put_slice(payload);
    }
    pub fn continue_(buf: &mut BytesMut, stream_id: u32, buffer_remaining: u32) {
        buf.reserve(HEADER_LEN + 4);
        buf.put_u8(TYPE_CONTINUE); buf.put_u32_le(stream_id); buf.put_u32_le(buffer_remaining);
    }
    pub fn close(buf: &mut BytesMut, stream_id: u32, reason: u8) {
        buf.reserve(HEADER_LEN + 1);
        buf.put_u8(TYPE_CLOSE); buf.put_u32_le(stream_id); buf.put_u8(reason);
    }
    pub fn info(buf: &mut BytesMut, stream_id: u32, version: u8, extensions: &[u8]) {
        buf.reserve(HEADER_LEN + 1 + extensions.len());
        buf.put_u8(TYPE_INFO); buf.put_u32_le(stream_id); buf.put_u8(version); buf.put_slice(extensions);
    }
    pub fn connect(buf: &mut BytesMut, stream_id: u32, stream_type: u8, port: u16, host: &str) {
        buf.reserve(HEADER_LEN + 3 + host.len());
        buf.put_u8(TYPE_CONNECT); buf.put_u32_le(stream_id); buf.put_u8(stream_type);
        buf.put_u16_le(port); buf.put_slice(host.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn roundtrip(p: Packet) {
        let mut buf = BytesMut::new();
        match &p {
            Packet::Connect { stream_id, stream_type, port, host } => Encoder::connect(&mut buf, *stream_id, *stream_type, *port, host),
            Packet::Data { stream_id, payload } => Encoder::data(&mut buf, *stream_id, payload),
            Packet::Continue { stream_id, buffer_remaining } => Encoder::continue_(&mut buf, *stream_id, *buffer_remaining),
            Packet::Close { stream_id, reason } => Encoder::close(&mut buf, *stream_id, *reason),
            Packet::Info { stream_id, version, extensions } => Encoder::info(&mut buf, *stream_id, *version, extensions),
        }
        assert_eq!(Packet::parse(buf.freeze()).unwrap(), p);
    }

    proptest! {
        #[test]
        fn data_roundtrip(id: u32, payload in proptest::collection::vec(any::<u8>(), 0..1024)) {
            roundtrip(Packet::Data { stream_id: id, payload: Bytes::from(payload) });
        }
        #[test]
        fn continue_roundtrip(id: u32, rem: u32) {
            roundtrip(Packet::Continue { stream_id: id, buffer_remaining: rem });
        }
        #[test]
        fn close_roundtrip(id: u32, reason: u8) {
            roundtrip(Packet::Close { stream_id: id, reason });
        }
    }

    #[test]
    fn connect_layout_has_no_hostname_length_byte() {
        // §0x01 CONNECT: stream_type:u8 | port:u16 | hostname (rest).
        // 5 hdr + 1 type + 2 port + 11 host = 19 bytes total.
        let mut buf = BytesMut::new();
        Encoder::connect(&mut buf, 7, STREAM_TCP, 443, "example.com");
        assert_eq!(buf.len(), 19);
        assert_eq!(&buf[..5], &[TYPE_CONNECT, 7, 0, 0, 0]);
    }

    #[test]
    fn initial_continue_uses_stream_zero() {
        // §0x03 CONTINUE, lines 17–18: initial CONTINUE must be on stream 0.
        let mut buf = BytesMut::new();
        Encoder::continue_(&mut buf, 0, 127);
        assert_eq!(&buf[..], &[0x03, 0, 0, 0, 0, 127, 0, 0, 0]);
    }
}
