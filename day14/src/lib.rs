use std::{collections::HashMap, path::Path, str::FromStr};

#[derive(Debug, Clone, Copy)]
struct InsertionRule {
    after: char,
    before: char,
    insert: char,
}

impl FromStr for InsertionRule {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut after = None;
        let mut before = None;
        let mut insert = None;

        for (idx, ch) in s.chars().enumerate() {
            match idx {
                0 if ch.is_ascii_uppercase() => after = Some(ch),
                1 if ch.is_ascii_uppercase() => before = Some(ch),
                2..=5 if b" -> "[idx - 2] == ch as u8 => {}
                6 if ch.is_ascii_uppercase() => insert = Some(ch),
                _ => return Err(Error::MalformedInput),
            }
        }

        Ok(InsertionRule {
            after: after.ok_or(Error::MalformedInput)?,
            before: before.ok_or(Error::MalformedInput)?,
            insert: insert.ok_or(Error::MalformedInput)?,
        })
    }
}

struct PairTable {
    first_letter: char,
    last_letter: char,
    pairs: HashMap<(char, char), u64>,
}

impl FromStr for PairTable {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.chars().all(|ch| ch.is_ascii_uppercase()) {
            return Err(Error::MalformedInput);
        }
        if s.is_empty() {
            return Err(Error::MalformedInput);
        }

        let first_letter = s.as_bytes()[0] as char;
        let last_letter = s.as_bytes()[s.len() - 1] as char;
        let mut pairs = HashMap::new();
        for window in s.as_bytes().windows(2) {
            let after = window[0] as char;
            let before = window[1] as char;
            *pairs.entry((after, before)).or_default() += 1;
        }

        Ok(PairTable {
            first_letter,
            last_letter,
            pairs,
        })
    }
}

impl PairTable {
    fn apply(self, rules: &[InsertionRule]) -> PairTable {
        let PairTable {
            first_letter,
            last_letter,
            mut pairs,
        } = self;
        let mut next_pairs = HashMap::with_capacity(pairs.len() * 2);

        // first apply each rule
        for rule in rules {
            if let Some(existing) = pairs.remove(&(rule.after, rule.before)) {
                *next_pairs.entry((rule.after, rule.insert)).or_default() += existing;
                *next_pairs.entry((rule.insert, rule.before)).or_default() += existing;
            }
        }

        // now carry over each remaining pair which wasn't in the rules set
        next_pairs.extend(pairs.drain());

        PairTable {
            first_letter,
            last_letter,
            pairs: next_pairs,
        }
    }

    fn element_quantities(&self) -> HashMap<char, u64> {
        let mut qty = HashMap::new();
        // only these two letters don't already appear twice in the input
        qty.insert(self.first_letter, 1);
        qty.insert(self.last_letter, 1);
        // as all pairs contain two letters, each letter is counted twice
        for ((first, second), q) in self.pairs.iter() {
            *qty.entry(*first).or_default() += *q;
            *qty.entry(*second).or_default() += *q;
        }
        // as everything is counted twice, halve it all
        for (_, v) in qty.iter_mut() {
            *v /= 2;
        }
        qty
    }

    fn puzzle_solution(&self) -> u64 {
        let quantities = self.element_quantities();
        let mut quantities: Vec<_> = quantities.values().collect();
        if quantities.len() == 0 {
            return 0;
        }
        quantities.sort_unstable();
        **quantities.last().unwrap() - **quantities.first().unwrap()
    }
}

fn parse_input(input: &Path) -> Result<(PairTable, Vec<InsertionRule>), Error> {
    use aoclib::input::{parse_newline_sep, parse_str};

    let mut sections = parse_newline_sep::<String>(input)?;
    let polymer_template = sections.next().ok_or(Error::MalformedInput)?;
    let polymer_template = polymer_template.trim().parse()?;

    let insertion_rules = sections.next().ok_or(Error::MalformedInput)?;
    let insertion_rules: Vec<InsertionRule> = parse_str(&insertion_rules)?.collect();

    if sections.next().is_some() {
        return Err(Error::MalformedInput);
    }

    Ok((polymer_template, insertion_rules))
}

fn solve(input: &Path, iterations: u8) -> Result<(), Error> {
    let (mut pair_table, insertion_rules) = parse_input(input)?;
    for _ in 0..iterations {
        pair_table = pair_table.apply(&insertion_rules);
    }
    let solution = pair_table.puzzle_solution();

    println!("part 1 solution: {}", solution);
    Ok(())
}

pub fn part1(input: &Path) -> Result<(), Error> {
    solve(input, 10)
}

pub fn part2(input: &Path) -> Result<(), Error> {
    solve(input, 40)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("malformed input")]
    MalformedInput,
    #[error("no solution found")]
    NoSolution,
}
