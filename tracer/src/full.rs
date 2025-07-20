use std::collections::HashMap;

use futures::channel::mpsc::UnboundedSender;
use per_set::PerMap;
use queries::{
    execution::{Cached, ExecutionContext},
    fingerprinting::Fingerprint,
    trace::Trace,
    QueryId,
};
use tracer_types::{CacheEntry, Message};

pub struct Tracer(pub(crate) UnboundedSender<Message>);

impl Trace for Tracer {
    fn cache_push(&self, key: &QueryId, entry: &Cached, context: Option<&ExecutionContext>) {
        let _ = self.0.unbounded_send(Message::Push {
            stack: stack(context),
            key: key.to_string(),
            entry: translate(&entry),
        });
    }

    fn cache_pull(&self, key: &QueryId, reason: &'static str, context: Option<&ExecutionContext>) {
        let _ = self.0.unbounded_send(Message::Pull {
            key: key.to_string(),
            reason: reason.to_owned(),
            stack: stack(context),
        });
    }

    fn cache_remove(&self, key: &QueryId, context: Option<&ExecutionContext>) {
        let _ = self.0.unbounded_send(Message::Remove {
            key: key.to_string(),
            stack: stack(context),
        });
    }

    fn cache_modify(&self, key: &QueryId, entry: &Cached, context: Option<&ExecutionContext>) {
        let _ = self.0.unbounded_send(Message::Modify {
            key: key.to_string(),
            entry: translate(entry),
            stack: stack(context),
        });
    }

    fn body_run(&self, key: &QueryId, context: Option<&ExecutionContext>) {
        let _ = self.0.unbounded_send(Message::BodyExecuted {
            key: key.to_string(),
            stack: stack(context),
        });
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

fn stack(context: Option<&ExecutionContext>) -> Vec<String> {
    let Some(ctx) = context else {
        return Vec::new();
    };
    ctx.trace_iter().map(|id| format!("{id}")).collect()
}
