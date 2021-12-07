use aoclib::{parse, CommaSep};
use std::path::Path;

fn total_fuel_at_best_position(
    crab_submarines: &[i32],
    per_submarine: impl Fn(i32, i32) -> i32,
) -> Option<i32> {
    let min = *crab_submarines.iter().min()?;
    let max = *crab_submarines.iter().max()?;
    (min..=max)
        .map(|assembly_point| {
            crab_submarines
                .iter()
                .copied()
                .map(|submarine| per_submarine(submarine, assembly_point))
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

fn solve(input: &Path, nature: &str, per_submarine: impl Fn(i32, i32) -> i32) -> Result<(), Error> {
    for (idx, line) in parse::<CommaSep<i32>>(input)?.enumerate() {
        let crab_submarines: Vec<_> = line.into();
        let total_fuel_at_best_position =
            total_fuel_at_best_position(&crab_submarines, &per_submarine)
                .ok_or(Error::NoSolution)?;
        println!(
            "{}: total fuel at best position ({}): {}",
            idx, nature, total_fuel_at_best_position
        )
    }
    Ok(())
}

pub fn part1(input: &Path) -> Result<(), Error> {
    solve(input, "linear", |submarine, assembly_point| {
        (submarine - assembly_point).abs()
    })
}

pub fn part2(input: &Path) -> Result<(), Error> {
    solve(input, "increasing rate", |submarine, assembly_point| {
        let n = (submarine - assembly_point).abs();
        triangular_sequence(n)
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
