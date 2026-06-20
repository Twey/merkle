use std::ops::RangeBounds;


mod tree;
pub use tree::Node;

/// The index of an item in the list of leaves.
pub type Index = usize;

#[derive(Clone, Debug, Default)]
pub struct Tree<D, Hash = digest::Output<D>> {
    tree: tree::Tree<Hash>,
    _digest: std::marker::PhantomData<D>,
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
pub struct Preproof<Digest, Hash = digest::Output<Digest>> {
    root: Hash,
    siblings: Vec<Hash>,
    node: Node,
    content: Hash,
    // If we didn't do domain separation we would want a height here.
    _digest: std::marker::PhantomData<Digest>,
}

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

// A known-valid Merkle proof.
#[non_exhaustive]
pub struct Proof<Digest, Hash = digest::Output<Digest>> {
    pub preproof: Preproof<Digest, Hash>,
}

impl<Digest, Hash> Tree<Digest, Hash> {
    pub fn root(&self) -> Option<&Hash> {
        self.tree.root()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tree.leaves().len()
    }
}

// Public API
impl<Digest: digest::Digest> Tree<Digest> {
    pub fn prove(&self, index: Index) -> Result<Proof<Digest>> {
        let node = self.tree.branches().len() + index;
        if node >= self.tree.nodes.len() {
            return Err(Error::IndexOutOfBounds)
        }

        Ok(Proof {
            preproof: Preproof {
                root: *self.tree.root().ok_or(Error::IndexOutOfBounds)?,
                node,
                content: *self.0.nodes.get(node).ok_or(Error::IndexOutOfBounds)?,
                siblings: tree::path_to_root(node).map(|node| self.0.nodes[tree::sibling(node)]).collect(),
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

    // Given a range of nodes that have been updated, update the branches above up to and including the root.
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
