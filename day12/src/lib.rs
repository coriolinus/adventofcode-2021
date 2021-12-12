use aoclib::parse;
use bitvec::prelude::*;
use std::{collections::VecDeque, path::Path};

#[derive(parse_display::FromStr)]
#[display("{from}-{to}")]
struct PrimitiveEdge {
    from: String,
    to: String,
}

struct Cave {
    label: String,
    is_big: bool,
}

// Edges are a map from every cave (by index) to the indices of every cave directly reachable therefrom.
type Edges = std::collections::HashMap<usize, Vec<usize>>;

/// Parse the input file into four fields:
///
/// - a list of caves
/// - an Edges map of all outgoing nodes from the current node
/// - a 2-tuple:
///   - the index of the start cave in the caves list
///   - the index of the end cave in the caves list
fn parse_input(input: &Path) -> Result<(Vec<Cave>, Edges, (usize, usize)), Error> {
    let prim_edges: Vec<_> = parse::<PrimitiveEdge>(input)?.collect();
    let mut labels = Vec::with_capacity(prim_edges.len() * 2);
    for pe in prim_edges.iter() {
        labels.push(pe.from.clone());
        labels.push(pe.to.clone());
    }
    labels.sort_unstable();
    labels.dedup();

    let mut caves = Vec::with_capacity(labels.len());
    for label in labels {
        caves.push(Cave {
            is_big: label.chars().any(|ch| ch.is_ascii_uppercase()),
            label,
        })
    }

    let index_of = |label: &str| -> usize {
        caves
            .iter()
            .enumerate()
            .find(|(_, cave)| cave.label == label)
            .expect("all labels derive from primitive edges")
            .0
    };

    let mut edges = Edges::with_capacity(caves.len());
    for pe in prim_edges {
        for (from, to) in [(&pe.from, &pe.to), (&pe.to, &pe.from)] {
            edges.entry(index_of(from)).or_default().push(index_of(to));
        }
    }

    let start = index_of("start");
    let end = index_of("end");

    Ok((caves, edges, (start, end)))
}

struct SearchNode {
    location: usize,
    visited: BitVec,
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (caves, edges, (start, end)) = parse_input(input)?;

    let mut queue = VecDeque::new();
    queue.push_back(SearchNode {
        location: start,
        visited: bitvec![0; caves.len()],
    });

    let mut paths = 0;
    while let Some(SearchNode {
        location,
        mut visited,
    }) = queue.pop_front()
    {
        visited.set(location, true);
        if location == end {
            paths += 1;
        } else {
            for next_location in edges
                .get(&location)
                .map(|locations| {
                    Box::new(locations.iter().copied()) as Box<dyn Iterator<Item = usize>>
                })
                .unwrap_or(Box::new(std::iter::empty()))
            {
                if caves[next_location].is_big || !visited[next_location] {
                    queue.push_back(SearchNode {
                        location: next_location,
                        visited: visited.clone(),
                    });
                }
            }
        }
    }

    println!("distinct paths through the cave system: {}", paths);
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
