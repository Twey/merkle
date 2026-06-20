use itertools::Either;
use sha2::{Sha256, Digest};

use std::ops::Range;

mod tree;
pub use tree::Tree;
pub use tree::Node;

/// The index of an item in the list of leaves.
pub type Index = usize;

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

// A possibly-invalid Merkle proof.
#[derive(Debug)]
pub struct Preproof {
    root: Hash,
    node: Node,
    // If we didn't do domain separation we would want a height here.
    content: Hash,
    siblings: Vec<Hash>,
}

impl<T: AsRef<[u8]>> FromIterator<T> for Tree<Hash> {
    fn from_iter<It: IntoIterator<Item = T>>(items: It) -> Self {
        let iter = items.into_iter();

        let (num_leaves, items) = if let (lower, Some(upper)) = iter.size_hint() && lower == upper {
            // If we know how many items there are, use the iterator directly
            (lower, Either::Left(iter))
        } else {
            // Otherwise, collect them into a `Vec` so we can count them
            let vec = Vec::from_iter(iter).into_iter();
            (vec.len(), Either::Right(vec))
        };

        let num_nodes = 2 * num_leaves - 1;
        let num_branches = num_nodes - num_leaves;

        let mut nodes = Vec::with_capacity(num_nodes);
        nodes.resize(num_branches, Hash::default());
        nodes.extend(Iterator::map(items, |item| Self::hash_leaf(item.as_ref())));

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

// Public API
impl Tree<Hash> {
    pub fn prove(&self, index: Index) -> Result<Proof> {
        let node = self.branches().len() + index;
        if node >= self.nodes.len() {
            return Err(Error::IndexOutOfBounds)
        }

        Ok(Proof {
            preproof: Preproof {
                root: *self.root().ok_or(Error::IndexOutOfBounds)?,
                node,
                content: *self.nodes.get(node).ok_or(Error::IndexOutOfBounds)?,
                siblings: tree::path_to_root(node).map(|node| self.nodes[tree::sibling(node)]).collect(),
            },
        })
    }
}

impl Tree<Hash> {
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

    fn update(&mut self, node: Index) {
        self.nodes[node] = Self::hash_branch(
            self.nodes[tree::left_child(node)],
            self.nodes[tree::right_child(node)],
        );
    }

    // Given a range of nodes that have been updated, update the branches above up to and including the root.
    // TODO distinguish better between Index and Index
    fn recalculate(&mut self, nodes: &Range<Node>) {
        let mut parents = tree::parents(nodes);

        while !parents.is_empty() {
            for parent in parents.clone().rev() {
                self.update(parent);
            }

            parents = tree::parents(&parents);
        }
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
            node = tree::parent(node);
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
