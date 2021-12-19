use std::{
    cell::RefCell,
    ops::Deref,
    path::Path,
    rc::{Rc, Weak},
    str::FromStr,
};

use aoclib::parse;

struct Branch<T> {
    left: Rc<Node<T>>,
    right: Rc<Node<T>>,
}

enum NodeType<T> {
    Leaf(T),
    Branch(Branch<T>),
}

impl<T> NodeType<T> {
    fn as_leaf(&self) -> Option<&T> {
        if let NodeType::Leaf(ref t) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_branch(&self) -> Option<&Branch<T>> {
        if let NodeType::Branch(ref branch) = self {
            Some(branch)
        } else {
            None
        }
    }
}

struct Node<T> {
    node_type: RefCell<NodeType<T>>,
    up: Option<Weak<Node<T>>>,
}

impl<T> Node<T> {
    /// Construct a new value node without a parent.
    fn new_value(value: T) -> Rc<Self> {
        Rc::new(Self {
            node_type: RefCell::new(NodeType::Leaf(value)),
            up: None,
        })
    }

    /// Construct a new pair node without a parent.
    ///
    /// If either child had a parent or an external reference, this function will return `None`.
    fn new_pair(left: Rc<Node<T>>, right: Rc<Node<T>>) -> Option<Rc<Self>> {
        if left.up.is_some()
            || right.up.is_some()
            || Rc::strong_count(&left) > 1
            || Rc::weak_count(&left) > 0
            || Rc::strong_count(&right) > 1
            || Rc::weak_count(&right) > 0
        {
            return None;
        }

        let root = Rc::new(Self {
            node_type: RefCell::new(NodeType::Branch(Branch { left, right })),
            up: None,
        });

        // We have to mess with the nodes to create appropriate up pointers,
        // even though at this point we don't have write access.
        // That's ok; we know that we have unique access to each of these, so it's ok to reach in
        // with unsafe sorcery and modify the item anyway.
        let left_ptr = Rc::as_ptr(&root.left_child().unwrap().upgrade().unwrap()) as *mut Self;
        let right_ptr = Rc::as_ptr(&root.right_child().unwrap().upgrade().unwrap()) as *mut Self;
        for ptr in [left_ptr, right_ptr] {
            unsafe {
                (*ptr).up = Some(Rc::downgrade(&root));
            }
        }
        Some(root)
    }

    /// Return the value of this node if this is a value node.
    fn value_copied<'a>(self: &'a Rc<Self>) -> Option<T>
    where
        T: Copy,
    {
        let node_type = self.node_type.borrow();
        node_type.as_leaf().copied()
    }

    /// Return the left child of this node if this is a branch node.
    fn left_child(self: &Rc<Self>) -> Option<Weak<Node<T>>> {
        let node_type = self.node_type.borrow();
        node_type
            .as_branch()
            .map(|branch| Rc::downgrade(&branch.left))
    }

    /// Return the right child of this node if this is a branch node.
    fn right_child(self: &Rc<Self>) -> Option<Weak<Node<T>>> {
        let node_type = self.node_type.borrow();
        node_type
            .as_branch()
            .map(|branch| Rc::downgrade(&branch.right))
    }

    /// Return the leftmost grandchild of this node.
    ///
    /// The returned node will always be a leaf.
    ///
    /// Returns `self` if `self` is already a leaf.
    fn leftmost_grandchild(self: &Rc<Self>) -> Weak<Node<T>> {
        match self.node_type.borrow().deref() {
            NodeType::Leaf(_) => Rc::downgrade(self),
            NodeType::Branch(branch) => branch.left.leftmost_grandchild(),
        }
    }

    /// Return the rightmost grandchild of this node.
    ///
    /// The returned node will always be a leaf.
    ///
    /// Returns `self` if `self` is already a leaf.
    fn rightmost_grandchild(self: &Rc<Self>) -> Weak<Node<T>> {
        match self.node_type.borrow().deref() {
            NodeType::Leaf(_) => Rc::downgrade(self),
            NodeType::Branch(branch) => branch.right.rightmost_grandchild(),
        }
    }

    /// Return the next left-most node regardless of type.
    ///
    /// This produces a sibling node: one whose depth is equal to this node's.
    fn left_sibling(self: &Rc<Self>) -> Option<Weak<Node<T>>> {
        let parent = self.up.as_ref()?.upgrade()?;
        let node_type = parent.node_type.borrow();
        let branch = node_type
            .as_branch()
            .expect("parenthood implies being a branch");

        if Rc::ptr_eq(self, &branch.right) {
            Some(Rc::downgrade(&branch.left))
        } else {
            let left_uncle = parent.left_sibling()?.upgrade()?;
            let node_type = left_uncle.node_type.borrow();
            let branch = node_type.as_branch()?;
            Some(Rc::downgrade(&branch.right))
        }
    }

    /// Return the next right-most node regardless of type.
    ///
    /// This produces a sibling node: one whose depth is equal to this node's.
    fn right_sibling(self: &Rc<Self>) -> Option<Weak<Node<T>>> {
        let parent = self.up.as_ref()?.upgrade()?;
        let node_type = parent.node_type.borrow();
        let branch = node_type
            .as_branch()
            .expect("parenthood implies being a branch");

        if Rc::ptr_eq(self, &branch.left) {
            Some(Rc::downgrade(&branch.right))
        } else {
            let right_uncle = parent.right_sibling()?.upgrade()?;
            let node_type = right_uncle.node_type.borrow();
            let branch = node_type.as_branch()?;
            Some(Rc::downgrade(&branch.left))
        }
    }

    /// Return the next leaf left from this node.
    fn left_leaf(self: &Rc<Self>) -> Option<Weak<Node<T>>> {
        let sibling = self.left_sibling()?.upgrade()?;
        Some(sibling.rightmost_grandchild())
    }

    /// Return the next leaf right from this node.
    fn right_leaf(self: &Rc<Self>) -> Option<Weak<Node<T>>> {
        let sibling = self.right_sibling()?.upgrade()?;
        Some(sibling.leftmost_grandchild())
    }
}

type SnailfishNumber = Node<u8>;

impl SnailfishNumber {
    pub fn add(self: Rc<Self>, other: Rc<Self>) -> Option<Rc<Self>> {
        SnailfishNumber::new_pair(self, other).map(|sfn| {
            sfn.reduce();
            sfn
        })
    }

    fn reduce(self: &Rc<Self>) {
        let mut operation_applied = true;
        while operation_applied {
            operation_applied = false;
            for operation in [
                Box::new(Self::try_explode) as Box<dyn Fn(&Rc<Self>) -> bool>,
                Box::new(Self::try_split),
            ] {
                operation_applied |= operation(self);
                if operation_applied {
                    break;
                }
            }
        }
    }

    fn try_explode(self: &Rc<Self>) -> bool {
        self.explode_inner(0)
    }

    fn explode_inner(self: &Rc<Self>, depth: usize) -> bool {
        // handle the actual explosion case
        let mut did_explode = false;
        if depth == 4 {
            if let Some(branch) = self.node_type.borrow().as_branch() {
                did_explode = true;
                debug_assert!(
                    branch.left.value_copied().is_some() && branch.right.value_copied().is_some(),
                    "problem statement promises that exploding values are always simple values"
                );

                if let Some(left) = self.left_leaf().and_then(|leaf| leaf.upgrade()) {
                    let new_value = branch.left.value_copied().expect(
                        "problem statement promises that explosions only hit simple numbers",
                    ) + left
                        .value_copied()
                        .expect("left_leaf always produces a leaf");
                    left.node_type.replace(NodeType::Leaf(new_value));
                }
                if let Some(right) = self.right_leaf().and_then(|leaf| leaf.upgrade()) {
                    let new_value = branch.right.value_copied().expect(
                        "problem statement promises that explosions only hit simple numbers",
                    ) + right
                        .value_copied()
                        .expect("right_leaf always produces a leaf");
                    right.node_type.replace(NodeType::Leaf(new_value));
                }
            }
            if did_explode {
                self.node_type.replace(NodeType::Leaf(0));
            }
        }

        // handle recursion by abusing short-circuit behavior:
        // if at any point something explodes, we return immediately instead of continuing to explode
        did_explode
            || if let Some(branch) = self.node_type.borrow().as_branch() {
                branch.left.explode_inner(depth + 1) || branch.right.explode_inner(depth + 1)
            } else {
                false
            }
    }

    fn try_split(self: &Rc<Self>) -> bool {
        if let Some(value) = self.value_copied() {
            if value >= 10 {
                let left = Self::new_value(value / 2);
                let right = Self::new_value(value / 2 + value % 2);
                self.node_type
                    .replace(NodeType::Branch(Branch { left, right }));
                return true;
            }
        }
        if let Some(branch) = self.node_type.borrow().as_branch() {
            branch.left.try_split() || branch.right.try_split()
        } else {
            false
        }
    }

    fn magnitude(self: &Rc<Self>) -> u64 {
        match self.node_type.borrow().deref() {
            NodeType::Leaf(value) => *value as u64,
            NodeType::Branch(branch) => {
                (branch.left.magnitude() * 3) + (branch.right.magnitude() * 2)
            }
        }
    }
}

impl FromStr for SnailfishNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let sum = parse::<SnailfishNumber>(input)?
        .map(Rc::new)
        .reduce(|acc, item| {
            acc.add(item)
                .expect("all these numbers should be distinct and have no other references")
        })
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
    #[error("no solution found")]
    NoSolution,
}
