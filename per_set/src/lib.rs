use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash, Hasher, RandomState},
    sync::Arc,
};

use nodes::{BitShifter, Node};

mod nodes;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct PerMap<K, V, S = RandomState> {
    data: Arc<Node<K, V>>,
    hasher: S,
}

impl<K, V> PerMap<K, V, RandomState> {
    pub fn empty() -> Self {
        Self::with_hasher(RandomState::new())
    }
}

impl<K, V> Default for PerMap<K, V> {
    fn default() -> Self {
        PerMap::<K, V>::empty()
    }
}

impl<K, V, S> PerMap<K, V, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            data: Arc::default(),
            hasher: hash_builder,
        }
    }

    pub fn len(&self) -> usize {
        self.data.weight()
    }

    pub fn is_empty(&self) -> bool {
        self.data.weight() == 0
    }
}

impl<K, V, S> PerMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    pub fn insert(&self, key: K, value: V) -> Self {
        let mut state = self.hasher.build_hasher();
        key.hash(&mut state);
        let hash = state.finish();
        let address = BitShifter::new(hash);
        let new_data = Node::insert(&self.data, key, value, address);
        Self {
            data: new_data,
            hasher: self.hasher.clone(),
        }
    }

    pub fn get<Q>(&self, key: &K) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        todo!()
    }

    pub fn union(&self, other: &PerMap<K, V, S>) -> Self {
        todo!()
    }
}
