use aoclib::parse;
use itertools::Itertools;
use std::path::Path;

fn count_increases(iter: impl Iterator<Item = u32>) -> usize {
    iter.tuple_windows::<(_, _)>()
        .filter(|(a, b)| b > a)
        .count()
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let increases = count_increases(parse::<u32>(input)?);
    println!("increases: {}", increases);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let increases = count_increases(
        parse::<u32>(input)?
            .tuple_windows::<(_, _, _)>()
            .map(|(a, b, c)| a + b + c),
    );
    println!("increases (3-windows): {}", increases);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
