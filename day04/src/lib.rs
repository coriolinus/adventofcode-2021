pub mod two_phase;

use aoclib::{
    geometry::{tile::DisplayWidth, Direction, Map},
    CommaSep,
};
use std::{fmt::Display, path::Path, str::FromStr};
use two_phase::{parse_two_phase, TrimmedCommaSep};

const HIGH_BIT: u8 = 0x80;
const LOW_BITS: u8 = !HIGH_BIT;

/// A tile on a bingo board.
///
/// We get a little fancy here: the value of any particular tile depends on
/// the low seven bits, while the high bit is used to indicate whether or not
/// the square has been marked. This is valid because we know that the value of
/// a particular tile never exceeds decimal 99, which can be represented in 7
/// bits.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
struct Tile(u8);

impl Tile {
    fn is_marked(self) -> bool {
        self.0 & HIGH_BIT != 0
    }

    fn value(self) -> u8 {
        self.0 & LOW_BITS
    }

    fn mark(&mut self) {
        self.0 |= HIGH_BIT
    }

    fn unmark(&mut self) {
        self.0 &= LOW_BITS
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3}", self.0)
    }
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 3;
}

/// Implementation of a bingo board.
struct Bingo(Map<Tile>);

impl Bingo {
    fn call(&mut self, value: u8) {
        for (_, tile) in self.0.iter_mut() {
            if tile.value() == value {
                tile.mark();
            }
        }
    }

    /// `true` when the board contains at least one marked row of bingos.
    fn check_bingo(&self) -> bool {
        let (dx, dy) = Direction::Up.deltas();
        let left_edge = self.0.project(self.0.bottom_left(), dx, dy);
        let (dx, dy) = Direction::Right.deltas();
        let horizontal_rows = left_edge.map(|left| self.0.project(left, dx, dy));
        let bottom_edge = self.0.project(self.0.bottom_left(), dx, dy);
        let (dx, dy) = Direction::Up.deltas();
        let vertical_rows = bottom_edge.map(|bottom| self.0.project(bottom, dx, dy));
        let mut rows = horizontal_rows.chain(vertical_rows);

        rows.any(|mut row| row.all(|tile| self.0[tile].is_marked()))
    }

    fn sum_unmarked(&self) -> u32 {
        self.0
            .iter()
            .filter_map(|(_, tile)| (!tile.is_marked()).then(|| tile.value() as u32))
            .sum()
    }
}

impl FromStr for Bingo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = Map::<Tile>::new(5, 5);
        let mut y = map.high_y();

        for line in s.trim_end().lines() {
            if y < 0 {
                return Err(Error::BadBoard);
            }

            let values: Vec<u8> = line
                .split_ascii_whitespace()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map_err(|_| Error::BadBoard)?;

            if values.len() != 5 {
                return Err(Error::BadBoard);
            }

            for (x, value) in values.iter().enumerate() {
                map[(x, y as usize)] = Tile(*value);
            }

            y -= 1;
        }

        Ok(Bingo(map))
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (calls, boards) = parse_two_phase::<TrimmedCommaSep<u8>, Bingo>(input)?;
    let calls: Vec<_> = calls.into();
    let mut boards: Vec<_> = boards.collect();

    for call in calls {
        for board in boards.iter_mut() {
            board.call(call);
            if board.check_bingo() {
                println!("winning score: {}", board.sum_unmarked() * call as u32);
                return Ok(());
            }
        }
    }

    Err(Error::NoSolution)
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] two_phase::TwoPhaseError),
    #[error("no solution found")]
    NoSolution,
    #[error("bad board")]
    BadBoard,
}
