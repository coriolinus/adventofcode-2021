use crate::Error;
use aoclib::parse;
use itertools::Itertools;
use std::{path::Path, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    value: u64,
    depth: u8,
    position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnailfishNumber {
    items: Vec<Item>,
}

impl FromStr for SnailfishNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut depth = 0;
        let mut number_start_idx = None;
        let mut items = Vec::new();

        for (idx, ch) in s.char_indices() {
            match ch {
                '[' => depth += 1,
                '0'..='9' => {
                    if number_start_idx.is_none() {
                        number_start_idx = Some(idx);
                    }
                }
                ',' => {
                    if let Some(number_start_idx) = number_start_idx {
                        let value = s[number_start_idx..idx]
                            .parse()
                            .map_err(|_| Error::ListParseError)?;
                        items.push(Item {
                            value,
                            depth,
                            position: Position::Left,
                        });
                    }
                    number_start_idx = None;
                }
                ']' => {
                    if let Some(number_start_idx) = number_start_idx {
                        let value = s[number_start_idx..idx]
                            .parse()
                            .map_err(|_| Error::ListParseError)?;
                        items.push(Item {
                            value,
                            depth,
                            position: Position::Right,
                        });
                    }
                    number_start_idx = None;
                    depth -= 1;
                }
                _ => return Err(Error::ListParseError),
            }
        }

        Ok(SnailfishNumber { items })
    }
}

impl SnailfishNumber {
    fn add(mut self, mut other: Self) -> Self {
        self.items.append(&mut other.items);
        for item in self.items.iter_mut() {
            item.depth += 1;
        }
        self.reduce();
        self
    }

    fn reduce(&mut self) {
        let mut operation_applied = true;
        while operation_applied {
            operation_applied = false;
            for operation in [
                Box::new(Self::try_explode) as Box<dyn Fn(&mut Self) -> bool>,
                Box::new(Self::try_split),
            ] {
                operation_applied |= operation(self);
                if operation_applied {
                    break;
                }
            }
        }
    }

    fn try_explode(&mut self) -> bool {
        if let Some(left_idx) = self
            .items
            .windows(2)
            .enumerate()
            .filter(|(_idx, window)| {
                let left = &window[0];
                let right = &window[1];
                left.depth == 5
                    && right.depth == 5
                    && left.position == Position::Left
                    && right.position == Position::Right
            })
            .map(|(idx, _window)| idx)
            .next()
        {
            let right_idx = left_idx + 1;
            let mut position = Position::Left;
            if let Some(prior_idx) = left_idx.checked_sub(1) {
                self.items[prior_idx].value += self.items[left_idx].value;
                if self.items[prior_idx].position == Position::Left
                    && self.items[prior_idx].depth + 1 == self.items[left_idx].depth
                {
                    position = Position::Right;
                }
            }
            let subsequent_idx = right_idx + 1;
            if subsequent_idx < self.items.len() {
                self.items[subsequent_idx].value += self.items[right_idx].value;
            }

            self.items[left_idx].value = 0;
            self.items[left_idx].depth -= 1;
            self.items[left_idx].position = position;
            self.items.remove(right_idx);

            true
        } else {
            false
        }
    }

    fn try_split(&mut self) -> bool {
        if let Some(idx) = self
            .items
            .iter()
            .enumerate()
            .filter(|(_idx, item)| item.value >= 10)
            .map(|(idx, _item)| idx)
            .next()
        {
            let value = self.items[idx].value;

            self.items[idx].depth += 1;
            self.items[idx].value = value / 2;
            self.items[idx].position = Position::Left;

            let new_item = Item {
                depth: self.items[idx].depth,
                value: value / 2 + value % 2,
                position: Position::Right,
            };

            self.items.insert(idx + 1, new_item);

            true
        } else {
            false
        }
    }

    fn magnitude(&self) -> u64 {
        let mut items = self.items.clone();

        for level in (1..=4).rev() {
            while let Some(left_idx) = items
                .windows(2)
                .enumerate()
                .filter(|(_idx, window)| {
                    let left = &window[0];
                    let right = &window[1];

                    left.depth == level
                        && right.depth == level
                        && left.position == Position::Left
                        && right.position == Position::Right
                })
                .map(|(idx, _window)| idx)
                .next()
            {
                let right_idx = left_idx + 1;

                let mut position = Position::Left;
                if let Some(prior_idx) = left_idx.checked_sub(1) {
                    if items[prior_idx].position == Position::Left
                        && items[prior_idx].depth + 1 == items[left_idx].depth
                    {
                        position = Position::Right;
                    }
                }

                items[left_idx].value = (3 * items[left_idx].value) + (2 * items[right_idx].value);
                items[left_idx].depth -= 1;
                items[left_idx].position = position;
                items.remove(right_idx);
            }
        }

        debug_assert_eq!(items.len(), 1);
        items[0].value
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let sum = parse::<SnailfishNumber>(input)?
        .reduce(|acc, item| acc.add(item))
        .ok_or(Error::NoSolution)?;
    println!("magnitude of snailfish sum: {}", sum.magnitude());
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let numbers: Vec<SnailfishNumber> = parse(input)?.collect();
    let max_magnitude = numbers
        .iter()
        .cartesian_product(numbers.iter())
        .filter(|(a, b)| a != b)
        .flat_map(|(a, b)| [a.clone().add(b.clone()), b.clone().add(a.clone())].into_iter())
        .max_by_key(|snailfish| snailfish.magnitude())
        .ok_or(Error::NoSolution)?;
    println!("max magnitude pairwise sum: {}", max_magnitude.magnitude());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoclib::input::parse_str;
    use rstest::rstest;

    fn parse(s: &str) -> SnailfishNumber {
        s.parse().unwrap()
    }

    #[test]
    fn simple_addition_example() {
        assert_eq!(
            parse("[1,2]").add(parse("[[3,4],5]")),
            parse("[[1,2],[[3,4],5]]")
        );
    }

    #[rstest]
    #[case("[[[[[9,8],1],2],3],4]", "[[[[0,9],2],3],4]")]
    #[case("[7,[6,[5,[4,[3,2]]]]]", "[7,[6,[5,[7,0]]]]")]
    #[case("[[6,[5,[4,[3,2]]]],1]", "[[6,[5,[7,0]]],3]")]
    #[case(
        "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]",
        "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]"
    )]
    #[case("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]", "[[3,[2,[8,0]]],[9,[5,[7,0]]]]")]
    fn explode(#[case] input: &str, #[case] expect: &str) {
        let mut sfn = parse(input);
        assert!(sfn.try_explode());
        assert_eq!(sfn, parse(expect));
    }

    #[rstest]
    #[case("[0,10]", "[0,[5,5]]")]
    #[case("[0,11]", "[0,[5,6]]")]
    #[case("[0,12]", "[0,[6,6]]")]
    fn split(#[case] input: &str, #[case] expect: &str) {
        let mut sfn = parse(input);
        assert!(sfn.try_split());
        assert_eq!(sfn, parse(expect));
    }

    #[test]
    fn multistage_addition_example() {
        assert_eq!(
            parse("[[[[4,3],4],4],[7,[[8,4],9]]]").add(parse("[1,1]")),
            parse("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]")
        )
    }

    const SUM_1: &str = "
[1,1]
[2,2]
[3,3]
[4,4]
    ";

    const SUM_2: &str = "
[1,1]
[2,2]
[3,3]
[4,4]
[5,5]
    ";

    const SUM_3: &str = "
[1,1]
[2,2]
[3,3]
[4,4]
[5,5]
[6,6]
    ";

    #[rstest]
    #[case(SUM_1.trim(), "[[[[1,1],[2,2]],[3,3]],[4,4]]")]
    #[case(SUM_2.trim(), "[[[[3,0],[5,3]],[4,4]],[5,5]]")]
    #[case(SUM_3.trim(), "[[[[5,0],[7,4]],[5,5]],[6,6]]")]
    fn example_sums(#[case] input: &str, #[case] expect: &str) {
        assert_eq!(
            parse_str::<SnailfishNumber>(input)
                .unwrap()
                .reduce(|acc, item| acc.add(item))
                .unwrap(),
            parse(expect)
        );
    }

    #[rstest]
    #[case("[9,1]", 29)]
    #[case("[1,9]", 21)]
    #[case("[[9,1],[1,9]]", 129)]
    #[case("[[1,2],[[3,4],5]]", 143)]
    #[case("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384)]
    #[case("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445)]
    #[case("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791)]
    #[case("[[[[5,0],[7,4]],[5,5]],[6,6]]", 1137)]
    #[case("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]", 3488)]
    fn example_magnitudes(#[case] input: &str, #[case] expect: u64) {
        assert_eq!(parse(input).magnitude(), expect);
    }

    #[test]
    fn example_assignment() {
        let expect = parse("[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]");
        const EXPECT_MAGNITUDE: u64 = 4140;
        assert_eq!(expect.magnitude(), EXPECT_MAGNITUDE);

        let assignment = "
[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]
        "
        .trim();
        let sum = parse_str::<SnailfishNumber>(assignment)
            .unwrap()
            .reduce(|acc, item| acc.add(item))
            .unwrap();
        assert_eq!(sum, expect);
        assert_eq!(sum.magnitude(), EXPECT_MAGNITUDE);
    }
}
