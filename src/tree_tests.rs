use super::*;

#[test]
fn empty_tree_has_no_root() {
    let tree = Tree::default();
    assert_eq!(tree.root(), None);
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
}

#[test]
fn single_leaf_root_is_leaf_hash() {
    let tree = Tree::from_iter(["a"]);
    assert_eq!(tree.root(), Some(Tree::hash_leaf(b"a")));
    assert_eq!(tree.len(), 1);
    assert!(!tree.is_empty());
}

#[test]
fn two_leaf_tree_root_is_parent_hash() {
    let tree = Tree::from_iter(["a", "b"]);
    let expected = Tree::hash_branch(
        Tree::hash_leaf(b"a"),
        Tree::hash_leaf(b"b"),
    );
    assert_eq!(tree.root(), Some(expected));
    assert_eq!(tree.len(), 2);
}

#[test]
fn three_leaf_tree() {
    let tree = Tree::from_iter(["a", "b", "c"]);
    assert!(tree.root().is_some());
    assert_eq!(tree.len(), 3);
}

#[test]
fn four_leaf_tree_root_is_correct() {
    let tree = Tree::from_iter(["a", "b", "c", "d"]);
    let left = Tree::hash_branch(
        Tree::hash_leaf(b"a"),
        Tree::hash_leaf(b"b"),
    );
    let right = Tree::hash_branch(
        Tree::hash_leaf(b"c"),
        Tree::hash_leaf(b"d"),
    );
    let expected = Tree::hash_branch(left, right);
    assert_eq!(tree.root(), Some(expected));
    assert_eq!(tree.len(), 4);
}

#[test]
fn deterministic_construction() {
    let tree1 = Tree::from_iter(["a", "b", "c"]);
    let tree2 = Tree::from_iter(["a", "b", "c"]);
    assert_eq!(tree1.root(), tree2.root());
}

#[test]
fn different_inputs_produce_different_roots() {
    let tree1 = Tree::from_iter(["a", "b"]);
    let tree2 = Tree::from_iter(["a", "c"]);
    assert_ne!(tree1.root(), tree2.root());
}

#[test]
fn order_matters() {
    let tree1 = Tree::from_iter(["a", "b"]);
    let tree2 = Tree::from_iter(["b", "a"]);
    assert_ne!(tree1.root(), tree2.root());
}

#[test]
fn duplicate_leaves() {
    let tree = Tree::from_iter(["a", "a", "a", "a"]);
    assert!(tree.root().is_some());
    assert_eq!(tree.len(), 4);
}

#[test]
fn hash_leaf_domain_separation() {
    // A leaf hash should differ from a branch hash of the same bytes.
    let leaf = Tree::hash_leaf(b"ab");
    let branch = Tree::hash_branch(
        Tree::hash_leaf(b"a"),
        Tree::hash_leaf(b"b"),
    );
    assert_ne!(leaf, branch);
}

#[test]
fn large_tree_len() {
    let tree = Tree::from_iter((0..1000).map(|i| format!("leaf_{i}")));
    assert_eq!(tree.len(), 1000);
    assert!(tree.root().is_some());
}

#[test]
fn power_of_two_sizes() {
    for n in [1, 2, 4, 8, 16, 32, 64] {
        let tree = Tree::from_iter((0..n).map(|i| format!("{i}")));
        assert_eq!(tree.len(), n);
        assert!(tree.root().is_some());
    }
}

#[test]
fn non_power_of_two_sizes() {
    for n in [3, 5, 6, 7, 9, 10, 15, 17, 31, 33, 63, 65] {
        let tree = Tree::from_iter((0..n).map(|i| format!("{i}")));
        assert_eq!(tree.len(), n);
        assert!(tree.root().is_some());
    }
}
