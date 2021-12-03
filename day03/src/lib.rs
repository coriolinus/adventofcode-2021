use aoclib::parse;
use std::{path::Path, str::FromStr};

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

// known wrong: 16449928
pub fn part1(input: &Path) -> Result<(), Error> {
    let diagnostic_report: Vec<DiagnosticCondition> = parse(input)?.collect();
    let (gamma, epsilon) = find_rates(&diagnostic_report);
    println!("power consumption: {}", gamma as u32 * epsilon as u32);
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
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
