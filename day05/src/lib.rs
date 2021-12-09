use aoclib::{
    geometry::{line::Line, Point},
    parse,
};
use std::path::Path;

const MAP_EDGE: usize = 1024;

#[derive(Debug, parse_display::FromStr, parse_display::Display, Clone, Copy)]
#[display("{x1},{y1} -> {x2}{y2}")]
#[from_str(regex = r"(?P<x1>\d+),(?P<y1>\d+)\s+->\s+(?P<x2>\d+),(?P<y2>\d+)")]
struct VentLine {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl From<VentLine> for Line {
    fn from(vl: VentLine) -> Self {
        Line::new(
            Point::new(vl.x1 as i32, vl.y1 as i32),
            Point::new(vl.x2 as i32, vl.y2 as i32),
        )
    }
}

fn is_horizontal_or_vertical(line: &Line) -> bool {
    line.from.x == line.to.x || line.from.y == line.to.y
}

/// Iterate over the points of the line, inclusive.
///
/// Only works for horizontal, vertical, or perfect diagonal lines.
/// Other angles will cause infinite incorrect iteration.
///
/// Consider adding this to aoclib.
fn line_points(line: Line) -> impl Iterator<Item = Point> {
    let vector = line.to - line.from;
    let dx = vector.x / vector.x.abs().max(1);
    let dy = vector.y / vector.y.abs().max(1);

    std::iter::successors(Some(line.from), move |prev| {
        if *prev == line.to {
            None
        } else {
            let mut next = *prev;
            next.x += dx;
            next.y += dy;
            Some(next)
        }
    })
    .fuse()
}

type Map = aoclib::geometry::Map<u8>;

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut map = Map::new(MAP_EDGE, MAP_EDGE);
    for point in parse::<VentLine>(input)?
        .map(Into::into)
        .filter(is_horizontal_or_vertical)
        .map(line_points)
        .flatten()
    {
        map[point] += 1;
    }

    let intersections_count = map.iter().filter(|(_, count)| **count > 1).count();

    println!(
        "count of intersections (horiz or vert): {}",
        intersections_count
    );

    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut map = Map::new(MAP_EDGE, MAP_EDGE);
    for point in parse::<VentLine>(input)?
        .map(Into::into)
        .map(line_points)
        .flatten()
    {
        map[point] += 1;
    }

    let intersections_count = map.iter().filter(|(_, count)| **count > 1).count();

    println!("count of intersections (all): {}", intersections_count);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
