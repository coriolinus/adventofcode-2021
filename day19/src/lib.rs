use aoclib::{geometry::vector3::Vector3, input::parse_newline_sep};
use std::{
    io::{BufRead, Cursor},
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
#[display("{x},{y},{z}")]
struct Vector3Parse {
    x: i32,
    y: i32,
    z: i32,
}

impl From<Vector3Parse> for Vector3 {
    fn from(v: Vector3Parse) -> Self {
        Vector3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
#[display("--- scanner {0} ---")]
struct ScannerHeader(u32);

#[derive(Debug, Clone)]
struct Scanner {
    id: u32,
    beacons: Vec<Vector3>,
}

impl FromStr for Scanner {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<String> = Cursor::new(s).lines().collect::<Result<Vec<String>, _>>()?;
        let header = lines
            .get(0)
            .ok_or(Error::ParseError)?
            .trim()
            .parse::<ScannerHeader>()?;
        let mut beacons = Vec::with_capacity(lines.len().checked_sub(1).unwrap_or_default());
        for beacon_line in &lines[1..] {
            if beacon_line.is_empty() {
                continue;
            }
            beacons.push(beacon_line.parse::<Vector3Parse>()?.into());
        }
        Ok(Scanner {
            id: header.0,
            beacons,
        })
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    for scanner in parse_newline_sep::<Scanner>(input)? {
        println!(
            "scanner {:2}: {} beacons in sight",
            scanner.id,
            scanner.beacons.len()
        );
    }
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse a scanner or beacon")]
    ParseError,
    #[error(transparent)]
    DeriveParseError(#[from] parse_display::ParseError),
    #[error("no solution found")]
    NoSolution,
}
