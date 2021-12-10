use aoclib::parse;
use std::path::Path;

const ILLEGAL_PAREN: u32 = 3;
const ILLEGAL_SQUARE: u32 = 57;
const ILLEGAL_CURLY: u32 = 1197;
const ILLEGAL_ANGLE: u32 = 25137;

#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::Display)]
pub enum Bracket {
    #[display("()")]
    Paren,
    #[display("[]")]
    Square,
    #[display("{}")]
    Curly,
    #[display("<>")]
    Angle,
}

impl Bracket {
    fn penalty(self) -> u32 {
        match self {
            Bracket::Paren => ILLEGAL_PAREN,
            Bracket::Square => ILLEGAL_SQUARE,
            Bracket::Curly => ILLEGAL_CURLY,
            Bracket::Angle => ILLEGAL_ANGLE,
        }
    }
}

pub type Stack = Vec<Bracket>;

fn process_bracket(stack: &mut Stack, ch: char) -> Result<(), Error> {
    let mut popped = None;
    match ch {
        '(' => stack.push(Bracket::Paren),
        '[' => stack.push(Bracket::Square),
        '{' => stack.push(Bracket::Curly),
        '<' => stack.push(Bracket::Angle),
        ')' => popped = Some((Bracket::Paren, stack.pop())),
        ']' => popped = Some((Bracket::Square, stack.pop())),
        '}' => popped = Some((Bracket::Curly, stack.pop())),
        '>' => popped = Some((Bracket::Angle, stack.pop())),
        _ => return Err(Error::NotABracket(ch)),
    }

    match popped {
        None => Ok(()),
        Some((bracket, Some(expect))) if bracket == expect => Ok(()),
        Some((bracket, _)) => Err(Error::Corrupted(bracket)),
    }
}

/// Process a line of input.
///
/// Returns the penalty, if any.
fn process_corrupted_line((line_no, line): (usize, String)) -> Option<u32> {
    let mut stack = Stack::new();
    for ch in line.chars() {
        match process_bracket(&mut stack, ch) {
            Ok(()) => {}
            Err(Error::Corrupted(bracket)) => return Some(bracket.penalty()),
            Err(Error::NotABracket(ch)) => {
                eprintln!(
                    "Line {}: {:?} is not a bracket. Don't trust the results!",
                    line_no + 1,
                    ch
                );
                return None;
            }
            _ => unreachable!(),
        }
    }
    None
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let score = parse::<String>(input)?
        .enumerate()
        .filter_map(process_corrupted_line)
        .sum::<u32>();

    println!("syntax err score: {}", score);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("not a bracket: {0}")]
    NotABracket(char),
    #[error("wrong close: {0}")]
    Corrupted(Bracket),
    #[error("no solution found")]
    NoSolution,
}
