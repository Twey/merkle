use std::ops::{Range, RangeBounds};

/// The index of a node in the tree.
pub type Node = u32;

/// An Eytzinger-layout implicit tree structure for complete binary trees.
#[derive(Default, Debug, Clone)]
pub struct Tree<T> {
    pub(crate) nodes: Vec<T>,
}

impl<T: Default> FromIterator<T> for Tree<T> {
    fn from_iter<It: IntoIterator<Item = T>>(items: It) -> Self {
        let items: Vec<_> = items.into_iter().collect();
        let num_leaves = items.len();
        let num_nodes = (2 * num_leaves).saturating_sub(1);
        let num_branches = num_nodes - num_leaves;

        let mut nodes = Vec::with_capacity(num_nodes);
        nodes.resize_with(num_branches, T::default);
        nodes.extend(items);

        Self { nodes }
    }
}

struct Parts<'a, T> {
    branches: &'a [T],
    leaves: &'a [T],
}

// Public API
impl<T> Tree<T> {
    pub fn root(&self) -> Option<&T> {
        self.nodes.first()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl<T> Tree<T> {
    pub fn branches(&self) -> &[T] {
        self.parts().branches
    }

    pub fn leaves(&self) -> &[T] {
        self.parts().leaves
    }

    fn parts(&self) -> Parts<'_, T> {
        let (branches, leaves) = self
            .nodes
            .split_at((self.nodes.len().saturating_sub(1)) / 2);
        Parts { branches, leaves }
    }

    // Get all parents of all nodes in the given range
    pub fn parents(&self, nodes: &impl RangeBounds<Node>) -> Range<Node> {
        let nodes = bound(
            nodes,
            // Bound to 1 because the root has no parent.
            1,
            Node::try_from(self.nodes.len()).expect("number of nodes may not exceed u32::MAX"),
        );

        // If the end of the range is a right node then include its parent in the range.
        let end_is_right_node = Node::from(nodes.end.is_multiple_of(2) && nodes.end != 0);

        Range {
            start: parent(nodes.start),
            end: parent(nodes.end) + end_is_right_node,
        }
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
    if index == 0 { 0 } else { (index - 1) / 2 }
}

pub fn left_child(index: Node) -> Node {
    2 * index + 1
}

pub fn right_child(index: Node) -> Node {
    2 * index + 2
}

// Bound a given range, returning a half-open range inside the given bounds.
fn bound(range: &impl RangeBounds<Node>, min: Node, max: Node) -> Range<Node> {
    use std::ops::Bound::{Excluded, Included, Unbounded};

    let start = match range.start_bound() {
        Included(n) => *n,
        Excluded(n) => n + 1,
        Unbounded => min,
    }
    .clamp(min, max);

    let end = match range.end_bound() {
        Included(n) => n + 1,
        Excluded(n) => *n,
        Unbounded => max,
    }
    .clamp(min, max + 1);

    Range { start, end }
}

// Yield the path from the given node to the root, including the starting node
// but excluding the root node itself.
pub fn path_to_root(node: Node) -> impl Iterator<Item = Node> {
    std::iter::successors(Some(node), |node| Some(parent(*node))).take_while(|node| *node != 0)
}

#[cfg(test)]
mod tests;
