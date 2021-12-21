use aoclib::{geometry::vector3::Vector3, input::parse_newline_sep};
use enum_iterator::IntoEnumIterator;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntoEnumIterator)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntoEnumIterator)]
enum Negation {
    Positive,
    Negative,
}

/// An `Orientation` is a transform from one labeled coordinate system to another.
///
/// Each coordinate system is assumed to share the same orthogonal axes; the only
/// difference is the label and direction of positive assigned to each axis.
///
/// There are 48 possible orientations: 6 permutations of axis labels multiplied
/// by 8 possible axis-positive assignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Orientation([(Negation, Axis); 3]);

impl Orientation {
    fn exhaustive_iterator() -> impl Iterator<Item = Orientation> {
        todo!();
        std::iter::empty()
    }

    fn extract(point: Vector3, (negation, axis): (Negation, Axis)) -> i32 {
        let negation = match negation {
            Negation::Positive => 1,
            Negation::Negative => -1,
        };
        let value = match axis {
            Axis::X => point.x,
            Axis::Y => point.y,
            Axis::Z => point.z,
        };
        negation * value
    }

    fn transform(self, point: Vector3) -> Vector3 {
        Vector3 {
            x: Self::extract(point, self.0[0]),
            y: Self::extract(point, self.0[1]),
            z: Self::extract(point, self.0[2]),
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        use Axis::*;
        use Negation::Positive;

        Self([(Positive, X), (Positive, Y), (Positive, Z)])
    }
}

#[derive(Debug, Clone, Default)]
struct Scanner {
    id: u32,
    beacons: Vec<Vector3>,
    absolute_position: Option<Vector3>,
    orientation: Option<Orientation>,
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
            ..Scanner::default()
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
