use aoclib::parse;
use std::{cmp::Ordering, path::Path, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct DiagnosticCondition {
    value: u16,
    width: usize,
}

impl FromStr for DiagnosticCondition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u16::from_str_radix(s, 2)
            .map_err(Into::into)
            .map(|value| DiagnosticCondition {
                value,
                width: s.len(),
            })
    }
}

/// Finds the `(gamma, epsilon)` rates from a diagnostic report
fn find_rates(report: &[DiagnosticCondition]) -> (u16, u16) {
    let threshold = report.len() / 2;

    let mut counts = [0_usize; 16];
    let mut max_width = 0;

    for condition in report.iter() {
        for position in 0..16 {
            if condition.value & 1 << position != 0 {
                counts[position] += 1;
            }
        }
        max_width = max_width.max(condition.width);
    }

    let mut gamma = 0;
    for position in 0..16 {
        if counts[position] > threshold {
            gamma |= 1 << position;
        }
    }

    let mut epsilon = !gamma;
    for unset_position in max_width..16 {
        epsilon &= !(1 << unset_position);
    }

    (gamma, epsilon)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifeSupport {
    OxygenGenerator,
    Co2Scrubber,
}

fn locate_rating(report: &[DiagnosticCondition], rating_type: LifeSupport) -> Result<u16, Error> {
    let max_width = report
        .iter()
        .map(|condition| condition.width)
        .max()
        .ok_or(Error::NoSolution)?;

    let mut possible_values: Vec<_> = report.iter().map(|condition| condition.value).collect();

    for position in (0..max_width).rev() {
        let ones = possible_values
            .iter()
            .filter(|value| *value & 1 << position != 0)
            .count();
        let zeros = possible_values.len() - ones;
        let desired_value = match (rating_type, zeros.cmp(&ones)) {
            (LifeSupport::OxygenGenerator, Ordering::Less | Ordering::Equal) => 1,
            (LifeSupport::OxygenGenerator, Ordering::Greater) => 0,
            (LifeSupport::Co2Scrubber, Ordering::Less | Ordering::Equal) => 0,
            (LifeSupport::Co2Scrubber, Ordering::Greater) => 1,
        };

        possible_values.retain(|value| value & 1 << position == desired_value << position);
        if possible_values.is_empty() {
            return Err(Error::NoSolution);
        }
        if possible_values.len() == 1 {
            return Ok(possible_values[0]);
        }
    }
    return Err(Error::NoSolution);
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let diagnostic_report: Vec<DiagnosticCondition> = parse(input)?.collect();
    let (gamma, epsilon) = find_rates(&diagnostic_report);
    println!("power consumption: {}", gamma as u32 * epsilon as u32);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let diagnostic_report: Vec<DiagnosticCondition> = parse(input)?.collect();
    let oxygen_generator_rating = locate_rating(&diagnostic_report, LifeSupport::OxygenGenerator)?;
    let co2_scrubber_rating = locate_rating(&diagnostic_report, LifeSupport::Co2Scrubber)?;
    println!(
        "life support rating: {}",
        oxygen_generator_rating as u32 * co2_scrubber_rating as u32
    );
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parsing diagnostic condition")]
    ParseDiagnosticCondition(#[from] std::num::ParseIntError),
    #[error("No solution found")]
    NoSolution,
}
