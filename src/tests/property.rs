use crate::*;
use proptest::prelude::*;

type Tree = crate::Tree<sha2::Sha256>;
type Preproof = crate::Preproof<sha2::Sha256>;
type Hash = digest::Output<sha2::Sha256>;

fn arb_leaves(min: usize, max: usize) -> impl Strategy<Value = Vec<Vec<u8>>> {
    proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..64), min..=max)
}

/// Generates a `Vec` of distinct leaves by using the index as a prefix.
fn arb_distinct_leaves(min: usize, max: usize) -> impl Strategy<Value = Vec<Vec<u8>>> {
    proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..60), min..=max).prop_map(
        |leaves| {
            leaves
                .into_iter()
                .enumerate()
                .map(|(i, mut leaf)| {
                    leaf.extend_from_slice(&(i as u32).to_le_bytes());
                    leaf
                })
                .collect()
        },
    )
}

fn arb_tree_and_index(
    min_leaves: usize,
    max_leaves: usize,
) -> impl Strategy<Value = (Tree, Index)> {
    arb_leaves(min_leaves, max_leaves).prop_flat_map(|leaves| {
        let len = leaves.len();
        let tree: Tree = leaves.iter().collect();
        (Just(tree), 0..len)
    })
}

proptest! {
    #[test]
    fn proof_verifies_for_each_leaf((tree, index) in arb_tree_and_index(1, 256)) {
        let proof = tree.prove(index).expect("prove should succeed");
        proof.preproof.verify().expect("proof should verify");
    }

    #[test]
    fn proof_fails_for_wrong_leaf(
        (tree, index) in arb_tree_and_index(1, 256),
        bogus in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let proof = tree.prove(index).unwrap();
        let bogus_hash = Tree::hash_leaf(&bogus);
        prop_assume!(bogus_hash != proof.preproof.content);
        let tampered = Preproof {
            content: bogus_hash,
            ..proof.preproof
        };
        prop_assert!(tampered.verify().is_err());
    }

    #[test]
    fn proof_fails_for_wrong_index(
        leaves in arb_distinct_leaves(4, 256),
        index_a in any::<proptest::sample::Index>(),
        index_b in any::<proptest::sample::Index>(),
    ) {
        let tree: Tree = leaves.iter().collect();
        let n = tree.len();
        let a = index_a.index(n);
        let b = index_b.index(n);
        prop_assume!(a != b);
        // Take proof for leaf a, but claim it's for leaf b.
        let proof = tree.prove(a).unwrap();
        let proof_b = tree.prove(b).unwrap();
        let tampered = Preproof {
            node: proof_b.preproof.node,
            ..proof.preproof
        };
        prop_assert!(tampered.verify().is_err());
    }

    #[test]
    fn proof_fails_for_wrong_root(
        (tree, index) in arb_tree_and_index(1, 256),
        bogus_root in any::<[u8; 32]>(),
    ) {
        let proof = tree.prove(index).unwrap();
        let bogus_root: Hash = bogus_root.into();
        prop_assume!(bogus_root != proof.preproof.root);
        let tampered = Preproof {
            root: bogus_root,
            ..proof.preproof
        };
        prop_assert!(tampered.verify().is_err());
    }

    #[test]
    fn proof_fails_when_sibling_is_tampered(
        (tree, index) in arb_tree_and_index(2, 256),
        bogus_sibling in any::<[u8; 32]>(),
    ) {
        let proof = tree.prove(index).unwrap();
        prop_assume!(!proof.preproof.siblings.is_empty());
        let mut bad_siblings = proof.preproof.siblings.clone();
        bad_siblings[0] = bogus_sibling.into();
        prop_assume!(bad_siblings != proof.preproof.siblings);
        let tampered = Preproof {
            siblings: bad_siblings,
            ..proof.preproof
        };
        prop_assert!(tampered.verify().is_err());
    }

    #[test]
    fn duplicate_values_can_still_be_proven_by_index(
        value in proptest::collection::vec(any::<u8>(), 0..64),
        count in 2usize..=64,
    ) {
        let tree: Tree = std::iter::repeat_n(&value, count).collect();
        for i in 0..count {
            let proof = tree.prove(i).expect("prove should succeed");
            proof.preproof.verify().expect("proof should verify");
        }
    }

    #[test]
    fn proof_out_of_bounds(leaves in arb_leaves(1, 128)) {
        let len = leaves.len();
        let tree: Tree = leaves.iter().collect();
        prop_assert!(tree.prove(len).is_err());
        prop_assert!(tree.prove(len + 1).is_err());
    }

    #[test]
    fn single_leaf_root_is_leaf_hash(leaf in proptest::collection::vec(any::<u8>(), 0..64)) {
        let tree: Tree = [&leaf].into_iter().collect();
        prop_assert_eq!(tree.root(), Some(&Tree::hash_leaf(&leaf)));
        prop_assert_eq!(tree.len(), 1);
        prop_assert!(!tree.is_empty());
    }

    #[test]
    fn two_leaf_tree_root_is_parent_hash(
        a in proptest::collection::vec(any::<u8>(), 0..64),
        b in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let tree: Tree = [&a, &b].into_iter().collect();
        let expected = Tree::hash_branch(
            Tree::hash_leaf(&a),
            Tree::hash_leaf(&b),
        );
        prop_assert_eq!(tree.root(), Some(&expected));
        prop_assert_eq!(tree.len(), 2);
    }

    #[test]
    fn deterministic_construction(leaves in arb_leaves(1, 128)) {
        let tree1: Tree = leaves.iter().collect();
        let tree2: Tree = leaves.iter().collect();
        prop_assert_eq!(tree1.root(), tree2.root());
    }

    #[test]
    fn different_inputs_produce_different_roots(
        a in proptest::collection::vec(any::<u8>(), 1..64),
        b in proptest::collection::vec(any::<u8>(), 1..64),
    ) {
        prop_assume!(a != b);
        let tree1: Tree = [&a, &b].into_iter().collect();
        let tree2: Tree = [&b, &a].into_iter().collect();
        prop_assert_ne!(tree1.root(), tree2.root());
    }

    #[test]
    fn len_matches_input_count(leaves in arb_leaves(1, 256)) {
        let n = leaves.len();
        let tree: Tree = leaves.iter().collect();
        prop_assert_eq!(tree.len(), n);
        prop_assert!(!tree.is_empty());
        prop_assert!(tree.root().is_some());
    }

    #[test]
    fn hash_leaf_domain_separation(
        a in proptest::collection::vec(any::<u8>(), 0..32),
        b in proptest::collection::vec(any::<u8>(), 0..32),
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
