use std::str::FromStr;

use bitreader::BitReader;
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum Type {
    Sum = 0,
    Product = 1,
    Minimum = 2,
    Maximum = 3,
    Literal = 4,
    GreaterThan = 5,
    LessThan = 6,
    EqualTo = 7,
    #[num_enum(default)]
    UnknownOperator = u8::MAX,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub type_id: Type,
}

impl Header {
    fn read(reader: &mut BitReader) -> Result<Self, Error> {
        let version = reader.read_u8(3).map_err(Error::Header)?;
        let type_id = reader.read_u8(3).map_err(Error::Header)?.into();
        Ok(Header { version, type_id })
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
    fn continue_looping(self, bits_read: u64, packets_read: u64, target: u64) -> bool {
        match self {
            LengthType::TotalBits => bits_read < target,
            LengthType::NumberSubPackets => packets_read < target,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Literal(u64),
    SubPackets(Vec<Packet>),
}

impl Payload {
    fn read(type_id: Type, reader: &mut BitReader) -> Result<Self, Error> {
        if let Type::Literal = type_id {
            let mut is_last = false;
            let mut filled_bits = 0;

            let mut value = 0;
            while !is_last && filled_bits <= u64::BITS {
                let group = reader.read_u64(5).map_err(Error::LiteralGroup)?;
                value = (value << 4) | (group & 0xf);
                filled_bits += 4;

                is_last = group & (1 << 4) == 0;
            }

            if !is_last {
                return Err(Error::OversizeLiteral);
            }

            Ok(Payload::Literal(value))
        } else {
            let length_type: LengthType = reader.read_u8(1).map_err(Error::LengthType)?.into();
            let target = match length_type {
                LengthType::TotalBits => reader.read_u64(15).map_err(Error::LengthTarget)?,
                LengthType::NumberSubPackets => reader.read_u64(11).map_err(Error::LengthTarget)?,
            };

            let pre_subpacket_index = reader.position();

            let mut subpackets = Vec::new();
            while length_type.continue_looping(
                reader.position() - pre_subpacket_index,
                subpackets.len() as u64,
                target,
            ) {
                subpackets.push(Packet::read(reader)?);
            }

            Ok(Payload::SubPackets(subpackets))
        }
    }

    pub fn as_literal(&self) -> Option<u64> {
        match self {
            Payload::Literal(value) => Some(*value),
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
    fn read(reader: &mut BitReader) -> Result<Self, Error> {
        let header = Header::read(reader)?;
        let payload = Payload::read(header.type_id, reader)?;

        Ok(Packet { header, payload })
    }

    /// Parse a slice of data as a packet.
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        Self::read(&mut BitReader::new(data))
    }

    /// Parse a hex string as a packet.
    pub fn parse_hex(data: &str) -> Result<Self, Error> {
        Self::parse(&hex::decode(data)?)
    }

    /// Compute the value of the packet.
    pub fn value(&self) -> u64 {
        fn subpacket_values<'a>(packet: &'a Packet) -> impl 'a + Iterator<Item = u64> {
            packet
                .payload
                .as_subpackets()
                .unwrap()
                .iter()
                .map(|packet| packet.value())
        }

        fn compare_two(packet: &Packet, comparitor: std::cmp::Ordering) -> u64 {
            let subpackets = packet.payload.as_subpackets().unwrap();
            if subpackets.len() != 2 {
                eprintln!(
                    "WARN: {:?} packet had {} subpackets; expected 2",
                    packet.header.type_id,
                    subpackets.len()
                );
                return 0;
            }
            if subpackets[0].value().cmp(&subpackets[1].value()) == comparitor {
                1
            } else {
                0
            }
        }

        match self.header.type_id {
            Type::Literal => self.payload.as_literal().unwrap(),
            Type::Sum => subpacket_values(self).sum(),
            Type::Product => subpacket_values(self).product(),
            Type::Minimum => subpacket_values(self).min().unwrap_or_default(),
            Type::Maximum => subpacket_values(self).max().unwrap_or_default(),
            Type::GreaterThan => compare_two(self, std::cmp::Ordering::Greater),
            Type::LessThan => compare_two(self, std::cmp::Ordering::Less),
            Type::EqualTo => compare_two(self, std::cmp::Ordering::Equal),
            Type::UnknownOperator => panic!("unknown operator has no value"),
        }
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
    #[error("literal does not fit into u64")]
    OversizeLiteral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_literal() {
        let packet = Packet::parse_hex("D2FE28").unwrap();
        assert_eq!(packet.header.version, 6);
        assert_eq!(packet.header.type_id, Type::Literal);
        assert_eq!(packet.payload, Payload::Literal(2021));
    }

    #[test]
    fn example_operator_bit_type() {
        let packet = Packet::parse_hex("38006F45291200").unwrap();
        assert_eq!(packet.header.version, 1);
        let subpackets = packet.payload.as_subpackets().unwrap();
        assert_eq!(subpackets.len(), 2);
        assert_eq!(subpackets[0].payload.as_literal().unwrap(), 10);
        assert_eq!(subpackets[1].payload.as_literal().unwrap(), 20);
    }

    #[test]
    fn example_operator_subpacket_type() {
        let packet = Packet::parse_hex("EE00D40C823060").unwrap();
        assert_eq!(packet.header.version, 7);
        let subpackets = packet.payload.as_subpackets().unwrap();
        assert_eq!(subpackets.len(), 3);
        assert_eq!(subpackets[0].payload.as_literal().unwrap(), 1);
        assert_eq!(subpackets[1].payload.as_literal().unwrap(), 2);
        assert_eq!(subpackets[2].payload.as_literal().unwrap(), 3);
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
