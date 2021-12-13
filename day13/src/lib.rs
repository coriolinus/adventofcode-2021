use aoclib::{
    geometry::{tile::Bool, Map, Point},
    parse,
};
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

type DisplayBoard = Map<Bool>;

pub fn part2(input: &Path) -> Result<(), Error> {
    let (points, folds) = parse_input(input)?;
    let mut point_collection = HashSet::with_capacity(points.len());
    for mut point in points {
        for fold in &folds {
            point = fold.apply(point);
        }
        point_collection.insert(point);
    }

    // determine the max and min points
    let mut max = Point::new(i32::MIN, i32::MIN);
    let mut min = Point::new(i32::MAX, i32::MAX);
    for point in &point_collection {
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
    }

    let mut board = DisplayBoard::new_offset(
        min,
        (max.x - min.x + 1) as usize,
        (max.y - min.y + 1) as usize,
    );
    for point in point_collection {
        board[point] = true.into();
    }
    board = board.flip_vertical();

    println!("activation code:\n{}", board);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
