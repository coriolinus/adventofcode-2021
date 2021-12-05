use aoclib::parse;
use std::path::Path;

#[derive(Debug, parse_display::FromStr, parse_display::Display)]
#[display("{x1},{y1} -> {x2}{y2}")]
#[from_str(regex = r"(?P<x1>\d+),(?P<y1>\d+)\s+->\s+(?P<x2>\d+),(?P<y2>\d+)")]
struct VentLine {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut x_min = !0;
    let mut y_min = !0;
    let mut x_max = 0;
    let mut y_max = 0;

    let mut count = 0;

    for vent_line in parse::<VentLine>(input)? {
        x_min = x_min.min(vent_line.x1);
        x_min = x_min.min(vent_line.x2);
        x_max = x_max.max(vent_line.x1);
        x_max = x_max.max(vent_line.x2);
        y_min = y_min.min(vent_line.y1);
        y_min = y_min.min(vent_line.y2);
        y_max = y_max.max(vent_line.y1);
        y_max = y_max.max(vent_line.y2);

        count += 1;
    }

    dbg!(x_min, x_max, y_min, y_max, count);
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}
