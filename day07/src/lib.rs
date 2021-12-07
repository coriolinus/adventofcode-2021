use aoclib::{parse, CommaSep};
use std::path::Path;

pub fn part1(input: &Path) -> Result<(), Error> {
    for (idx, line) in parse::<CommaSep<i32>>(input)?.enumerate() {
        let crab_submarines: Vec<_> = line.into();
        let min = *crab_submarines.iter().min().ok_or(Error::NoSolution)?;
        let max = *crab_submarines.iter().max().ok_or(Error::NoSolution)?;
        dbg!(min, max);
        let total_fuel_at_best_position = (min..=max)
            .map(|assembly| {
                crab_submarines
                    .iter()
                    .copied()
                    .map(|submarine| (submarine - assembly).abs())
                    .sum::<i32>()
            })
            .min()
            .ok_or(Error::NoSolution)?;
        println!(
            "{}: total fuel at best position: {}",
            idx, total_fuel_at_best_position
        )
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
    #[error("no solution found")]
    NoSolution,
}
