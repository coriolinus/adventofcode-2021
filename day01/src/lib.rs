use aoclib::parse;
use itertools::Itertools;
use std::path::Path;

pub fn part1(input: &Path) -> Result<(), Error> {
    let increases = parse::<u32>(input)?
        .tuple_windows::<(_, _)>()
        .filter(|(a, b)| b > a)
        .count();
    println!("increases: {}", increases);
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}
