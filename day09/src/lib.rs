use aoclib::geometry::{
    map::{ContextInto, Traversable},
    tile::DisplayWidth,
    Point,
};
use std::path::Path;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::FromStr)]
struct Digit(aoclib::geometry::map::tile::Digit);

impl DisplayWidth for Digit {
    const DISPLAY_WIDTH: usize = aoclib::geometry::map::tile::Digit::DISPLAY_WIDTH;
}

impl From<Digit> for u8 {
    fn from(digit: Digit) -> u8 {
        digit.0.into()
    }
}

impl ContextInto<Traversable> for Digit {
    type Context = ();

    fn ctx_into(self, _position: Point, _context: &Self::Context) -> Traversable {
        match self.into() {
            9 => Traversable::Obstructed,
            _ => Traversable::Free,
        }
    }
}

type Map = aoclib::geometry::Map<Digit>;

fn read_input(input: &Path) -> Result<(Map, Vec<Point>), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;
    let low_points = map
        .iter()
        .filter(|(point, height)| {
            map.orthogonal_adjacencies(*point)
                .all(|adj| map[adj] > **height)
        })
        .map(|(point, _)| point)
        .collect();
    Ok((map, low_points))
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (map, low_points) = read_input(input)?;
    let risk_level = low_points
        .iter()
        .map(|point| <Digit as Into<u8>>::into(map[*point]) as u32 + 1)
        .sum::<u32>();

    println!("sum of low point risk levels: {}", risk_level);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let (map, low_points) = read_input(input)?;
    let mut region_sizes: Vec<_> = low_points
        .iter()
        .map(|point| {
            let mut size: u64 = 0;
            map.reachable_from(*point, |_point, _tile| {
                size += 1;
                false
            });
            size
        })
        .collect();
    region_sizes.sort_unstable();
    let basin_size_product: u64 = region_sizes.iter().rev().take(3).product();

    println!("product of 3 largest basin sizes: {}", basin_size_product);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("could not read map")]
    MapConv(#[from] aoclib::geometry::map::MapConversionErr),
}
