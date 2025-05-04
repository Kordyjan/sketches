use crate::data::{ErasedResponse, QueryResponse};
use crate::{
    ErasedQuery, Executor, Query,
    data::{Object, Param, QueryId},
    fingerprinting::{Fingerprint, stamp_with_fingerprint},
};
use ::core::future::Future;
use anyhow::{Context, Result, anyhow, bail};
use async_std::{
    channel::Sender,
    channel::{self, Receiver},
    stream::StreamExt,
    sync::Mutex,
};
use dashmap::{DashMap, DashSet};
use futures::FutureExt;
use futures::stream::FuturesUnordered;
use per_set::{PerMap, PerSet};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use std::{
    iter,
    pin::Pin,
    sync::Arc,
    task::{Context as WakeContext, Poll, Waker},
};

type CacheMap = PerMap<QueryId, Fingerprint>;

struct Cached {
    result: Result<(Fingerprint, Arc<dyn Object>)>,
    world_state: CacheMap,
    deps_state: CacheMap,
}

type QDashMap<V> = DashMap<QueryId, V, FxBuildHasher>;

pub struct Reactor {
    params: QDashMap<(Fingerprint, Arc<dyn Object>)>,
    trace: Mutex<Vec<String>>,
    trace_sender: Sender<String>,
    trace_receiver: Receiver<String>,
    cache: QDashMap<Cached>,
    current: QDashMap<SmallVec<[Waker; 4]>>,
    past_queries: QDashMap<ErasedQuery>,
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}

impl Reactor {
    pub fn new() -> Self {
        let (trace_sender, trace_receiver) = channel::unbounded();
        Reactor {
            params: QDashMap::default(),
            trace: Mutex::new(Vec::new()),
            trace_sender,
            trace_receiver,
            cache: QDashMap::default(),
            current: QDashMap::default(),
            past_queries: QDashMap::default(),
        }
    }

    fn new_continuity(self: &Arc<Self>) -> Continuity {
        Continuity::new(Arc::clone(self))
    }

    fn do_execute<Q, T>(
        self: &Arc<Self>,
        query: Q,
        view_parent: impl Into<Option<Arc<ExecutionView>>>,
        continuity: &Arc<Continuity>,
    ) -> impl Future<Output = Result<(Fingerprint, T::Boxed)>>
    where
        Q: Query<Response = T> + Send + Sync + 'static,
        T: QueryResponse,
    {
        let (world_sender, world_receiver) = channel::unbounded();
        let (direct_sender, direct_receiver) = channel::unbounded();
        let view_parent = view_parent.into();
        let dependents = if let Some(view_parent) = &view_parent {
            &view_parent.dependents
        } else {
            &PerSet::empty()
        };
        let dependents = dependents.insert(query.id().clone());
        let view_wrapper = ExecutionContext(Arc::new(ExecutionView {
            continuity: Arc::clone(continuity),
            current: query.id(),
            parent: view_parent,
            dependents,
            world_dependencies: world_sender,
            direct_dependencies: direct_sender,
        }));
        DoExecute::new(self, query, view_wrapper, world_receiver, direct_receiver)
    }

    fn start_processing<Q, T>(
        self: &Arc<Self>,
        query: Q,
        view_wrapper: ExecutionContext,
        world_receiver: Receiver<CacheMap>,
        direct_receiver: Receiver<CacheMap>,
    ) where
        Q: Query<Response = T> + Send + Sync + 'static,
        T: QueryResponse,
    {
        let reactor = Arc::clone(self);
        let handle = async_global_executor::spawn(async move {
            let id = query.id();

            let cache_correct = match reactor.cache.get(&id) {
                Some(cached) if reactor.verify(&cached.world_state) => true,
                Some(cached) => {
                    use futures::StreamExt;
                    if cached.deps_state.is_empty() {
                        false
                    } else {
                        let deps_state = cached.deps_state.clone();
                        drop(cached);
                        let iter = deps_state.iter().map(|state| async {
                            let Some(query) = reactor.past_queries.get(&state.0) else {
                                return false;
                            };
                            let q = query.clone();
                            drop(query);
                            let continuity = Arc::new(reactor.new_continuity());
                            let result = reactor.do_execute(q, None, &continuity).await;
                            let res = matches!(result, Ok((f, _)) if f == state.1);
                            res
                        });

                        // Collect all results first
                        let stream = iter.collect::<FuturesUnordered<_>>();
                        let results = stream.collect::<Vec<_>>().await;
                        // Then check if all are true
                        let res = results.iter().all(|&a| a);
                        res
                    }
                }
                None => false,
            };

            if !cache_correct {
                reactor.cache.remove(&id);
                let result = query.body(&view_wrapper).await;
                view_wrapper.0.world_dependencies.close();
                let world_dependencies = world_receiver
                    .fold(PerMap::empty(), |acc, d| acc.union(&d))
                    .await;
                if let Some(parent) = &view_wrapper.0.parent {
                    let Ok(()) = parent
                        .world_dependencies
                        .send(world_dependencies.clone())
                        .await
                    else {
                        panic!(
                            "Could not send world dependencies to the parent of {}",
                            query.id().clone()
                        )
                    };
                }
                view_wrapper.0.direct_dependencies.close();
                let direct_dependencies = direct_receiver
                    .fold(PerMap::empty(), |acc, d| acc.union(&d))
                    .await;
                let obj = result.map(|v| stamp_with_fingerprint(v.into_object()));
                reactor.cache.insert(
                    query.id().clone(),
                    Cached {
                        result: obj,
                        world_state: world_dependencies,
                        deps_state: direct_dependencies,
                    },
                );
                reactor
                    .trace_sender
                    .send(id.to_string())
                    .await
                    .expect("Trace channel is broken.");
            }

            reactor.past_queries.entry(id.clone()).or_insert_with(|| {
                ErasedQuery(
                    id.clone(),
                    Arc::new(move |ctx| {
                        let query = query.clone();
                        async move {
                            query
                                .body(ctx)
                                .await
                                .map(|r| ErasedResponse(r.into_object()))
                        }
                        .boxed()
                    }),
                )
            });
            reactor.wake(&id);
        });

        handle.detach();
    }

    fn wake(&self, query_id: &QueryId) {
        if let Some((_, mut wakers)) = self.current.remove(query_id) {
            for waker in wakers.drain(..) {
                waker.wake();
            }
        } else {
            panic!("Calling `wake` but no waker is waiting for {query_id}")
        }
    }

    fn verify(&self, world_state: &PerMap<QueryId, Fingerprint>) -> bool {
        world_state
            .iter()
            .all(|st| self.params.get(&st.0).is_none_or(|p| p.0 == st.1))
    }
}

impl Executor for Arc<Reactor> {
    fn set_param<T: Object>(&self, param: &Param<T>, value: T) {
        self.params.insert(
            param.query_id().clone(),
            stamp_with_fingerprint(Arc::new(value)),
        );
    }

    fn execute<T, Q>(&self, query: Q) -> impl Future<Output = Result<T::Boxed>>
    where
        Q: Query<Response = T> + Sync + Send + 'static,
        T: QueryResponse,
    {
        self.new_continuity().drive(query)
    }

    async fn trace(&self) -> Vec<String> {
        let mut lock = self.trace.lock().await;
        while let Ok(message) = self.trace_receiver.try_recv() {
            lock.push(message);
        }
        Vec::clone(&*lock)
    }
}

struct DoExecute<Q> {
    reactor: Arc<Reactor>,
    query_and_view: Option<(Q, ExecutionContext)>,
    query_id: QueryId,
    world_receiver: Receiver<CacheMap>,
    direct_receiver: Receiver<CacheMap>,
}

impl<Q: Query> DoExecute<Q> {
    fn new(
        reactor: &Arc<Reactor>,
        query: Q,
        view_wrapper: ExecutionContext,
        world_receiver: Receiver<CacheMap>,
        direct_receiver: Receiver<CacheMap>,
    ) -> Self {
        DoExecute {
            reactor: Arc::clone(reactor),
            query_id: query.id(),
            query_and_view: Some((query, view_wrapper)),
            world_receiver,
            direct_receiver,
        }
    }
}

impl<Q> Future for DoExecute<Q>
where
    Q::Response: QueryResponse,
    Q: Query,
{
    type Output = Result<(
        Fingerprint,
        <<Q as Query>::Response as QueryResponse>::Boxed,
    )>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut WakeContext<'_>) -> Poll<Self::Output> {
        if let Some((query, view)) = self.as_mut().query_and_view.take() {
            let mut entry = self.reactor.current.entry(query.id()).or_default();
            let len = entry.len();
            entry.push(cx.waker().clone());
            if len == 0 {
                self.reactor.start_processing(
                    query,
                    view,
                    self.world_receiver.clone(),
                    self.direct_receiver.clone(),
                );
            }
            Poll::Pending
        } else if let Some(res2) = self.reactor.cache.get(&self.query_id) {
            let res = &res2.result;
            let res = res
                .as_ref()
                .map_err(|e| anyhow!("Error in dependency: {}", e))
                .and_then(|(fingerprint, arc)| {
                    Q::Response::downcast(arc.clone())
                        .with_context(|| {
                            format!("Conflicting past_queries with id: {}", self.query_id)
                        })
                        .map(|response| (*fingerprint, response))
                });
            Poll::Ready(res)
        } else {
            Poll::Pending
        }
    }
}

impl<Q> Unpin for DoExecute<Q> {}

struct Continuity {
    reactor: Arc<Reactor>,
    fresh_queries: DashSet<QueryId, FxBuildHasher>,
    world_deps_states: QDashMap<Fingerprint>,
}

impl Continuity {
    pub fn new(reactor: Arc<Reactor>) -> Self {
        Continuity {
            reactor,
            fresh_queries: DashSet::default(),
            world_deps_states: DashMap::default(),
        }
    }

    pub fn drive<Q, T>(self, query: Q) -> impl Future<Output = Result<T::Boxed>>
    where
        Q: Query<Response = T>,
        T: QueryResponse,
    {
        async move {
            let (_, res) = Arc::new(self).do_execute(query, None).await?;
            Ok(res)
        }
    }

    async fn do_execute<Q, T>(
        self: Arc<Self>,
        query: Q,
        parent_view: impl Into<Option<Arc<ExecutionView>>>,
    ) -> Result<(Fingerprint, T::Boxed)>
    where
        Q: Query<Response = T>,
        T: QueryResponse,
    {
        let id = query.id().clone();
        let result: (Fingerprint, T::Boxed) = if self.fresh_queries.contains(&query.id()) {
            let cached_result = &self
                .reactor
                .cache
                .get(&query.id())
                .context("Cache was corrupted")?
                .result;
            let Ok((fingerprint, value)) = cached_result.as_ref() else {
                bail!("Query {id} in cache was overriden with failed execution.")
            };

            let value = T::downcast(value.clone())
                .with_context(|| format!("Conflicting past_queries with id: {}", query.id()))?;

            (*fingerprint, value)
        } else {
            self.reactor
                .do_execute(query, parent_view, &self)
                .await
                .with_context(|| format!("as a part of {id}"))?
        };
        self.fresh_queries.insert(id);
        Ok(result)
    }
}

struct ExecutionView {
    current: QueryId,
    parent: Option<Arc<ExecutionView>>,
    dependents: PerSet<QueryId>,
    continuity: Arc<Continuity>,
    world_dependencies: Sender<PerMap<QueryId, Fingerprint>>,
    direct_dependencies: Sender<PerMap<QueryId, Fingerprint>>,
}

pub struct ExecutionContext(Arc<ExecutionView>);

impl ExecutionView {
    fn trace_until<'a>(&'a self, id: &'a QueryId) -> impl Iterator<Item = QueryId> + use<'a> {
        let mut found: bool = false;
        self.trace_iter().take_while(move |e| {
            if found {
                false
            } else if e == id {
                found = true;
                true
            } else {
                true
            }
        })
    }

    fn trace_iter<'a>(&'a self) -> impl Iterator<Item = QueryId> + use<'a> {
        let mut current: Option<&'a ExecutionView> = Some(self);
        iter::from_fn(move || {
            let res = current.map(|c| c.current.clone());
            current = current.and_then(|c| c.parent.as_ref()).map(|c| &**c);
            res
        })
    }
}

impl ExecutionContext {
    pub fn get_param<T: Object>(
        &self,
        param: &Param<T>,
    ) -> impl Future<Output = Result<Arc<T>>> + Send {
        async move {
            let Some(pair) = self.0.continuity.reactor.params.get(param.query_id()) else {
                bail!("No param with id {}", param.query_id())
            };

            let (fingerprint, value) = &*pair;

            let result = value
                .clone()
                .as_any()
                .downcast::<T>()
                .map_err(|_| anyhow!("Conflicting params with id {}", param.query_id()))?;
            self.0
                .world_dependencies
                .send(PerMap::empty().insert(param.query_id().clone(), *fingerprint))
                .await?;
            Ok(result)
        }
    }

    pub fn run<T, Q>(&self, query: Q) -> impl Future<Output = Result<Arc<T>>> + Send
    where
        Q: Query<Response = T> + Send + Sync + 'static,
        T: Object,
    {
        async move {
            if self.0.dependents.contains(&query.id()) {
                bail!(
                    "Cyclic dependency during execution of {}\nTrace: {:?}",
                    query.id(),
                    self.0.trace_until(&query.id()).collect::<Vec<_>>()
                )
            }
            let id = query.id().clone();
            let (fingerprint, result) = self
                .0
                .continuity
                .clone()
                .do_execute(query, Arc::clone(&self.0))
                .await
                .with_context(|| format!("as a part of {id}"))?;
            self.0
                .direct_dependencies
                .send(PerMap::empty().insert(id.clone(), fingerprint))
                .await?;
            Ok(result)
        }
    }
}
