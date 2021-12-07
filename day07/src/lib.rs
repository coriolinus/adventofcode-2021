use aoclib::{parse, CommaSep};
use std::path::Path;

#[cfg(feature = "parallelism")]
use rayon::prelude::*;

fn total_fuel_at_best_position(
    crab_submarines: &[i32],
    fuel_per_submarine: impl Sync + Fn(i32) -> i32,
) -> Option<i32> {
    let min = *crab_submarines.iter().min()?;
    let max = *crab_submarines.iter().max()?;
    #[cfg(not(feature = "parallelism"))]
    let range = min..=max;
    #[cfg(feature = "parallelism")]
    let range = (min..=max).into_par_iter();

    range
        .map(|assembly_point| {
            crab_submarines
                .iter()
                .copied()
                .map(|submarine| fuel_per_submarine((submarine - assembly_point).abs()))
                .sum::<i32>()
        })
        .min()
}

/// The triangular numbers compute the fuel used by a crab submarine moving distance `n`.
///
/// See <https://oeis.org/A000217>.
fn triangular_sequence(n: i32) -> i32 {
    n * (n + 1) / 2
}

fn solve(
    input: &Path,
    nature: &str,
    fuel_per_submarine: impl Sync + Fn(i32) -> i32,
) -> Result<(), Error> {
    for (idx, line) in parse::<CommaSep<i32>>(input)?.enumerate() {
        let crab_submarines: Vec<_> = line.into();
        let total_fuel_at_best_position =
            total_fuel_at_best_position(&crab_submarines, &fuel_per_submarine)
                .ok_or(Error::NoSolution)?;
        println!(
            "{}: total fuel at best position ({}): {}",
            idx, nature, total_fuel_at_best_position
        )
    }
    Ok(())
}

pub fn part1(input: &Path) -> Result<(), Error> {
    solve(input, "linear", std::convert::identity)
}

pub fn part2(input: &Path) -> Result<(), Error> {
    solve(input, "increasing rate", triangular_sequence)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
