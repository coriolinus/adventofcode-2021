use lalrpop_util::lalrpop_mod;
lalrpop_mod!(parser);

#[cfg(feature = "list_impl")]
pub mod list_impl;

use std::{cell::RefCell, fmt, ops::Deref, path::Path, str::FromStr};

use aoclib::parse;

#[derive(PartialEq)]
struct Branch<T> {
    left: Box<Node<T>>,
    right: Box<Node<T>>,
}

#[derive(PartialEq)]
enum Contents<T> {
    Leaf(T),
    Branch(Branch<T>),
}

impl<T: fmt::Debug> fmt::Debug for Contents<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaf(leaf) => write!(f, "{:?}", leaf),
            Self::Branch(branch) => f
                .debug_list()
                .entry(&branch.left)
                .entry(&branch.right)
                .finish(),
        }
    }
}

struct RefLeaf<'a, T>(std::cell::Ref<'a, Contents<T>>);

impl<'a, T> Deref for RefLeaf<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if let Contents::Leaf(value) = self.0.deref() {
            value
        } else {
            panic!("RefLeaf is only constructed for leaf contents")
        }
    }
}

struct RefBranch<'a, T>(std::cell::Ref<'a, Contents<T>>);

impl<'a, T> Deref for RefBranch<'a, T> {
    type Target = Branch<T>;

    fn deref(&self) -> &Self::Target {
        if let Contents::Branch(branch) = self.0.deref() {
            branch
        } else {
            panic!("RefBranch is only constructed for branch contents")
        }
    }
}

pub struct Node<T> {
    contents: RefCell<Contents<T>>,
    up: Option<*const Node<T>>,
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.contents.borrow())
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.contents == other.contents
    }
}

impl<T> Node<T> {
    /// Construct a new value node without a parent.
    pub fn new_orphan_value(value: T) -> Box<Self> {
        Box::new(Self {
            contents: RefCell::new(Contents::Leaf(value)),
            up: None,
        })
    }

    /// Construct a new value node which has a parent.
    pub fn new_value(value: T, parent: &Box<Self>) -> Box<Self> {
        Box::new(Self {
            contents: RefCell::new(Contents::Leaf(value)),
            up: Some(&**parent as _),
        })
    }

    /// Construct a new pair node without a parent.
    ///
    /// If either child had a parent or an external reference, this function will return `None`.
    pub fn new_pair(left: Box<Node<T>>, right: Box<Node<T>>) -> Box<Self> {
        let root = Box::new(Self {
            contents: RefCell::new(Contents::Branch(Branch { left, right })),
            up: None,
        });

        // we have to encapsulate these pointers so the borrow checker doesn't complain
        {
            // We have to mess with the nodes to create appropriate up pointers,
            // even though at this point we don't have write access.
            // That's ok; we know that we have unique access to each of these, so it's ok to reach in
            // with unsafe sorcery and modify the item anyway.
            let left_ptr = &*root.branch().unwrap().left as *const Self as *mut Self;
            let right_ptr = &*root.branch().unwrap().right as *const Self as *mut Self;
            for ptr in [left_ptr, right_ptr] {
                unsafe {
                    (*ptr).up = Some(&*root as _);
                }
            }
        }

        root
    }

    /// Return the value of this node if this is a value node.
    fn value(&self) -> Option<RefLeaf<'_, T>> {
        match self.contents.borrow().deref() {
            Contents::Leaf(_) => Some(RefLeaf(self.contents.borrow())),
            Contents::Branch(_) => None,
        }
    }

    /// Return the branch of this node if this is a branch node.
    fn branch(&self) -> Option<RefBranch<'_, T>> {
        match self.contents.borrow().deref() {
            Contents::Leaf(_) => None,
            Contents::Branch(_) => Some(RefBranch(self.contents.borrow())),
        }
    }

    /// Return the leftmost grandchild of this node.
    ///
    /// The returned node will always be a leaf.
    ///
    /// Returns `self` if `self` is already a leaf.
    fn leftmost_grandchild(&self) -> *const Self {
        match self.contents.borrow().deref() {
            Contents::Leaf(_) => self as _,
            Contents::Branch(branch) => branch.left.leftmost_grandchild(),
        }
    }

    /// Return the rightmost grandchild of this node.
    ///
    /// The returned node will always be a leaf.
    ///
    /// Returns `self` if `self` is already a leaf.
    fn rightmost_grandchild(&self) -> *const Self {
        match self.contents.borrow().deref() {
            Contents::Leaf(_) => self as _,
            Contents::Branch(branch) => branch.right.rightmost_grandchild(),
        }
    }

    /// Return the parent of this node.
    fn parent<'a>(&'a self) -> Option<&'a Self> {
        // safe because we only ever access a node via the root, and without concurrency.
        // if we have access to a node, its parent pointer is valid.
        self.up.map(|ptr| unsafe { &*ptr })
    }

    /// Return `Some(true)` when this node is its parent's left branch.
    ///
    /// `None` when this node is the root.
    fn is_left(&self) -> Option<bool> {
        let parent = self.parent()?;
        let left_child = &parent.branch().expect("parenthood implies branch").left;
        Some(std::ptr::eq(self as _, &**left_child as _))
    }

    /// Return `Some(true)` when this node is its parent's right branch.
    ///
    /// `None` when this node is the root.
    fn is_right(&self) -> Option<bool> {
        self.is_left().map(|left| !left)
    }

    /// Return the parent or grandparent of the next left-most node.
    ///
    /// This always produces a branch node of depth less than this node's.
    /// If this node is on the right, this produces the node's imediate parent.
    /// Otherwise, it will step upward arbitrarily far, seeking an ancestor
    /// whose direct descendent is on the right. It then returns that ancestor.
    fn left_parent<'a>(&'a self) -> Option<&'a Node<T>> {
        let parent = self.parent()?;
        if self.is_right()? {
            Some(parent)
        } else {
            parent.left_parent()
        }
    }

    /// Return the parent or grandparent of the next right-most node.
    ///
    /// This always produces a branch node of depth less than this node's.
    /// If this node is on the left, this produces the node's immediate parent.
    /// Otherwise, it will step upwards arbitrarily far, seeking an ancestor
    /// whose direct descendent is on the left. It then returns that ancestor.
    fn right_parent<'a>(&'a self) -> Option<&'a Node<T>> {
        let parent = self.parent()?;

        if self.is_left()? {
            Some(&parent)
        } else {
            parent.right_parent()
        }
    }

    /// Return the next leaf left from this node.
    fn left_leaf(&self) -> Option<*const Self> {
        let parent = self.left_parent()?;
        Some(parent.branch()?.left.rightmost_grandchild())
    }

    /// Return the next leaf right from this node.
    fn right_leaf(&self) -> Option<*const Self> {
        let parent = self.right_parent()?;
        Some(parent.branch()?.right.leftmost_grandchild())
    }

    /// Check that all legs of this node have valid up pointers
    #[cfg(test)]
    fn check_legs(&self) {
        if let Some(branch) = self.branch() {
            assert!(std::ptr::eq(self as _, branch.left.parent().unwrap() as _));
            assert!(std::ptr::eq(self as _, branch.right.parent().unwrap() as _));
            branch.left.check_legs();
            branch.right.check_legs();
        }
    }
}

type SnailfishNumber = Node<u8>;

impl SnailfishNumber {
    pub fn add(self: Box<Self>, other: Box<Self>) -> Box<Self> {
        let sfn = SnailfishNumber::new_pair(self, other);
        sfn.reduce();
        sfn
    }

    fn reduce(self: &Box<Self>) {
        let mut operation_applied = true;
        while operation_applied {
            operation_applied = false;
            for operation in [
                Box::new(Self::try_explode) as Box<dyn Fn(&Box<Self>) -> bool>,
                Box::new(Self::try_split),
            ] {
                operation_applied |= operation(self);
                if operation_applied {
                    break;
                }
            }
        }
    }

    fn try_explode(self: &Box<Self>) -> bool {
        self.explode_inner(0)
    }

    fn explode_inner(&self, depth: usize) -> bool {
        // left branch first
        if let Some(branch) = self.branch() {
            if branch.left.explode_inner(depth + 1) {
                return true;
            }
        }

        // oops, what if it's time for _us_ to explode?
        let mut did_explode = false;
        if depth == 4 {
            if let Some(branch) = self.branch() {
                did_explode = true;
                debug_assert!(
                    branch.left.value().is_some() && branch.right.value().is_some(),
                    "problem statement promises that exploding values are always simple values"
                );

                if let Some(left) = self.left_leaf() {
                    // left reference must always be valid
                    let left = unsafe { &*left };
                    let new_value = *branch.left.value().expect(
                        "problem statement promises that explosions only hit simple numbers",
                    ) + *left.value().expect("left_leaf always produces a leaf");
                    left.contents.replace(Contents::Leaf(new_value));
                }
                if let Some(right) = self.right_leaf() {
                    // right reference must always be valid
                    let right = unsafe { &*right };
                    let new_value = *branch.right.value().expect(
                        "problem statement promises that explosions only hit simple numbers",
                    ) + *right.value().expect("right_leaf always produces a leaf");
                    right.contents.replace(Contents::Leaf(new_value));
                }
            }
            if did_explode {
                self.contents.replace(Contents::Leaf(0));
                return true;
            }
        }

        // right branch
        self.branch()
            .map(|branch| branch.right.explode_inner(depth + 1))
            .unwrap_or_default()
    }

    fn try_split(self: &Box<Self>) -> bool {
        // left branch
        if let Some(branch) = self.branch() {
            if branch.left.try_split() {
                return true;
            }
        }

        // try this value
        let value = self.value().map(|ref_leaf| *ref_leaf);
        if let Some(value) = value {
            if value >= 10 {
                let left = Self::new_value(value / 2, self);
                let right = Self::new_value(value / 2 + value % 2, self);
                self.contents
                    .replace(Contents::Branch(Branch { left, right }));
                return true;
            }
        }

        self.branch()
            .map(|branch| branch.right.try_split())
            .unwrap_or_default()
    }

    fn magnitude(&self) -> u64 {
        match self.contents.borrow().deref() {
            Contents::Leaf(value) => *value as u64,
            Contents::Branch(branch) => {
                (branch.left.magnitude() * 3) + (branch.right.magnitude() * 2)
            }
        }
    }
}

impl FromStr for Box<SnailfishNumber> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::SnailfishParser::new()
            .parse(s)
            .map_err(|err| err.map_token(|t| t.to_string()).into())
    }
}

// known wrong, too low: 1094
// known wrong, too low: 2972
pub fn part1(input: &Path) -> Result<(), Error> {
    let sum = parse::<Box<SnailfishNumber>>(input)?
        .reduce(|acc, item| acc.add(item))
        .ok_or(Error::NoSolution)?;
    println!("magnitude of snailfish sum: {}", sum.magnitude());
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!("the list-based implementation is much faster, so did part2 there")
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error")]
    ParseError(#[from] lalrpop_util::ParseError<usize, String, &'static str>),
    #[error("no solution found")]
    NoSolution,
    #[cfg(feature = "list_impl")]
    #[error("failed to parse")]
    ListParseError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoclib::input::parse_str;
    use rstest::rstest;

    fn parse(s: &str) -> Box<SnailfishNumber> {
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
    #[case("[[[[[9,8],1],2],3],4]")]
    #[case("[7,[6,[5,[4,[3,2]]]]]")]
    #[case("[[6,[5,[4,[3,2]]]],1]")]
    #[case("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]")]
    #[case("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]")]
    fn test_parent_links(#[case] input: &str) {
        let sfn = parse(input);
        sfn.check_legs();
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
    #[case(
        "[[[[7,7],[7,0]],[[7,8],[8,7]]],[[[6,7],[6,[6,7]]],[[0,7],[17,0]]]]",
        "[[[[7,7],[7,0]],[[7,8],[8,7]]],[[[6,7],[12,0]],[[7,7],[17,0]]]]"
    )]
    fn explode(#[case] input: &str, #[case] expect: &str) {
        let sfn = parse(input);
        assert!(sfn.try_explode());
        assert_eq!(sfn, parse(expect));
    }

    #[rstest]
    #[case("10", "[5,5]")]
    #[case("11", "[5,6]")]
    #[case("12", "[6,6]")]
    fn split(#[case] input: &str, #[case] expect: &str) {
        let sfn = parse(input);
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
            parse_str::<Box<SnailfishNumber>>(input)
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
        let sum = parse_str::<Box<SnailfishNumber>>(assignment)
            .unwrap()
            .reduce(|acc, item| acc.add(item))
            .unwrap();
        assert_eq!(sum, expect);
        assert_eq!(sum.magnitude(), EXPECT_MAGNITUDE);
    }

    #[rstest]
    #[case(
        "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]",
        "[[[5,[2,8]],4],[5,[[9,9],0]]]",
        "[[[[7,0],[7,8]],[[7,9],[0,6]]],[[[7,0],[6,6]],[[7,7],[0,9]]]]"
    )]
    #[case(
        "[[[[7,0],[7,8]],[[7,9],[0,6]]],[[[7,0],[6,6]],[[7,7],[0,9]]]]",
        "[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]",
        "[[[[7,7],[7,7]],[[7,0],[7,7]]],[[[7,7],[6,7]],[[7,7],[8,9]]]]"
    )]
    #[case(
        "[[[[7,7],[7,7]],[[7,0],[7,7]]],[[[7,7],[6,7]],[[7,7],[8,9]]]]",
        "[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]",
        "[[[[6,6],[6,6]],[[7,7],[7,7]]],[[[7,0],[7,7]],[[7,8],[8,8]]]]"
    )]
    #[case(
        "[[[[6,6],[6,6]],[[7,7],[7,7]]],[[[7,0],[7,7]],[[7,8],[8,8]]]]",
        "[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]",
        "[[[[6,6],[7,7]],[[7,7],[8,8]]],[[[8,8],[0,8]],[[8,9],[9,9]]]]"
    )]
    #[case(
        "[[[[6,6],[7,7]],[[7,7],[8,8]]],[[[8,8],[0,8]],[[8,9],[9,9]]]]",
        "[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]",
        "[[[[6,6],[7,7]],[[7,7],[7,0]]],[[[7,7],[8,8]],[[8,8],[8,9]]]]"
    )]
    #[case(
        "[[[[6,6],[7,7]],[[7,7],[7,0]]],[[[7,7],[8,8]],[[8,8],[8,9]]]]",
        "[[[[5,4],[7,7]],8],[[8,3],8]]",
        "[[[[7,7],[7,7]],[[7,7],[7,7]]],[[[0,7],[8,8]],[[8,8],[8,9]]]]"
    )]
    #[case(
        "[[[[7,7],[7,7]],[[7,7],[7,7]]],[[[0,7],[8,8]],[[8,8],[8,9]]]]",
        "[[9,3],[[9,9],[6,[4,9]]]]",
        "[[[[7,7],[7,7]],[[7,7],[8,8]]],[[[8,8],[0,8]],[[8,9],[8,7]]]]"
    )]
    #[case(
        "[[[[7,7],[7,7]],[[7,7],[8,8]]],[[[8,8],[0,8]],[[8,9],[8,7]]]]",
        "[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]",
        "[[[[7,7],[7,7]],[[7,7],[7,7]]],[[[8,7],[8,7]],[[7,9],[5,0]]]]"
    )]
    #[case(
        "[[[[7,7],[7,7]],[[7,7],[7,7]]],[[[8,7],[8,7]],[[7,9],[5,0]]]]",
        "[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]",
        "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]"
    )]
    fn constructed_cases(#[case] acc: &str, #[case] elem: &str, #[case] expect: &str) {
        assert_eq!(parse(acc).add(parse(elem)), parse(expect));
    }
}
