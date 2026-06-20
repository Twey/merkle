use std::ops::RangeBounds;
use sha2::{Sha256, Digest};

mod tree;
pub use tree::Node;

/// The index of an item in the list of leaves.
pub type Index = usize;

#[derive(Clone, Debug, Default)]
pub struct Tree(tree::Tree<Hash>);

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
    siblings: Vec<Hash>,
    node: Node,
    content: Hash,
    // If we didn't do domain separation we would want a height here.
}

impl<T: AsRef<[u8]>> FromIterator<T> for Tree {
    fn from_iter<It: IntoIterator<Item = T>>(items: It) -> Self {
        let mut me = Self(items.into_iter().map(|item| Self::hash_leaf(item.as_ref())).collect());
        let num_branches = me.0.branches().len();
        me.recalculate(num_branches..);
        me
    }
}

// A known-valid Merkle proof.
#[non_exhaustive]
pub struct Proof {
    pub preproof: Preproof,
}

// Public API
impl Tree {
    pub fn root(&self) -> Option<&Hash> {
        self.0.root()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.leaves().len()
    }

    pub fn prove(&self, index: Index) -> Result<Proof> {
        let node = self.0.branches().len() + index;
        if node >= self.0.nodes.len() {
            return Err(Error::IndexOutOfBounds)
        }

        Ok(Proof {
            preproof: Preproof {
                root: *self.0.root().ok_or(Error::IndexOutOfBounds)?,
                node,
                content: *self.0.nodes.get(node).ok_or(Error::IndexOutOfBounds)?,
                siblings: tree::path_to_root(node).map(|node| self.0.nodes[tree::sibling(node)]).collect(),
            },
        })
    }
}

impl Tree {
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

    fn update(&mut self, node: Node) {
        self.0.nodes[node] = Self::hash_branch(
            self.0.nodes[tree::left_child(node)],
            self.0.nodes[tree::right_child(node)],
        );
    }

    // Given a range of nodes that have been updated, update the branches above up to and including the root.
    // TODO distinguish better between Index and Index
    fn recalculate(&mut self, nodes: impl RangeBounds<Node>) {
        let mut parents = self.0.parents(&nodes);

        while !parents.is_empty() {
            for parent in parents.clone().rev() {
                self.update(parent);
            }

            parents = self.0.parents(&parents);
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
