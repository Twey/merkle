use super::*;
use proptest::prelude::*;

fn arb_leaves(min: usize, max: usize) -> impl Strategy<Value = Vec<Vec<u8>>> {
    prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), min..=max)
}

proptest! {
    #[test]
    fn single_leaf_root_is_leaf_hash(leaf in prop::collection::vec(any::<u8>(), 0..64)) {
        let tree = Tree::from_iter([&leaf]);
        prop_assert_eq!(tree.root(), Some(&Tree::hash_leaf(&leaf)));
        prop_assert_eq!(tree.len(), 1);
        prop_assert!(!tree.is_empty());
    }

    #[test]
    fn two_leaf_tree_root_is_parent_hash(
        a in prop::collection::vec(any::<u8>(), 0..64),
        b in prop::collection::vec(any::<u8>(), 0..64),
    ) {
        let tree = Tree::from_iter([&a, &b]);
        let expected = Tree::hash_branch(
            Tree::hash_leaf(&a),
            Tree::hash_leaf(&b),
        );
        prop_assert_eq!(tree.root(), Some(&expected));
        prop_assert_eq!(tree.len(), 2);
    }

    #[test]
    fn deterministic_construction(leaves in arb_leaves(1, 128)) {
        let tree1 = Tree::from_iter(leaves.iter());
        let tree2 = Tree::from_iter(leaves.iter());
        prop_assert_eq!(tree1.root(), tree2.root());
    }

    #[test]
    fn different_inputs_produce_different_roots(
        a in prop::collection::vec(any::<u8>(), 1..64),
        b in prop::collection::vec(any::<u8>(), 1..64),
    ) {
        prop_assume!(a != b);
        let tree1 = Tree::from_iter([&a, &b]);
        let tree2 = Tree::from_iter([&b, &a]);
        // Swapping inputs should change the root (order matters).
        prop_assert_ne!(tree1.root(), tree2.root());
    }

    #[test]
    fn len_matches_input_count(leaves in arb_leaves(1, 256)) {
        let n = leaves.len();
        let tree = Tree::from_iter(leaves.iter());
        prop_assert_eq!(tree.len(), n);
        prop_assert!(!tree.is_empty());
        prop_assert!(tree.root().is_some());
    }

    #[test]
    fn duplicate_leaves(
        value in prop::collection::vec(any::<u8>(), 0..64),
        count in 2usize..=64,
    ) {
        let leaves: Vec<_> = std::iter::repeat_n(&value, count).collect();
        let tree = Tree::from_iter(leaves);
        prop_assert!(tree.root().is_some());
        prop_assert_eq!(tree.len(), count);
    }

    #[test]
    fn hash_leaf_domain_separation(
        a in prop::collection::vec(any::<u8>(), 0..32),
        b in prop::collection::vec(any::<u8>(), 0..32),
    ) {
        let mut concatenated = a.clone();
        concatenated.extend(&b);
        let leaf = Tree::hash_leaf(&concatenated);
        let branch = Tree::hash_branch(
            Tree::hash_leaf(&a),
            Tree::hash_leaf(&b),
        );
        prop_assert_ne!(leaf, branch);
    }
}
