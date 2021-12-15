use std::{
    cmp::{Ordering, Reverse},
    collections::{binary_heap::BinaryHeap, HashSet},
    path::Path,
};

use aoclib::geometry::{tile::Digit, Map, Point};

#[derive(Debug, PartialEq, Eq, Default)]
struct HeapNode {
    position: Point,
    total_risk: u64,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_risk
            .cmp(&other.total_risk)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn find_lowest_risk_path_top_left_to_bottom_right(map: &Map<u8>) -> u64 {
    let mut visited = HashSet::new();
    let mut heap = BinaryHeap::new();

    heap.push(Reverse(HeapNode {
        position: map.top_left(),
        ..HeapNode::default()
    }));
    while let Some(Reverse(node)) = heap.pop() {
        if visited.contains(&node.position) {
            continue;
        }
        if node.position == map.bottom_right() {
            return node.total_risk;
        }
        visited.insert(node.position);
        for adjacent in map.orthogonal_adjacencies(node.position) {
            if !visited.contains(&adjacent) {
                heap.push(Reverse(HeapNode {
                    position: adjacent,
                    total_risk: node.total_risk + map[adjacent] as u64,
                }));
            }
        }
    }

    unreachable!("every map has _some_ traversable path")
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map<Digit> as TryFrom<&Path>>::try_from(input)?;
    let map: Map<u8> = map.convert_tile_type();
    let total_risk = find_lowest_risk_path_top_left_to_bottom_right(&map);
    println!("total risk (small map): {}", total_risk);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let map = {
        let small_map = <Map<Digit> as TryFrom<&Path>>::try_from(input)?;
        let small_map: Map<u8> = small_map.convert_tile_type();
        let small_map = small_map.flip_vertical();

        let mut map = Map::new(small_map.width() * 5, small_map.height() * 5);
        for (point, tile) in map.iter_mut() {
            let increase = (point.x as usize / small_map.width()
                + point.y as usize / small_map.height()) as u8;
            *tile = (small_map[(
                point.x as usize % small_map.width(),
                point.y as usize % small_map.height(),
            )] + increase
                - 1)
                % 9
                + 1;
            if *tile == 0 {
                *tile = 1;
            }
        }
        map.flip_vertical()
    };
    // {
    //     let mut dmap = Map::<Digit>::new(map.width(), map.height());
    //     for (point, tile) in dmap.iter_mut() {
    //         *tile = map[point].to_string().parse().unwrap();
    //     }
    //     eprintln!("{}", dmap);
    // }
    let total_risk = find_lowest_risk_path_top_left_to_bottom_right(&map);
    println!("total risk (big map): {}", total_risk);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
