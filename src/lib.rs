//! A generic Merkle tree implementation parameterized over a
//! [`digest::Digest`] hash function.
//!
//! # Examples
//!
//! ```
//! use merkle::Tree;
//! use sha2::Sha256;
//!
//! // Build a tree from byte-like items.
//! let tree: Tree<Sha256> = ["alpha", "beta", "gamma"].into_iter().collect();
//!
//! // Generate and verify a proof for a leaf.
//! let proof = tree.prove(1).unwrap();
//! let valid = proof.preproof.verify().unwrap();
//! ```

use std::ops::RangeBounds;


mod tree;
use tree::Node;

/// The index of an item in the list of leaves.
pub type Index = usize;

/// A Merkle tree that computes hashes using the digest algorithm `Digest`.
///
/// Construct a tree by collecting an iterator of byte-like items (anything
/// that implements [`AsRef<[u8]>`]).  An empty iterator produces an empty
/// tree whose [`root`](Tree::root) is `None`.
#[derive(Clone, Debug, Default)]
pub struct Tree<Digest, Hash = digest::Output<Digest>> {
    tree: tree::Tree<Hash>,
    _digest: std::marker::PhantomData<Digest>,
}

/// Errors returned by tree operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested leaf index is not present in the tree.
    #[error("index out of bounds")]
    IndexOutOfBounds,
    /// A [`Preproof`] did not verify against its claimed root.
    #[error("proof verification failed")]
    VerificationFailed,
}

/// A [`Result`](std::result::Result) type alias using [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A Merkle inclusion proof that has not yet been verified.
///
/// Obtained from [`Tree::prove`].  Call [`verify`](Preproof::verify) to
/// check the proof and obtain a [`Proof`].
#[derive(Debug)]
pub struct Preproof<Digest, Hash = digest::Output<Digest>> {
    root: Hash,
    siblings: Vec<Hash>,
    node: Node,
    content: Hash,
    _digest: std::marker::PhantomData<Digest>,
}

/// Builds a [`Tree`] from an iterator of byte-like items.
///
/// Each item is hashed as a leaf with domain separation (a `0x00` prefix).
/// Branch nodes are then computed bottom-up, each prefixed with `0x01`.
impl<Item: AsRef<[u8]>, Digest: digest::Digest> FromIterator<Item> for Tree<Digest> {
    fn from_iter<It: IntoIterator<Item = Item>>(items: It) -> Self {
        let mut me = Self {
            tree: items.into_iter().map(|item| Self::hash_leaf(item.as_ref())).collect(),
            _digest: std::marker::PhantomData,
        };
        let num_branches = me.tree.branches().len();
        me.recalculate(num_branches..);
        me
    }
}

/// A Merkle inclusion proof that has been verified to be valid.
///
/// This type can only be constructed by successfully calling
/// [`Preproof::verify`], so its existence guarantees that the proof
/// checked out.
#[non_exhaustive]
pub struct Proof<Digest, Hash = digest::Output<Digest>> {
    /// The underlying proof data.
    pub preproof: Preproof<Digest, Hash>,
}

impl<Digest, Hash> Tree<Digest, Hash> {
    /// Returns the root hash of the tree, or `None` if the tree is empty.
    #[must_use]
    pub fn root(&self) -> Option<&Hash> {
        self.tree.root()
    }

    /// Returns `true` if the tree contains no leaves.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Returns the number of leaves in the tree.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tree.leaves().len()
    }
}

impl<Digest: digest::Digest> Tree<Digest> {
    /// Generates a Merkle inclusion proof for the leaf at `index`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if `index >= self.len()`.
    pub fn prove(&self, index: Index) -> Result<Proof<Digest>> {
        let node = self.tree.branches().len() + index;
        if node >= self.tree.nodes.len() {
            return Err(Error::IndexOutOfBounds)
        }

        Ok(Proof {
            preproof: Preproof {
                root: self.tree.root().ok_or(Error::IndexOutOfBounds)?.clone(),
                node,
                content: self.tree.nodes.get(node).ok_or(Error::IndexOutOfBounds)?.clone(),
                siblings: tree::path_to_root(node).map(|node| self.tree.nodes[tree::sibling(node)].clone()).collect(),
                _digest: std::marker::PhantomData,
            },
        })
    }
}

impl<Digest: digest::Digest> Tree<Digest> {
    fn hash_leaf(value: &[u8]) -> digest::Output<Digest> {
        let mut hasher = Digest::new();
        hasher.update([0x00]);
        hasher.update(value);
        hasher.finalize()
    }

    fn hash_branch(left: digest::Output<Digest>, right: digest::Output<Digest>) -> digest::Output<Digest> {
        let mut hasher = Digest::new();
        hasher.update([0x01]);
        hasher.update(left);
        hasher.update(right);
        hasher.finalize()
    }

    fn update(&mut self, node: Node) {
        self.tree.nodes[node] = Self::hash_branch(
            self.tree.nodes[tree::left_child(node)].clone(),
            self.tree.nodes[tree::right_child(node)].clone(),
        );
    }

    fn recalculate(&mut self, nodes: impl RangeBounds<Node>) {
        let mut parents = self.tree.parents(&nodes);

        while !parents.is_empty() {
            for parent in parents.clone().rev() {
                self.update(parent);
            }

            parents = self.tree.parents(&parents);
        }
    }
}

impl<Digest: digest::Digest> Preproof<Digest> {
    /// Verifies this proof by recomputing the root hash from the leaf and
    /// its siblings, returning a `Proof` on success that can be used as evidence.
    ///
    /// # Errors
    ///
    /// Returns [`Error::VerificationFailed`] if the recomputed root does not match.
    pub fn verify(self) -> Result<Proof<Digest>> {
        let mut hash = self.content.clone();
        let mut node = self.node;

        for sibling in &self.siblings {
            let (left, right) = if node.is_multiple_of(2) {
                (sibling.clone(), hash)
            } else {
                (hash, sibling.clone())
            };

            hash = Tree::<Digest>::hash_branch(left, right);
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
