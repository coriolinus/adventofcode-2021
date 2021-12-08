use aoclib::parse;
use std::{path::Path, str::FromStr};

/// A `SegmentMap` maps all valid signals to outputs.
type SegmentMap = std::collections::HashMap<Pattern, u8>;

/// A pattern of signals intended to control a 7-segment display.
///
/// Each signal is meant to control a single segment of the display.
/// The problem is in determining which controls which.
///
/// Each signal letter is represented by a single bit of a u8.
/// 'a' corresponds to the least significant bit, and 'g' to `1 << 6`.
/// The most significant bit is unused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
struct Pattern(u8);

impl FromStr for Pattern {
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
        Ok(Pattern(presence))
    }
}

impl Pattern {
    fn segment_count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// An entry in the problem input.
///
/// It consists of 10 unique signal patterns, and four output digits.
#[derive(Debug, Default, Clone, Copy)]
struct Entry {
    signal_patterns: [Pattern; 10],
    output_value: [Pattern; 4],
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

impl Entry {
    fn analyze_signals(&self) -> Option<SegmentMap> {
        macro_rules! eq_or {
            ($left:expr, $right:expr, $err:literal) => {
                if $left != $right {
                    #[cfg(debug_assertions)]
                    eprintln!($err);
                    return None;
                }
            };
        }

        let mut map = SegmentMap::with_capacity(10);

        // initialize with immediately knowable signals
        let mut eight_pattern = None;
        for pattern in self.signal_patterns.iter().copied() {
            match pattern.segment_count() {
                2 => {
                    map.insert(pattern, 1);
                }
                4 => {
                    map.insert(pattern, 4);
                }
                3 => {
                    map.insert(pattern, 7);
                }
                7 => {
                    map.insert(pattern, 8);
                    eight_pattern = Some(pattern.0);
                }
                _ => {}
            };
        }

        eq_or!(
            map.len(),
            4,
            "patterns must include all digits identifiable by segment count"
        );
        let eight_pattern = eight_pattern?;

        // We can now consider segments in isolation. Segment b appears 6 times,
        // e appears 4 times, and f appears 9 times. All these appearance
        // frequencies are unique. (Segments d and g both appear 7 times, and a
        // and c appear 8 times.) This can help us identify the rest of the
        // numbers.
        let mut appearance_counts = [0_u8; 7];
        for pattern in self.signal_patterns.iter() {
            for bit_idx in 0..7 {
                if pattern.0 & 1 << bit_idx != 0 {
                    appearance_counts[bit_idx] += 1;
                }
            }
        }

        let mut segment_b = None;
        let mut segment_e = None;
        let mut segment_f = None;

        for (idx, count) in appearance_counts.into_iter().enumerate() {
            match count {
                6 => segment_b.is_none().then(|| segment_b = Some(1 << idx))?,
                4 => segment_e.is_none().then(|| segment_e = Some(1 << idx))?,
                9 => segment_f.is_none().then(|| segment_f = Some(1 << idx))?,
                7 | 8 => {}
                _ => return None,
            }
        }

        // all segments must be set, so let's simplify
        let segment_b = segment_b?;
        let segment_e = segment_e?;
        let segment_f = segment_f?;

        // we can identify 9 because it is like 8 without segment e
        let nine_pattern = Pattern(eight_pattern & !segment_e);
        debug_assert!(self.signal_patterns.contains(&nine_pattern));
        map.insert(nine_pattern, 9);

        // Considering five-segment patterns (2, 3, 5):
        //
        // - only 2 contains segment e
        // - only 5 contains segment b
        // - 3 contains segment f but not segment b
        let mut found_two = false;
        let mut found_three = false;
        let mut five_pattern = None;

        for pattern in self
            .signal_patterns
            .iter()
            .filter(|pattern| pattern.segment_count() == 5)
        {
            // two pattern
            if pattern.0 & segment_e != 0 {
                eq_or!(
                    found_two,
                    false,
                    "multiple patterns identified which might be two"
                );
                found_two = true;
                map.insert(*pattern, 2);
            }
            // five pattern
            if pattern.0 & segment_b != 0 {
                eq_or!(
                    five_pattern.is_some(),
                    false,
                    "multiple patterns identified which might be five"
                );
                five_pattern = Some(pattern.0);
                map.insert(*pattern, 5);
            }
            // three pattern
            if pattern.0 & segment_f != 0 && pattern.0 & segment_b == 0 {
                eq_or!(
                    found_three,
                    false,
                    "multiple patterns identified which might be three"
                );
                found_three = true;
                map.insert(*pattern, 3);
            }
        }

        if !found_two || !found_three {
            return None;
        }
        let five_pattern = five_pattern?;

        // Now we just need to distinguish between the 0 and 6 patterns, each of which
        // uses 6 segments.
        // That's pretty easy, though; 6 is just 5 plus segment e.
        let six_pattern = Pattern(five_pattern | segment_e);
        debug_assert!(self.signal_patterns.contains(&six_pattern));
        map.insert(six_pattern, 6);

        // By process of elimination, 0 is the last unknown pattern.
        let mut found_zero = false;
        for pattern in self.signal_patterns.iter() {
            if !map.contains_key(pattern) {
                eq_or!(
                    found_zero,
                    false,
                    "multiple patterns identified which might be zero"
                );
                found_zero = true;
                map.insert(*pattern, 0);
            }
        }

        debug_assert_eq!(map.len(), 10, "we must have identified all digits");

        Some(map)
    }

    fn output_value(&self, map: &SegmentMap) -> Option<u32> {
        let mut value = 0;
        for (position, digit_signals) in self.output_value.iter().rev().enumerate() {
            let signal_value = *map.get(digit_signals)?;
            value += signal_value as u32 * 10_u32.pow(position as u32);
        }
        Some(value)
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
    let mut output_sum = 0;
    for entry in parse::<Entry>(input)? {
        let map = entry.analyze_signals().ok_or(Error::NoSegmentMap)?;
        let value = entry.output_value(&map).ok_or(Error::UnknownSignal)?;
        output_sum += value;
    }
    println!("output sum: {}", output_sum);
    Ok(())
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
    #[error("could not calculate an appropriate segment map")]
    NoSegmentMap,
    #[error("signal not found in segment map")]
    UnknownSignal,
    #[error("no solution found")]
    NoSolution,
}
