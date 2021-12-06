use aoclib::{input::CommaSep, parse};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

const INTERVAL_BETWEEN_SPAWN: usize = 6;
const INTERVAL_TO_FIRST_SPAWN: usize = 8;

#[derive(Debug, Default)]
struct School([u64; INTERVAL_TO_FIRST_SPAWN + 1]);

impl Deref for School {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for School {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl School {
    fn new(fish: impl IntoIterator<Item = usize>) -> Option<Self> {
        let mut school = School::default();
        for fish in fish {
            if fish > INTERVAL_TO_FIRST_SPAWN {
                return None;
            }
            school[fish] += 1;
        }
        Some(school)
    }

    fn next(&mut self) {
        let spawn_count = self[0];
        // decrement all ages (overwriting the fish spawning)
        for age in 0..INTERVAL_TO_FIRST_SPAWN {
            self[age] = self[age + 1];
        }
        // fish which spawned have their timers reset to the spawning interval
        self[INTERVAL_BETWEEN_SPAWN] += spawn_count;
        // new fish
        self[INTERVAL_TO_FIRST_SPAWN] = spawn_count;
    }

    fn sum_fish(&self) -> u64 {
        self.0.iter().copied().sum()
    }
}

pub fn part1(input: &Path, days: usize) -> Result<(), Error> {
    for (idx, line) in parse::<CommaSep<usize>>(input)?.enumerate() {
        let mut school = School::new(line).ok_or(Error::ElderFish(idx))?;
        for _day in 0..days {
            school.next();
        }
        println!(
            "{}: total fish after {} days: {}",
            idx,
            days,
            school.sum_fish()
        );
    }
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    part1(input, 256)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Fish is too old (line {0})")]
    ElderFish(usize),
}
