use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash, RandomState},
    marker::PhantomData,
};

#[cfg(test)]
mod tests;

pub struct PerMap<K, V, S = RandomState>(PhantomData<K>, PhantomData<V>, PhantomData<S>);

impl<K, V> PerMap<K, V, RandomState> {
    pub fn empty() -> Self {
        Self(PhantomData, PhantomData, PhantomData)
    }
}

impl<K, V> Default for PerMap<K, V> {
    fn default() -> Self {
        PerMap::<K, V>::empty()
    }
}

impl<K, V, S> PerMap<K, V, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

impl<K, V, S> PerMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    pub fn insert(&self, key: K, value: V) -> Self {
        todo!()
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
