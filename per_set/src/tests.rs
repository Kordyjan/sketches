use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hasher},
};

use super::PerMap;
use proptest::{collection::hash_map, prelude::*};
use test_utils::map_with_selected;

proptest! {
    #[test]
    fn len_is_correct(elems in hash_map(0u64..1024, ".*", 0usize..16)) {
        let map = PerMap::<u64, String>::empty();
        let len = elems.len();

        let map = elems.into_iter().fold(map, |m, (k, v)| m.insert(k, v));

        prop_assert_eq!(len, map.len());
    }

    #[test]
    fn values_are_retrieved(elems in hash_map(0u64..1024, ".*", 0usize..16)) {
        let map = PerMap::<u64, String>::empty();
        let map = elems.iter().fold(map, |m, (k, v)| m.insert(*k, v.clone()));

        for (k, v) in elems {
            prop_assert_eq!(Some(&v), map.get(&k));
        }
    }

    #[test]
    fn values_can_be_updated_but_older_snapshots_remain((elems, selected) in map_with_selected(16, 0u64..1024)) {
        let map = PerMap::<u64, String>::empty();

        let map = elems.iter().fold(map, |m, (k, v)| m.insert(*k, v.clone()));

        let new_value = "new value".to_string();
        let new_map = map.insert(selected, new_value.clone());

        let old_value = elems.get(&selected);
        prop_assert_eq!(Some(&new_value), new_map.get(&selected));
        prop_assert_eq!(old_value, map.get(&selected));
    }

    #[test]
    fn map_can_handle_hash_clashes(elems in hash_map(0u64..1024, ".*", 0usize..16)) {
        let map = PerMap::<u64, String, DegenerateBuildHasher>::with_hasher(DegenerateBuildHasher);

        let map = elems.iter().fold(map, |m, (k, v)| m.insert(*k, v.clone()));

        let len = elems.len();
        prop_assert_eq!(len, map.len());

        for (k, v) in elems {
            prop_assert_eq!(Some(&v), map.get(&k));
        }
    }

    #[test]
    fn union_of_maps_preserves_all_keys_and_has_values_from_right_side(
        left_only in hash_map(0u64..1024, ".*", 0usize..16),
        right_only in hash_map(1024u64..2048, ".*", 0usize..16),
        common in hash_map(0u64..1024, ".*", 0usize..16),
    ) {
        let left = left_only.iter()
            .chain(common.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<_, _>>();

        let right = right_only.iter()
            .chain(common.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<_, _>>();

        let left_map = left.iter().fold(PerMap::<u64, String>::empty(), |m, (k, v)| m.insert(*k, v.clone()));
        let right_map = right.iter().fold(PerMap::<u64, String>::empty(), |m, (k, v)| m.insert(*k, v.clone()));

        let keys = left_only.keys().chain(right_only.keys()).chain(common.keys()).map(|u| *u).collect::<HashSet<_>>();

        let res = left_map.union(&right_map);

        prop_assert_eq!(keys.len(), res.len());
        for k in keys {
            prop_assert!(res.get(&k).is_some())
        }

        for (k, v) in right {
            prop_assert_eq!(Some(&v), res.get(&k));
        }
    }

}

#[derive(Clone)]
struct DegenerateBuildHasher;

struct DegenerateHasher;

impl BuildHasher for DegenerateBuildHasher {
    type Hasher = DegenerateHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DegenerateHasher
    }
}

impl Hasher for DegenerateHasher {
    fn finish(&self) -> u64 {
        7
    }

    fn write(&mut self, _: &[u8]) {}

    fn write_u64(&mut self, _: u64) {}
}
