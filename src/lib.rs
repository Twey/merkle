use sha2::{Sha256, Digest};

use std::ops::Range;

type NodeIndex = usize;
type Index = usize;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash([u8; 32]);

impl std::fmt::Debug for Hash {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in &self.0 {
            write!(formatter, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("proof verification failure")]
    VerificationFailed,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// TODO store leaf values
// TODO parameterize over hasher
#[derive(Default, Debug, Clone)]
pub struct Tree {
    // Implicit data structure saves allocations.
    // The (1-indexed) nth child of node k is at 2k+n
    nodes: Vec<Hash>,
}

// A possibly-invalid Merkle proof.
#[derive(Debug)]
pub struct Preproof {
    root: Hash,
    // TODO distinguish nodes and indices better
    node: NodeIndex,
    // If we didn't do domain separation we would want a height here.
    content: Hash,
    siblings: Vec<Hash>,
}

impl<T: AsRef<[u8]>> FromIterator<T> for Tree {
    fn from_iter<It: IntoIterator<Item = T>>(items: It) -> Self {
        // TODO try to use size_hint to avoid this allocation
        let items: Vec<_> = items.into_iter().map(|item| Self::hash_leaf(item.as_ref())).collect();

        let num_leaves = items.len();
        let num_nodes = 2 * num_leaves - 1;
        let num_branches = num_nodes - num_leaves;

        let mut nodes = Vec::with_capacity(num_nodes);
        nodes.resize(num_branches, Hash::default());
        nodes.extend_from_slice(&items);

        let mut me = Self { nodes };

        me.recalculate(&(num_branches..num_nodes));

        me
    }
}

// A known-valid Merkle proof.
#[non_exhaustive]
pub struct Proof {
    pub preproof: Preproof,
}

struct Parts<'a> {
    branches: &'a [Hash],
    leaves: &'a [Hash],
}

impl Tree {
    pub fn root(&self) -> Option<Hash> {
        self.nodes.first().cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.parts().leaves.len()
    }

    pub fn prove(&self, index: Index) -> Result<Proof> {
        dbg!((self, index));

        let node = self.parts().branches.len() + index;
        if node >= self.nodes.len() {
            return Err(Error::IndexOutOfBounds)
        }

        Ok(Proof {
            preproof: Preproof {
                root: self.nodes[0],
                node,
                content: self.nodes[node],
                siblings: Self::path_to_root(node).map(|node| self.nodes[Self::sibling(node)]).collect(),
            },
        })
    }

    fn sibling(node: NodeIndex) -> NodeIndex {
        if node.is_multiple_of(2) {
            node - 1
        } else {
            node + 1
        }
    }

    fn hash_leaf(value: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update([0x00]);
        hasher.update(value);
        Hash(hasher.finalize().into())
    }

    fn hash_branch(left: Hash, right: Hash) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update([0x01]);
        hasher.update(left);
        hasher.update(right);
        Hash(hasher.finalize().into())
    }

    fn parent(index: Index) -> Index {
        if index == 0 {
            0
        } else {
            (index - 1) / 2
        }
    }

    fn left_child(index: Index) -> Index {
        2 * index + 1
    }

    fn right_child(index: Index) -> Index {
        2 * index + 2
    }

    // Get all parents of all nodes in the given range
    fn parents(range: &Range<Index>) -> Range<Index> {
        // Bound to 1 because the root has no parent.
        let start = std::cmp::max(range.start, 1);
        // If the end of the range is a right node then include its parent in the range.
        let end_is_right_node = if range.end.is_multiple_of(2) && range.end != 0 { 1 } else { 0 };

        Range {
            start: Self::parent(start),
            end: Self::parent(range.end) + end_is_right_node,
        }
    }

    fn update(&mut self, node: Index) {
        self.nodes[node] = Self::hash_branch(
            self.nodes[Self::left_child(node)],
            self.nodes[Self::right_child(node)],
        );
    }

    // Given a range of nodes that have been updated, update the branches above up to and including the root.
    // TODO distinguish better between Index and Index
    fn recalculate(&mut self, nodes: &Range<NodeIndex>) {
        let mut parents = Self::parents(nodes);

        while !parents.is_empty() {
            for parent in parents.clone().rev() {
                self.update(parent);
            }

            parents = Self::parents(&parents);
        }
    }

    fn parts(&self) -> Parts<'_> {
        let (branches, leaves) = self.nodes.split_at((self.nodes.len().saturating_sub(1)) / 2);
        Parts { branches, leaves }
    }

    fn path_to_root(mut node: Index) -> impl Iterator<Item = NodeIndex> {
        std::iter::from_fn(move || {
            let node_ = Some(node);
            node = Self::parent(node);
            node_
        }).take_while(|node| *node != 0)
    }
}

impl Preproof {
    pub fn verify(self) -> Result<Proof> {
        let mut hash = self.content;
        let mut node = self.node;

        for &sibling in &self.siblings {
            let (left, right) = if node.is_multiple_of(2) {
                (sibling, hash)
            } else {
                (hash, sibling)
            };

            hash = Tree::hash_branch(left, right);
            node = Tree::parent(node);
        }

        if hash == self.root && node == 0 {
            Ok(Proof { preproof: self })
        } else {
            Err(Error::VerificationFailed)
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod tree_proptests;
#[cfg(test)]
mod proptests;
