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
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }
        for (pos, elem) in elems {
            prop_assert_eq!(Some(&elem), sparse_vec.get(pos));
        }
    }

    #[test]
    fn elements_can_be_overwritten((elems, selected) in map_with_selected(5, 0usize..16)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }

        let new_value = "new value".to_string();

        sparse_vec.insert(selected, new_value.clone());
        prop_assert_eq!(Some(&new_value), sparse_vec.get(selected));
        for e in elems.keys().filter(|e| **e != selected) {
            prop_assert_eq!(elems.get(e), sparse_vec.get(*e));
        }
    }

    #[test]
    fn elemenets_can_be_swapped((elems, selected) in map_with_selected(5, 0usize..16)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }

        let new_value = "new value".to_string();

        let old_value = sparse_vec.swap(selected, new_value.clone());
        prop_assert_eq!(Some(&new_value), sparse_vec.get(selected));
        prop_assert_eq!(elems.get(&selected).cloned(), old_value);
        for e in elems.keys().filter(|e| **e != selected) {
            prop_assert_eq!(elems.get(e), sparse_vec.get(*e));
        }
    }

    #[test]
    fn non_existing_elements_can_be_swapped(elems in hash_map(0usize..16, ".*", 0usize..5)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }

        let new_value = "new value".to_string();
        let non_present = (0usize..16).find(|i| !elems.contains_key(i)).unwrap();
        let swapped = sparse_vec.swap(non_present, new_value.clone());
        prop_assert_eq!(None, swapped);
        prop_assert_eq!(Some(&new_value), sparse_vec.get(non_present));
        for e in elems.keys() {
            prop_assert_eq!(elems.get(e), sparse_vec.get(*e));
        }
        prop_assert_eq!(elems.len() + 1, sparse_vec.len());
    }

    #[test]
    fn elements_can_be_removed((elems, selected) in map_with_selected(5, 0usize..16)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }

        let old_value = sparse_vec.remove(selected);
        prop_assert_eq!(None, sparse_vec.get(selected));
        prop_assert_eq!(elems.get(&selected).cloned(), old_value);
    }

    #[test]
    fn not_inserted_elements_are_empty(elems in hash_map(0usize..16, ".*", 0usize..5)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }

        for id in (0usize..16).collect::<HashSet<_>>()
            .difference(&elems.keys().copied()
            .collect::<HashSet<_>>()) {
            prop_assert_eq!(None, sparse_vec.get(*id));
        }
    }

    #[test]
    fn iteration_returns_correctly_ordered_elements(elems in hash_map(0usize..16, ".*", 0usize..5)) {
        let mut sparse_vec = SparseVec::<16, String>::new();
        for (pos, elem) in &elems {
            sparse_vec.insert(*pos, elem.clone());
        }

        let res = sparse_vec.iter().collect::<Vec<_>>();

        let mut expected = elems.iter().collect::<Vec<_>>();
        expected.sort();
        let expected = expected.into_iter().map(|(_, e)| e).collect::<Vec<_>>();

        prop_assert_eq!(expected, res);

    }
}
