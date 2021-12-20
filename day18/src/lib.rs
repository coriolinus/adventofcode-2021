use lalrpop_util::lalrpop_mod;
lalrpop_mod!(parser);

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

struct RefLeaf<'a, T>(std::cell::Ref<'a, Contents<T>>);

impl<'a, T> Deref for RefLeaf<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if let Contents::Leaf(value) = self.0.deref() {
            &value
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
            &branch
        } else {
            panic!("RefBranch is only constructed for branch contents")
        }
    }
}

impl<T> Node<T> {
    /// Construct a new value node without a parent.
    fn new_value(value: T) -> Self {
        Self {
            contents: RefCell::new(Contents::Leaf(value)),
            up: None,
        }
    }

    /// Construct a new pair node without a parent.
    ///
    /// If either child had a parent or an external reference, this function will return `None`.
    fn new_pair(left: Node<T>, right: Node<T>) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        let root = Self {
            contents: RefCell::new(Contents::Branch(Branch { left, right })),
            up: None,
        };

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
                    (*ptr).up = Some(&root as _);
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
    fn is_left(&self) -> Option<bool>
    where
        T: fmt::Debug,
    {
        let parent = self.parent()?;
        let left_child = &parent.branch().expect("parenthood implies branch").left;
        Some(std::ptr::eq(self as _, &**left_child as _))
    }

    /// Return `Some(true)` when this node is its parent's right branch.
    ///
    /// `None` when this node is the root.
    fn is_right(&self) -> Option<bool>
    where
        T: fmt::Debug,
    {
        self.is_left().map(|left| !left)
    }

    /// Return the parent or grandparent of the next left-most node.
    ///
    /// This always produces a branch node of depth less than this node's.
    /// If this node is on the right, this produces the node's imediate parent.
    /// Otherwise, it will step upward arbitrarily far, seeking an ancestor
    /// whose direct descendent is on the right. It then returns that ancestor.
    fn left_parent<'a>(&'a self) -> Option<&'a Node<T>>
    where
        T: fmt::Debug,
    {
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
    fn right_parent<'a>(&'a self) -> Option<&'a Node<T>>
    where
        T: fmt::Debug,
    {
        let parent = self.parent()?;

        if self.is_left()? {
            Some(&parent)
        } else {
            parent.right_parent()
        }
    }

    /// Return the next leaf left from this node.
    fn left_leaf(&self) -> Option<*const Self>
    where
        T: fmt::Debug,
    {
        let parent = self.left_parent()?;
        Some(parent.branch()?.left.rightmost_grandchild())
    }

    /// Return the next leaf right from this node.
    fn right_leaf(&self) -> Option<*const Self>
    where
        T: fmt::Debug,
    {
        let parent = self.right_parent()?;
        Some(parent.branch()?.right.leftmost_grandchild())
    }
}

type SnailfishNumber = Node<u8>;

impl SnailfishNumber {
    pub fn add(self: Self, other: Self) -> Self {
        let sfn = SnailfishNumber::new_pair(self, other);
        sfn.reduce();
        sfn
    }

    fn reduce(&self) {
        let mut operation_applied = true;
        while operation_applied {
            operation_applied = false;
            for operation in [
                Box::new(Self::try_explode) as Box<dyn Fn(&Self) -> bool>,
                Box::new(Self::try_split),
            ] {
                operation_applied |= operation(self);
                if operation_applied {
                    break;
                }
            }
        }
    }

    fn try_explode(&self) -> bool {
        self.explode_inner(0)
    }

    fn explode_inner(&self, depth: usize) -> bool {
        eprintln!("explode_inner({})", depth);
        // handle the actual explosion case
        let mut did_explode = false;
        if depth == 4 {
            if let Some(branch) = self.branch() {
                did_explode = true;
                debug_assert!(
                    branch.left.value().is_some() && branch.right.value().is_some(),
                    "problem statement promises that exploding values are always simple values"
                );

                if let Some(left) = self.left_leaf() {
                    let new_value = *branch.left.value().expect(
                        "problem statement promises that explosions only hit simple numbers",
                    ) + *unsafe { &*left }
                        .value()
                        .expect("left_leaf always produces a leaf");
                    unsafe { &*left }
                        .contents
                        .replace(Contents::Leaf(new_value));
                }
                if let Some(right) = self.right_leaf() {
                    let new_value = *branch.right.value().expect(
                        "problem statement promises that explosions only hit simple numbers",
                    ) + *unsafe { &*right }
                        .value()
                        .expect("right_leaf always produces a leaf");
                    unsafe { &*right }
                        .contents
                        .replace(Contents::Leaf(new_value));
                }
            }
            if did_explode {
                self.contents.replace(Contents::Leaf(0));
            }
        }

        // handle recursion by abusing short-circuit behavior:
        // if at any point something explodes, we return immediately instead of continuing to explode
        did_explode
            || if let Contents::Branch(branch) = self.contents.borrow().deref() {
                branch.left.explode_inner(depth + 1) || branch.right.explode_inner(depth + 1)
            } else {
                false
            }
    }

    fn try_split(&self) -> bool {
        if let Some(value) = self.value() {
            if *value >= 10 {
                let left = Box::new(Self::new_value(*value / 2));
                let right = Box::new(Self::new_value(*value / 2 + *value % 2));
                self.contents
                    .replace(Contents::Branch(Branch { left, right }));
                return true;
            }
        }
        if let Some(branch) = self.branch() {
            branch.left.try_split() || branch.right.try_split()
        } else {
            false
        }
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

impl FromStr for SnailfishNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::SnailfishParser::new()
            .parse(s)
            .map_err(|err| err.map_token(|t| t.to_string()).into())
    }
}

// known wrong, too low: 1094
pub fn part1(input: &Path) -> Result<(), Error> {
    let sum = parse::<SnailfishNumber>(input)?
        .reduce(|acc, item| acc.add(item))
        .ok_or(Error::NoSolution)?;
    println!("magnitude of snailfish sum: {}", sum.magnitude());
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error")]
    ParseError(#[from] lalrpop_util::ParseError<usize, String, &'static str>),
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn parse(s: &str) -> SnailfishNumber {
        s.parse().unwrap()
    }

    #[test]
    fn addition_1() {
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
        let sfn = parse(input);
        eprintln!("parsed");
        assert!(sfn.try_explode());
        assert_eq!(sfn, parse(expect));
    }
}
