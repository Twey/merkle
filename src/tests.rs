use super::*;

#[test]
fn empty_tree_has_no_root() {
    let tree = Tree::<Hash>::default();
    assert_eq!(tree.root(), None);
}

#[test]
fn single_leaf_root_is_leaf_hash() {
    let tree = Tree::from_iter(["a"]);
    assert_eq!(tree.root(), Some(&Tree::hash_leaf(b"a")));
}

#[test]
fn two_leaf_tree_root_is_parent_hash() {
    let tree = Tree::from_iter(["a", "b"]);
    let expected = Tree::hash_branch(
        Tree::hash_leaf(b"a"),
        Tree::hash_leaf(b"b"),
    );
    assert_eq!(tree.root(), Some(&expected));
}

#[test]
fn odd_leaf_count_is_handled_deterministically() {
    let tree1 = Tree::from_iter(["a", "b", "c"]);
    let tree2 = Tree::from_iter(["a", "b", "c"]);
    assert_eq!(tree1.root(), tree2.root());
    // With 3 leaves the tree must still produce a definite root.
    assert!(tree1.root().is_some());
}

#[test]
fn proof_verifies_for_each_leaf() {
    let values = ["a", "b", "c", "d"];
    let tree = Tree::from_iter(values);
    for i in 0..values.len() {
        let proof = tree.prove(i).expect("prove should succeed");
        proof.preproof.verify().expect("proof should verify");
    }
}

#[test]
fn proof_fails_for_wrong_leaf() {
    let tree = Tree::from_iter(["a", "b", "c", "d"]);
    let proof = tree.prove(0).unwrap();
    let tampered = Preproof {
        content: Tree::hash_leaf(b"z"),
        ..proof.preproof
    };
    assert!(tampered.verify().is_err());
}

#[test]
fn proof_fails_for_wrong_index() {
    let tree = Tree::from_iter(["a", "b", "c", "d"]);
    let proof = tree.prove(0).unwrap();
    let tampered = Preproof {
        node: 1,
        ..proof.preproof
    };
    assert!(tampered.verify().is_err());
}

#[test]
fn proof_fails_for_wrong_root() {
    let tree = Tree::from_iter(["a", "b", "c", "d"]);
    let proof = tree.prove(0).unwrap();
    let tampered = Preproof {
        root: Hash([0xffu8; 32]),
        ..proof.preproof
    };
    assert!(tampered.verify().is_err());
}

#[test]
fn proof_fails_when_proof_is_tampered() {
    let tree = Tree::from_iter(["a", "b", "c", "d"]);
    let proof = tree.prove(0).unwrap();
    let mut bad_siblings = proof.preproof.siblings.clone();
    if let Some(first) = bad_siblings.first_mut() {
        *first = Hash([0xffu8; 32]);
    }
    let tampered = Preproof {
        siblings: bad_siblings,
        ..proof.preproof
    };
    assert!(tampered.verify().is_err());
}

#[test]
fn duplicate_values_can_still_be_proven_by_index() {
    let tree = Tree::from_iter(["a", "a", "a", "a"]);
    for i in 0..4 {
        let proof = tree.prove(i).expect("prove should succeed");
        proof.preproof.verify().expect("proof should verify");
    }
}

#[test]
fn large_tree_builds_and_verifies() {
    let tree = Tree::from_iter((0..1000).map(|i| format!("leaf_{i}")));

    assert_eq!(tree.len(), 1000);
    assert!(tree.root().is_some());

    // Verify a sample of proofs across the tree.
    for &i in &[0, 1, 499, 500, 998, 999] {
        let proof = tree.prove(i).expect("prove should succeed");
        // assert_eq!(proof.preproof.index, i);
        proof.preproof.verify().expect("proof should verify");
    }
}

#[test]
fn small_verify() {
    let tree = Tree::from_iter((0..3).map(|i| format!("leaf {i}")));
    let proof = tree.prove(1).unwrap();
    dbg!((&tree, &proof.preproof));
    proof.preproof.verify().unwrap();
}
