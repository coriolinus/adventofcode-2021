use aoclib::geometry::Point;
use aoclib::parse;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, parse_display::Display, parse_display::FromStr)]
#[display("{} {0}", style = "lowercase")]
pub enum Command {
    Forward(i32),
    Down(i32),
    Up(i32),
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let final_position =
        parse::<Command>(input)?.fold(Point::default(), |mut position, command| {
            match command {
                Command::Forward(x) => position.x += x,
                Command::Down(y) => position.y += y,
                Command::Up(y) => position.y -= y,
            }
            position
        });
    println!(
        "product of horizontal position and depth: {}",
        final_position.x * final_position.y
    );
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
