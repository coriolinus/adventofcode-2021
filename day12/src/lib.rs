use aoclib::parse;
use parse_display::FromStr;
use std::path::Path;

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

fn parse_input(input: &Path) -> Result<(Vec<Cave>, Edges), Error> {
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
    Ok((caves, edges))
}

pub fn part1(input: &Path) -> Result<(), Error> {
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
