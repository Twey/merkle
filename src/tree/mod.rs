use std::ops::{Range, RangeBounds};

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

impl<T: Default> FromIterator<T> for Tree<T> {
    fn from_iter<It: IntoIterator<Item = T>>(items: It) -> Self {
        use itertools::Either;

        let iter = items.into_iter();

        let (num_leaves, items) = if let (lower, Some(upper)) = iter.size_hint() && lower == upper {
            // If we know how many items there are, use the iterator directly
            (lower, Either::Left(iter))
        } else {
            // Otherwise, collect them into a `Vec` so we can count them
            let vec = Vec::from_iter(iter).into_iter();
            (vec.len(), Either::Right(vec))
        };

        let num_nodes = (2 * num_leaves).saturating_sub(1);
        let num_branches = num_nodes - num_leaves;

        let mut nodes = Vec::with_capacity(num_nodes);
        nodes.resize_with(num_branches, T::default);
        nodes.extend(items);

        Self { nodes }
    }
}

pub struct Parts<'a, T> {
    pub branches: &'a [T],
    pub leaves: &'a [T],
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

    pub fn parts(&self) -> Parts<'_, T> {
        let (branches, leaves) = self.nodes.split_at((self.nodes.len().saturating_sub(1)) / 2);
        Parts { branches, leaves }
    }

    // Get all parents of all nodes in the given range
    pub fn parents(&self, nodes: &impl RangeBounds<Node>) -> Range<Node> {
        // Bound to 1 because the root has no parent.
        let nodes = bound(nodes, 1, self.nodes.len());
        // If the end of the range is a right node then include its parent in the range.
        let end_is_right_node = if nodes.end.is_multiple_of(2) && nodes.end != 0 { 1 } else { 0 };

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


fn bound(range: &impl RangeBounds<usize>, min: usize, max: usize) -> Range<usize> {
    use std::ops::Bound::*;

    let start = match range.start_bound() {
        Included(n) => *n,
        Excluded(n) => n + 1,
        Unbounded => min,
    }.clamp(min, max);

    let end = match range.end_bound() {
        Included(n) => n + 1,
        Excluded(n) => *n,
        Unbounded => max,
    }.clamp(min, max);

    Range { start, end }
}

pub fn path_to_root(node: Node) -> impl Iterator<Item = Node> {
    std::iter::successors(Some(node), |node| Some(parent(*node)))
        .take_while(|node| *node != 0)
}

#[cfg(test)]
mod tests;
