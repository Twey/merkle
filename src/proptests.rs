use super::*;
use proptest::prelude::*;

fn arb_leaves(min: usize, max: usize) -> impl Strategy<Value = Vec<Vec<u8>>> {
    prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), min..=max)
}

fn arb_tree_and_index(min_leaves: usize, max_leaves: usize) -> impl Strategy<Value = (Tree<Hash>, Index)> {
    arb_leaves(min_leaves, max_leaves).prop_flat_map(|leaves| {
        let len = leaves.len();
        let tree = Tree::from_iter(leaves.iter());
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
        bogus in prop::collection::vec(any::<u8>(), 0..64),
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
        leaves in arb_leaves(4, 256),
        index_a in any::<prop::sample::Index>(),
        index_b in any::<prop::sample::Index>(),
    ) {
        // Ensure all leaves are distinct so any index swap changes the content.
        prop_assume!(leaves.iter().collect::<std::collections::HashSet<_>>().len() == leaves.len());
        let tree = Tree::from_iter(leaves.iter());
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
        let bogus_root = Hash(bogus_root);
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
        bad_siblings[0] = Hash(bogus_sibling);
        prop_assume!(bad_siblings != proof.preproof.siblings);
        let tampered = Preproof {
            siblings: bad_siblings,
            ..proof.preproof
        };
        prop_assert!(tampered.verify().is_err());
    }

    #[test]
    fn duplicate_values_can_still_be_proven_by_index(
        value in prop::collection::vec(any::<u8>(), 0..64),
        count in 2usize..=64,
    ) {
        let leaves: Vec<_> = std::iter::repeat_n(&value, count).collect();
        let tree = Tree::from_iter(leaves);
        for i in 0..count {
            let proof = tree.prove(i).expect("prove should succeed");
            proof.preproof.verify().expect("proof should verify");
        }
    }

    #[test]
    fn proof_out_of_bounds(leaves in arb_leaves(1, 128)) {
        let len = leaves.len();
        let tree = Tree::from_iter(leaves.iter());
        prop_assert!(tree.prove(len).is_err());
        prop_assert!(tree.prove(len + 1).is_err());
    }
}
