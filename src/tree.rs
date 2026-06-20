use std::ops::Range;

/// The index of a node in the tree.
pub type Node = usize;

// TODO store leaf values
// TODO parameterize over hasher
#[derive(Default, Debug, Clone)]
pub struct Tree<T> {
    // Implicit data structure saves allocations.
    // The (1-indexed) nth child of node k is at 2k+n
    pub(crate) nodes: Vec<T>,
}

pub struct Parts<'a, T> {
    branches: &'a [T],
    leaves: &'a [T],
}

// Public API
impl<T> Tree<T> {
    pub fn root(&self) -> Option<&T> {
        self.nodes.first()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.leaves().len()
    }
}

impl<T> Tree<T> {
    pub fn branches(&self) -> &[T] {
        self.parts().branches
    }

    pub fn leaves(&self) -> &[T] {
        self.parts().leaves
    }

    pub fn parts(&self) -> Parts<'_, T> {
        let (branches, leaves) = self.nodes.split_at((self.nodes.len().saturating_sub(1)) / 2);
        Parts { branches, leaves }
    }
}

pub fn sibling(node: Node) -> Node {
    if node.is_multiple_of(2) {
        node - 1
    } else {
        node + 1
    }
}

pub fn parent(index: Node) -> Node {
    if index == 0 {
        0
    } else {
        (index - 1) / 2
    }
}

pub fn left_child(index: Node) -> Node {
    2 * index + 1
}

pub fn right_child(index: Node) -> Node {
    2 * index + 2
}

// Get all parents of all nodes in the given range
pub fn parents(range: &Range<Node>) -> Range<Node> {
    // Bound to 1 because the root has no parent.
    let start = std::cmp::max(range.start, 1);
    // If the end of the range is a right node then include its parent in the range.
    let end_is_right_node = if range.end.is_multiple_of(2) && range.end != 0 { 1 } else { 0 };

    Range {
        start: parent(start),
        end: parent(range.end) + end_is_right_node,
    }
}

pub fn path_to_root(node: Node) -> impl Iterator<Item = Node> {
    std::iter::successors(Some(node), |node| Some(parent(*node)))
        .take_while(|node| *node != 0)
}
