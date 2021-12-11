use aoclib::geometry::{tile::Digit, Map};
use std::path::Path;

/// Advance a map's state, returning the number of flashes.
fn step(map: &mut Map<u8>) -> u64 {
    let mut flash_map = Map::<bool>::new(map.width(), map.height());

    for (_, tile) in map.iter_mut() {
        *tile += 1;
    }

    // loop until no new flashes
    loop {
        let mut new_flashes = 0;
        for point in map.points() {
            if map[point] > 9 && !flash_map[point] {
                new_flashes += 1;
                flash_map[point] = true;
                for adjacency in map.make_adjacencies(point) {
                    map[adjacency] += 1;
                }
            }
        }
        if new_flashes == 0 {
            break;
        }
    }

    let mut flashes = 0;
    for (point, flashed) in flash_map.iter() {
        if *flashed {
            flashes += 1;
            map[point] = 0;
        }
    }

    flashes
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map<Digit> as TryFrom<&Path>>::try_from(input)?;
    let mut map: Map<u8> = map.convert_tile_type();
    let mut flashes = 0;

    for _ in 0..100 {
        flashes += step(&mut map);
    }

    println!("flashes after 100 steps: {}", flashes);

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
