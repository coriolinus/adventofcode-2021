use aoclib::parse;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::Display)]
pub enum Bracket {
    #[display(")")]
    Paren,
    #[display("]")]
    Square,
    #[display("}}")]
    Curly,
    #[display(">")]
    Angle,
}

impl Bracket {
    fn corruption_penalty(self) -> u32 {
        match self {
            Bracket::Paren => 3,
            Bracket::Square => 57,
            Bracket::Curly => 1197,
            Bracket::Angle => 25137,
        }
    }

    fn autocomplate_score(self) -> u64 {
        match self {
            Bracket::Paren => 1,
            Bracket::Square => 2,
            Bracket::Curly => 3,
            Bracket::Angle => 4,
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
/// Returns the stack if the line is incomplete, or the penalty if it is corrupted.
fn process_line(line_no: usize, line: String) -> Result<Stack, u32> {
    let mut stack = Stack::new();
    for ch in line.chars() {
        match process_bracket(&mut stack, ch) {
            Ok(()) => {}
            Err(Error::Corrupted(bracket)) => return Err(bracket.corruption_penalty()),
            Err(Error::NotABracket(ch)) => {
                eprintln!(
                    "Line {}: {:?} is not a bracket. Don't trust the results!",
                    line_no + 1,
                    ch
                );
                return Err(0);
            }
            _ => unreachable!(),
        }
    }
    Ok(stack)
}

fn score_stack(stack: Stack) -> u64 {
    let mut score = 0;
    for closer in stack.into_iter().rev() {
        score *= 5;
        score += closer.autocomplate_score();
    }
    score
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let score = parse::<String>(input)?
        .enumerate()
        .filter_map(|(line_no, line)| process_line(line_no, line).err())
        .sum::<u32>();

    println!("syntax err score: {}", score);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut scores: Vec<_> = parse::<String>(input)?
        .enumerate()
        .filter_map(|(line_no, line)| process_line(line_no, line).ok().map(score_stack))
        .collect();
    scores.sort_unstable();
    let middle_score = scores[scores.len() / 2];
    println!("median autocomplete score: {}", middle_score);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("not a bracket: {0}")]
    NotABracket(char),
    #[error("wrong close: {0}")]
    Corrupted(Bracket),
}
