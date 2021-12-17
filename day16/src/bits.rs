use std::str::FromStr;

use bitreader::BitReader;
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum Type {
    Literal = 4,
    #[num_enum(default)]
    UnknownOperator = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub type_id: Type,
}

impl Header {
    /// Read the header data from the bitreader.
    ///
    /// Return `(Self, num_bits_read)`, or an error.
    fn read(reader: &mut BitReader) -> Result<(Self, usize), Error> {
        let version = reader.read_u8(3).map_err(Error::Header)?;
        let type_id = reader.read_u8(3).map_err(Error::Header)?;
        Ok((
            Header {
                version,
                type_id: type_id.into(),
            },
            6,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
enum LengthType {
    TotalBits = 0,
    #[num_enum(default)]
    NumberSubPackets = 1,
}

impl LengthType {
    fn continue_looping(self, bits_read: usize, packets_read: usize, target: usize) -> bool {
        match self {
            LengthType::TotalBits => bits_read < target,
            LengthType::NumberSubPackets => packets_read < target,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Literal(Vec<u64>),
    SubPackets(Vec<Packet>),
}

impl Payload {
    /// Read the payload data from the bitreader.
    ///
    /// Return `(Self, num_bits_read)`, or an error.
    fn read(type_id: Type, reader: &mut BitReader) -> Result<(Self, usize), Error> {
        match type_id {
            Type::Literal => {
                const GROUP_SIZE: u8 = 5;
                let mut output = Vec::new();
                let mut is_last = false;
                let mut bits_read = 0;

                while !is_last {
                    let mut chunk = 0;
                    for _ in 0..(u64::BITS / 4) {
                        let group = reader.read_u64(GROUP_SIZE).map_err(Error::LiteralGroup)?;
                        bits_read += GROUP_SIZE as usize;
                        chunk = (chunk << 4) | (group & 0xf);

                        is_last = group & (1 << 4) == 0;
                        if is_last {
                            break;
                        }
                    }

                    output.push(chunk);
                }

                Ok((Payload::Literal(output), bits_read))
            }
            Type::UnknownOperator => {
                let mut bits_read = 0;
                let mut subpacket_bits_read = 0;
                let mut packets_read = 0;

                let length_type: LengthType = reader.read_u8(1).map_err(Error::LengthType)?.into();
                bits_read += 1;
                let target = match length_type {
                    LengthType::TotalBits => {
                        bits_read += 15;
                        reader.read_u16(15).map_err(Error::LengthTarget)? as usize
                    }
                    LengthType::NumberSubPackets => {
                        bits_read += 11;
                        reader.read_u16(11).map_err(Error::LengthTarget)? as usize
                    }
                };

                let mut subpackets = Vec::new();
                while length_type.continue_looping(subpacket_bits_read, packets_read, target) {
                    let (packet, packet_bits) = Packet::read(reader)?;
                    packets_read += 1;
                    bits_read += packet_bits;
                    subpacket_bits_read += packet_bits;
                    subpackets.push(packet);
                }

                Ok((Payload::SubPackets(subpackets), bits_read))
            }
        }
    }

    pub fn as_literal(&self) -> Option<&Vec<u64>> {
        match self {
            Payload::Literal(ref value) => Some(value),
            Payload::SubPackets(_) => None,
        }
    }

    pub fn as_subpackets(&self) -> Option<&Vec<Packet>> {
        match self {
            Payload::Literal(_) => None,
            Payload::SubPackets(ref packets) => Some(packets),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub header: Header,
    pub payload: Payload,
}

impl Packet {
    fn read(reader: &mut BitReader) -> Result<(Self, usize), Error> {
        let mut bits_read = 0;
        let (header, bits) = Header::read(reader)?;
        bits_read += bits;
        let (payload, bits) = Payload::read(header.type_id, reader)?;
        bits_read += bits;

        Ok((Packet { header, payload }, bits_read))
    }

    /// Parse a slice of data as a packet.
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        Self::read(&mut BitReader::new(data)).map(|(packet, _)| packet)
    }

    /// Parse a hex string as a packet.
    pub fn parse_hex(data: &str) -> Result<Self, Error> {
        Self::parse(&hex::decode(data)?)
    }
}

impl FromStr for Packet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_hex(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("header")]
    Header(#[source] bitreader::BitReaderError),
    #[error("literal group")]
    LiteralGroup(#[source] bitreader::BitReaderError),
    #[error("length type")]
    LengthType(#[source] bitreader::BitReaderError),
    #[error("length target")]
    LengthTarget(#[source] bitreader::BitReaderError),
    #[error("parsing hex")]
    HexDecode(#[from] hex::FromHexError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_literal() {
        let packet = Packet::parse_hex("D2FE28").unwrap();
        assert_eq!(packet.header.version, 6);
        assert_eq!(packet.header.type_id, Type::Literal);
        assert_eq!(packet.payload, Payload::Literal(vec![2021]));
    }

    #[test]
    fn example_operator_bit_type() {
        let packet = Packet::parse_hex("38006F45291200").unwrap();
        assert_eq!(packet.header.version, 1);
        assert_eq!(packet.header.type_id, Type::UnknownOperator);
        let subpackets = packet.payload.as_subpackets().unwrap();
        assert_eq!(subpackets.len(), 2);
        assert_eq!(subpackets[0].payload.as_literal().unwrap(), &[10]);
        assert_eq!(subpackets[1].payload.as_literal().unwrap(), &[20]);
    }

    #[test]
    fn example_operator_subpacket_type() {
        let packet = Packet::parse_hex("EE00D40C823060").unwrap();
        assert_eq!(packet.header.version, 7);
        assert_eq!(packet.header.type_id, Type::UnknownOperator);
        let subpackets = packet.payload.as_subpackets().unwrap();
        assert_eq!(subpackets.len(), 3);
        assert_eq!(subpackets[0].payload.as_literal().unwrap(), &[1]);
        assert_eq!(subpackets[1].payload.as_literal().unwrap(), &[2]);
        assert_eq!(subpackets[2].payload.as_literal().unwrap(), &[3]);
    }

    #[test]
    #[allow(non_snake_case)]
    fn example_8A004A801A8002F478() {
        let packet = Packet::parse_hex("8A004A801A8002F478").unwrap();
        assert_eq!(packet.header.version, 4);
        let packet = &packet.payload.as_subpackets().unwrap()[0];
        assert_eq!(packet.header.version, 1);
        let packet = &packet.payload.as_subpackets().unwrap()[0];
        assert_eq!(packet.header.version, 5);
        let packet = &packet.payload.as_subpackets().unwrap()[0];
        assert_eq!(packet.header.version, 6);
        assert!(packet.payload.as_literal().is_some());
    }
}
