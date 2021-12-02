use aoclib::parse;
use itertools::Itertools;
use std::path::Path;

trait CountIncreases {
    fn count_increases(self) -> usize;
}

impl<Iter, Item> CountIncreases for Iter
where
    Iter: Iterator<Item = Item>,
    Item: PartialOrd + Clone,
{
    fn count_increases(self) -> usize {
        self.tuple_windows::<(_, _)>()
            .filter(|(a, b)| b > a)
            .count()
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let increases = parse::<u32>(input)?.count_increases();
    println!("increases: {}", increases);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let increases = parse::<u32>(input)?
        .tuple_windows::<(_, _, _)>()
        .map(|(a, b, c)| a + b + c)
        .count_increases();
    println!("increases (3-windows): {}", increases);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
