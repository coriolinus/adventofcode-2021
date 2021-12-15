use std::{
    cmp::{Ordering, Reverse},
    collections::{binary_heap::BinaryHeap, HashSet},
    path::Path,
    rc::Rc,
};

use aoclib::geometry::{tile::Digit, Point};

type Map = aoclib::geometry::Map<Digit>;

#[derive(Debug, PartialEq, Eq, Default)]
struct HeapNode {
    position: Point,
    total_risk: u64,
    previous: Option<Rc<HeapNode>>,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_risk
            .cmp(&other.total_risk)
            .then_with(|| self.position.cmp(&other.position))
            .then_with(|| self.previous.cmp(&other.previous))
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;

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
        let node = Rc::new(node);
        if node.position == map.bottom_right() {
            println!("total risk: {}", node.total_risk);
            break;
        }
        visited.insert(node.position);
        for adjacent in map.orthogonal_adjacencies(node.position) {
            if !visited.contains(&adjacent) {
                heap.push(Reverse(HeapNode {
                    position: adjacent,
                    total_risk: node.total_risk + <Digit as Into<u8>>::into(map[adjacent]) as u64,
                    previous: Some(node.clone()),
                }));
            }
        }
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
