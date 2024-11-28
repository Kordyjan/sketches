use proptest::collection::hash_map;
use proptest::{prelude::*, sample::select};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub fn map_with_selected<K>(
    elem_num: usize,
    keys: impl Strategy<Value = K>,
) -> impl Strategy<Value = (HashMap<K, String>, K)>
where
    K: Debug + Hash + Eq + Clone + 'static,
{
    let map = hash_map(keys, ".*", 1..elem_num);
    map.prop_flat_map(|map| {
        let keys = map.keys().map(|k| k.clone()).collect::<Vec<K>>();
        select(keys).prop_map(move |selected| (map.clone(), selected))
    })
}
