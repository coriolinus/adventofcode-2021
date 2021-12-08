use aoclib::parse;
use std::{path::Path, str::FromStr};

/// A pattern of signals intended to control a 7-segment display.
///
/// Each signal is meant to control a single segment of the display.
/// The problem is in determining which controls which.
///
/// Each signal letter is represented by a single bit of a u8.
/// 'a' corresponds to the least significant bit, and 'g' to `1 << 6`.
/// The most significant bit is unused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct Signals(u8);

impl FromStr for Signals {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut presence = 0;
        for ch in s.chars() {
            let idx = (ch as u8)
                .checked_sub(b'a')
                .ok_or(Error::InvalidLetter(ch))?;
            if idx > 7 {
                return Err(Error::InvalidLetter(ch));
            }
            let idx = 1 << idx;
            if presence & idx != 0 {
                return Err(Error::DuplicateLetter(ch));
            }
            presence |= idx;
        }
        Ok(Signals(presence))
    }
}

impl Signals {
    fn segment_count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// An entry in the problem input.
///
/// It consists of 10 unique signal patterns, and four output digits.
#[derive(Debug, Default, Clone, Copy)]
struct Entry {
    signal_patterns: [Signals; 10],
    output_value: [Signals; 4],
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut e = Entry::default();

        for (idx, token) in s.split_whitespace().enumerate() {
            match idx {
                0..=9 => e.signal_patterns[idx] = token.parse()?,
                10 if token == "|" => {}
                11..=14 => e.output_value[idx - 11] = token.parse()?,
                _ => return Err(Error::MalformedEntry),
            }
        }

        if e.signal_patterns
            .iter()
            .chain(e.output_value.iter())
            .any(|signals| signals.segment_count() == 0)
        {
            return Err(Error::MalformedEntry);
        }

        Ok(e)
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let output_identifiable_digits_count = parse::<Entry>(input)?
        .flat_map(|entry| entry.output_value.into_iter())
        .filter(|signals| match signals.segment_count() {
            2 | 3 | 4 | 7 => true,
            _ => false,
        })
        .count();
    println!(
        "identifiable output digits: {}",
        output_identifiable_digits_count
    );
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("duplicate letter in signal ({0})")]
    DuplicateLetter(char),
    #[error("invalid letter in signal ({0})")]
    InvalidLetter(char),
    #[error("malformed entry")]
    MalformedEntry,
    #[error("no solution found")]
    NoSolution,
}
