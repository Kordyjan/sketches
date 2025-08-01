use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    sync::Arc,
};

use rustc_hash::FxBuildHasher;

use crate::nodes::{BitShifter, Node};

#[derive(Clone, Debug)]
pub struct PerMap<K, V, S = FxBuildHasher> {
    pub(crate) data: Arc<Node<K, V>>,
    hasher: S,
}

impl<K, V> PerMap<K, V, FxBuildHasher> {
    #[must_use]
    pub fn empty() -> Self {
        Self::with_hasher(FxBuildHasher)
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

    pub fn iter(&self) -> crate::iter::Iter<'_, K, V> {
        crate::iter::Iter::new(self)
    }
}

impl<K, V, S> PerMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    #[must_use]
    pub fn insert(&self, key: K, value: V) -> Self {
        let hash = self.hasher.hash_one(&key);
        let address = BitShifter::new(hash);
        let new_data = Node::insert(&self.data, key, value, address);
        Self {
            data: new_data,
            hasher: self.hasher.clone(),
        }
    }

    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = self.hasher.hash_one(key);
        let address = BitShifter::new(hash);
        let res = self.data.get(key, address);
        res
    }

    #[must_use]
    pub fn union(&self, other: &PerMap<K, V, S>) -> Self {
        PerMap {
            data: Node::merge(&self.data, &other.data),
            hasher: self.hasher.clone(),
        }
    }
}

impl<'a, K, V, S> IntoIterator for &'a PerMap<K, V, S> {
    type Item = &'a Arc<(K, V)>;

    type IntoIter = crate::iter::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        crate::iter::Iter::new(self)
    }
}
