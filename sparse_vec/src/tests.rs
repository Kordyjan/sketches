use std::collections::HashSet;

use super::SparseVec;
use proptest::{collection::hash_map, prelude::*};

use test_utils::map_with_selected;

proptest! {
    #[test]
    fn len_is_correct(elems in hash_map(0usize..16, ".*", 0usize..5)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        let len = elems.len();
        for (pos, elem) in elems {
            sparse_vec.insert(pos, elem);
        }
        prop_assert_eq!(len, sparse_vec.len());
    }

    #[test]
    fn elements_can_be_retrieved(elems in hash_map(0usize..16, ".*", 0usize..5)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in elems.iter() {
            sparse_vec.insert(*pos, elem.clone());
        }
        for (pos, elem) in elems {
            prop_assert_eq!(Some(&elem), sparse_vec.get(pos));
        }
    }

    #[test]
    fn elements_can_be_overwritten((elems, selected) in map_with_selected(5, 0usize..16)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in elems.iter() {
            sparse_vec.insert(*pos, elem.clone());
        }

        let new_value = "new value".to_string();

        sparse_vec.insert(selected, new_value.clone());
        prop_assert_eq!(Some(&new_value), sparse_vec.get(selected));

    }

    #[test]
    fn elemenets_can_be_swapped((elems, selected) in map_with_selected(5, 0usize..16)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in elems.iter() {
            sparse_vec.insert(*pos, elem.clone());
        }

        let new_value = "new value".to_string();

        let old_value = sparse_vec.swap(selected, new_value.clone());
        prop_assert_eq!(Some(&new_value), sparse_vec.get(selected));
        prop_assert_eq!(elems.get(&selected).map(String::clone), old_value)
    }

    #[test]
    fn elements_can_be_removed((elems, selected) in map_with_selected(5, 0usize..16)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in elems.iter() {
            sparse_vec.insert(*pos, elem.clone());
        }

        let old_value = sparse_vec.remove(selected);
        prop_assert_eq!(None, sparse_vec.get(selected));
        prop_assert_eq!(elems.get(&selected).map(String::clone), old_value)
    }

    #[test]
    fn not_inserted_elements_are_empty(elems in hash_map(0usize..16, ".*", 0usize..5)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in elems.iter() {
            sparse_vec.insert(*pos, elem.clone());
        }

        for id in (0usize..16).collect::<HashSet<_>>()
            .difference(&elems.keys()
            .map(|u| *u)
            .collect::<HashSet<_>>()) {
            prop_assert_eq!(None, sparse_vec.get(*id));
            ()
        }
    }
}
