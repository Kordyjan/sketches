use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    ops::Deref,
    sync::Arc,
};

use rustc_hash::FxBuildHasher;

use crate::{iter, PerMap};

#[derive(Debug, Clone)]
pub struct PerSet<K, S = FxBuildHasher>(PerMap<K, (), S>);

impl<K> PerSet<K> {
    #[must_use]
    pub fn empty() -> Self {
        PerSet(PerMap::empty())
    }
}

impl<K> Default for PerSet<K> {
    fn default() -> Self {
        PerSet::<K>::empty()
    }
}

impl<K, S> PerSet<K, S> {
    #[must_use]
    pub fn with_hasher(hash_builder: S) -> Self {
        PerSet(PerMap::with_hasher(hash_builder))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, S> PerSet<K, S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    #[must_use]
    pub fn insert(&self, key: K) -> Self {
        PerSet(self.0.insert(key, ()))
    }

    #[must_use]
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.0.get(key).is_some()
    }

    #[must_use]
    pub fn union(&self, other: &PerSet<K, S>) -> Self {
        PerSet(self.0.union(&other.0))
    }
}

pub struct Element<'a, K>(&'a Arc<(K, ())>);

impl<T> Deref for Element<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().0
    }
}

pub struct Iter<'a, K>(pub(crate) iter::Iter<'a, K, ()>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = Element<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Element)
    }
}
