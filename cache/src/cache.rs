use crate::data::Object;
use crate::fingerprinting::Fingerprint;
use crate::{QDashMap, QueryId};
use dashmap::mapref::one::Ref;
use per_set::PerMap;
use std::sync::Arc;

pub trait Cache {
    fn push(&self, key: QueryId, entry: Cached);
    fn pull(&self, key: &QueryId) -> Option<Ref<QueryId, Cached>>;
    fn remove(&self, key: &QueryId) -> Option<(QueryId, Cached)>;
    fn modify(&self, key: QueryId, f: impl FnOnce(&mut Cached));
}

impl Cache for QDashMap<Cached> {
    fn push(&self, key: QueryId, entry: Cached) {
        self.insert(key, entry);
    }

    fn pull(&self, key: &QueryId) -> Option<Ref<QueryId, Cached>> {
        self.get(key)
    }

    fn remove(&self, key: &QueryId) -> Option<(QueryId, Cached)> {
        self.remove(key)
    }

    fn modify(&self, key: QueryId, f: impl FnOnce(&mut Cached)) {
        self.entry(key).and_modify(f);
    }
}

pub type CacheMap = PerMap<QueryId, Fingerprint>;

pub struct Cached {
    pub result: anyhow::Result<(Fingerprint, Arc<dyn Object>)>,
    pub world_state: CacheMap,
    pub deps_state: CacheMap,
    pub direct_world_state: CacheMap,
}
