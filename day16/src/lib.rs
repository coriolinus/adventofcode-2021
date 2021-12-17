pub mod bits;

use bits::{Packet, Payload};
use std::path::Path;

fn sum_versions(packet: &Packet) -> u64 {
    let mut sum = packet.header.version as u64;

    if let Payload::SubPackets(ref subpackets) = packet.payload {
        sum += subpackets.iter().map(sum_versions).sum::<u64>();
    }

    sum
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let data = std::fs::read_to_string(input)?;
    let packet = Packet::parse_hex(data.trim())?;
    println!("version sum: {}", sum_versions(&packet));
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parsing packet")]
    Packet(#[from] bits::Error),
    #[error("no solution found")]
    NoSolution,
}
