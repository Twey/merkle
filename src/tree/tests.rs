use super::*;
use proptest::prelude::*;

#[test]
fn empty_tree_has_no_root() {
    let tree: Tree<()> = [].into_iter().collect();
    assert_eq!(tree.root(), None);
    assert!(tree.is_empty());
    assert_eq!(tree.leaves().len(), 0);
}

fn arb_values(min: usize, max: usize) -> impl Strategy<Value = Vec<i32>> {
    proptest::collection::vec(any::<i32>(), min..=max)
}

proptest! {
    #[test]
    fn leaves_are_preserved(values in arb_values(1, 256)) {
        let tree: Tree<_> = values.iter().copied().collect();
        prop_assert_eq!(tree.leaves(), values.as_slice());
    }

    #[test]
    fn len_matches_input_count(values in arb_values(1, 256)) {
        let n = values.len();
        let tree: Tree<_> = values.iter().copied().collect();
        prop_assert_eq!(tree.leaves().len(), n);
        prop_assert!(!tree.is_empty());
        prop_assert!(tree.root().is_some());
    }

    #[test]
    fn branches_count_is_n_minus_one(values in arb_values(1, 256)) {
        let n = values.len();
        let tree: Tree<_> = values.iter().copied().collect();
        prop_assert_eq!(tree.branches().len(), n - 1);
    }

    #[test]
    fn total_nodes_is_two_n_minus_one(values in arb_values(1, 256)) {
        let n = values.len();
        let tree: Tree<_> = values.iter().copied().collect();
        prop_assert_eq!(tree.branches().len() + tree.leaves().len(), 2 * n - 1);
    }

    #[test]
    fn single_leaf_root_is_value(value in any::<i32>()) {
        let tree: Tree<_> = [value].into_iter().collect();
        prop_assert_eq!(tree.root(), Some(&value));
    }

    #[test]
    fn duplicate_values(value in any::<i32>(), count in 2usize..=64) {
        let tree: Tree<_> = std::iter::repeat_n(value, count).collect();
        prop_assert!(tree.root().is_some());
        prop_assert_eq!(tree.leaves().len(), count);
        prop_assert!(tree.leaves().iter().all(|v| *v == value));
    }
}
