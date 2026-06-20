use super::super::*;

#[test]
fn empty_tree_has_no_root() {
    let tree: Tree<()> = [].into_iter().collect();
    assert_eq!(tree.root(), None);
    assert!(tree.is_empty());
    assert_eq!(tree.leaves().len(), 0);
}

#[test]
fn single_leaf() {
    let tree: Tree<_> = [10].into_iter().collect();
    assert_eq!(tree.root(), Some(&10));
    assert_eq!(tree.leaves().len(), 1);
    assert!(!tree.is_empty());
}

#[test]
fn two_leaves() {
    let tree: Tree<_> = [10, 20].into_iter().collect();
    assert!(tree.root().is_some());
    assert_eq!(tree.leaves(), &[10, 20]);
    assert_eq!(tree.leaves().len(), 2);
}

#[test]
fn three_leaves() {
    let tree: Tree<_> = [10, 20, 30].into_iter().collect();
    assert!(tree.root().is_some());
    assert_eq!(tree.leaves(), &[10, 20, 30]);
    assert_eq!(tree.leaves().len(), 3);
}

#[test]
fn four_leaves() {
    let tree: Tree<_> = [10, 20, 30, 40].into_iter().collect();
    assert!(tree.root().is_some());
    assert_eq!(tree.leaves(), &[10, 20, 30, 40]);
    assert_eq!(tree.leaves().len(), 4);
}

#[test]
fn single_leaf_root_is_leaf_value() {
    let tree: Tree<_> = [42].into_iter().collect();
    assert_eq!(tree.root(), Some(&42));
}

#[test]
fn leaves_are_preserved() {
    let values: Vec<i32> = (0..8).collect();
    let tree: Tree<_> = values.iter().copied().collect();
    assert_eq!(tree.leaves(), values.as_slice());
}

#[test]
fn branches_count() {
    // For n leaves, there are n-1 branches.
    for n in [1, 2, 3, 4, 5, 8, 16, 31, 64] {
        let tree: Tree<_> = (0..n).collect();
        assert_eq!(tree.branches().len(), n - 1, "n={n}");
        assert_eq!(tree.leaves().len(), n, "n={n}");
    }
}

#[test]
fn duplicate_leaves() {
    let tree: Tree<_> = [7, 7, 7, 7].into_iter().collect();
    assert!(tree.root().is_some());
    assert_eq!(tree.leaves().len(), 4);
    assert!(tree.leaves().iter().all(|&v| v == 7));
}

#[test]
fn large_tree_len() {
    let tree: Tree<_> = (0..1000).collect();
    assert_eq!(tree.leaves().len(), 1000);
    assert!(tree.root().is_some());
}

#[test]
fn power_of_two_sizes() {
    for n in [1, 2, 4, 8, 16, 32, 64] {
        let tree: Tree<_> = (0..n).collect();
        assert_eq!(tree.leaves().len(), n);
        assert!(tree.root().is_some());
    }
}

#[test]
fn non_power_of_two_sizes() {
    for n in [3, 5, 6, 7, 9, 10, 15, 17, 31, 33, 63, 65] {
        let tree: Tree<_> = (0..n).collect();
        assert_eq!(tree.leaves().len(), n);
        assert!(tree.root().is_some());
    }
}
