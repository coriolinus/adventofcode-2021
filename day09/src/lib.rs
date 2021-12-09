use aoclib::geometry::map::tile::Digit;
use std::path::Path;

type Map = aoclib::geometry::Map<Digit>;

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;
    let low_points: Vec<_> = map
        .iter()
        .filter(|(point, height)| {
            map.orthogonal_adjacencies(*point)
                .all(|adj| map[adj] > **height)
        })
        .collect();
    let risk_level = low_points
        .iter()
        .map(|(_, digit)| <Digit as Into<u8>>::into(**digit) as u32 + 1)
        .sum::<u32>();

    println!("sum of low point risk levels: {}", risk_level);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("could not read map")]
    MapConv(#[from] aoclib::geometry::map::MapConversionErr),
    #[error("no solution found")]
    NoSolution,
}
