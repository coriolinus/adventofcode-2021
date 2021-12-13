use aoclib::{geometry::Point, parse};
use std::{cmp::Ordering, collections::HashSet, path::Path};

#[derive(Debug, Clone, Copy, parse_display::FromStr)]
enum Axis {
    #[display("x")]
    X,
    #[display("y")]
    Y,
}

#[derive(parse_display::FromStr)]
enum Instruction {
    #[display("{0},{1}")]
    Dot(i32, i32),
    #[display("fold along {0}={1}")]
    Fold(Axis, i32),
    #[display("")]
    Blank,
}

#[derive(Debug, Clone, Copy)]
struct Fold {
    axis: Axis,
    offset: i32,
}

impl Fold {
    fn apply(self, mut point: Point) -> Point {
        let scalar = match self.axis {
            Axis::X => &mut point.x,
            Axis::Y => &mut point.y,
        };
        match (*scalar).cmp(&self.offset) {
            Ordering::Less => {
                // no change
            }
            Ordering::Equal => {
                eprintln!("folding {:?} over {:?}, point on fold line", point, self);
            }
            Ordering::Greater => {
                // reflect the point
                let diff = *scalar - self.offset;
                debug_assert!(diff > 0);
                *scalar -= 2 * diff;
            }
        }
        point
    }
}

fn parse_input(input: &Path) -> Result<(Vec<Point>, Vec<Fold>), Error> {
    let mut points = Vec::new();
    let mut folds = Vec::new();

    for instruction in parse::<Instruction>(input)? {
        match instruction {
            Instruction::Dot(x, y) => points.push(Point::new(x, y)),
            Instruction::Fold(axis, offset) => folds.push(Fold { axis, offset }),
            Instruction::Blank => {}
        }
    }

    Ok((points, folds))
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (points, folds) = parse_input(input)?;
    let first_fold = *folds.first().ok_or(Error::NoSolution)?;
    let mut point_collection = HashSet::with_capacity(points.len());
    for point in points {
        point_collection.insert(first_fold.apply(point));
    }
    println!("{} points after first fold", point_collection.len());
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
