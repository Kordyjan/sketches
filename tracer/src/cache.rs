use std::collections::HashMap;

use cache::{
    cache::{Cache, Cached}, fingerprinting::Fingerprint,
    QDashMap,
    QueryId,
};
use dashmap::{mapref::one::Ref, Entry};
use futures::channel::mpsc::UnboundedSender;
use per_set::PerMap;
use tracer_types::{CacheEntry, Message};

pub struct TracingCache {
    sender: UnboundedSender<Message>,
    map: QDashMap<Cached>,
}

impl TracingCache {
    pub(crate) fn new(sender: UnboundedSender<Message>) -> TracingCache {
        TracingCache {
            sender,
            map: QDashMap::default(),
        }
    }
}

impl Cache for TracingCache {
    fn push(&self, key: QueryId, entry: Cached) {
        let _ = self.sender.unbounded_send(Message::Push {
            key: key.to_string(),
            entry: translate(&entry),
        });
        self.map.push(key.clone(), entry);
    }

    fn pull(&self, key: &QueryId) -> Option<Ref<QueryId, Cached>> {
        let _ = self.sender.unbounded_send(Message::Pull {
            key: key.to_string(),
        });
        self.map.pull(key)
    }

    fn remove(&self, key: &QueryId) -> Option<(QueryId, Cached)> {
        let _ = self.sender.unbounded_send(Message::Remove {
            key: key.to_string(),
        });
        self.map.remove(key)
    }

    fn modify(&self, key: QueryId, f: Box<dyn FnOnce(&mut Cached) + '_>) -> Entry<QueryId, Cached> {
        let res = self.map.modify(key.clone(), f);
        if let Entry::Occupied(e) = &res {
            let _ = self.sender.unbounded_send(Message::Modify {
                key: key.to_string(),
                entry: translate(e.get()),
            });
        }
        res
    }
}

fn translate(cached: &Cached) -> CacheEntry {
    fn transalte_deps(map: &PerMap<QueryId, Fingerprint>) -> HashMap<String, String> {
        map.iter()
            .map(|a| (a.0.to_string(), format!("{:?}", a.1)))
            .collect()
    }

    CacheEntry {
        value: format!("{:?}", cached.result.as_ref().unwrap().1),
        fingerprint: format!("{:?}", cached.result.as_ref().unwrap().0),
        world_state: transalte_deps(&cached.world_state),
        deps_state: transalte_deps(&cached.deps_state),
        direct_world_state: transalte_deps(&cached.direct_world_state),
    }
}
