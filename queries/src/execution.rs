use crate::data::{ErasedResponse, QueryResponse};
use crate::{
    data::{Object, Param, QueryId}, fingerprinting::{stamp_with_fingerprint, Fingerprint}, ErasedQuery,
    Executor,
    Query,
};
use anyhow::{anyhow, bail, Context, Result};
use dashmap::{DashMap, DashSet};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{
    channel::mpsc::{self}, lock::Mutex, stream::FuturesUnordered, FutureExt,
    StreamExt,
    TryStream,
    TryStreamExt,
};
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
    direct_world_state: CacheMap,
}

type QDashMap<V> = DashMap<QueryId, V, FxBuildHasher>;

pub struct Reactor {
    params: QDashMap<(Fingerprint, Arc<dyn Object>)>,
    trace: Mutex<(Vec<String>, UnboundedReceiver<String>)>,
    trace_sender: UnboundedSender<String>,
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
    #[must_use]
    pub fn new() -> Self {
        let (trace_sender, trace_receiver) = mpsc::unbounded();
        Reactor {
            params: QDashMap::default(),
            trace: Mutex::new((Vec::new(), trace_receiver)),
            trace_sender,
            cache: QDashMap::default(),
            current: QDashMap::default(),
            past_queries: QDashMap::default(),
        }
    }

    fn new_continuity(self: &Arc<Self>) -> Arc<Continuity> {
        Arc::new(Continuity::new(Arc::clone(self)))
    }

    fn do_execute<Q, T>(
        self: &Arc<Self>,
        query: Q,
        parent_context: Option<ExecutionContext>,
    ) -> impl Future<Output = Result<(Fingerprint, T::Boxed)>>
    where
        Q: Query<Response = T> + Send + Sync + 'static,
        T: QueryResponse,
    {
        DoExecute::new(self, query, parent_context)
    }

    fn start_processing<Q, T>(self: &Arc<Self>, query: Q, parent_context: Option<ExecutionContext>)
    where
        Q: Query<Response = T> + Send + Sync + 'static,
        T: QueryResponse,
    {
        let reactor = Arc::clone(self);
        let handle = async_global_executor::spawn(async move {
            let id = query.id();

            let cache_correct = match reactor.cache.get(&id) {
                Some(cached) if reactor.verify(&cached.world_state) => true,
                Some(cached) if !reactor.verify(&cached.direct_world_state) => false,
                Some(cached) => {
                    let deps_state = cached.deps_state.clone();
                    drop(cached);
                    Self::recheck(&reactor, &id, deps_state).await
                }
                None => false,
            };

            if !cache_correct {
                let cached = loop {
                    if let Some(res) = reactor
                        .run_body_once(query.clone(), parent_context.clone())
                        .await
                    {
                        break res;
                    }
                };

                if let Some(ExecutionContext(parent)) = parent_context {
                    let _ = parent
                        .world_dependencies
                        .unbounded_send(cached.world_state.clone());
                }

                reactor.cache.insert(query.id().clone(), cached);
                reactor
                    .trace_sender
                    .unbounded_send(id.to_string())
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

    async fn run_body_once<T: QueryResponse>(
        self: &Arc<Self>,
        query: impl Query<Response = T>,
        parent_context: Option<ExecutionContext>,
    ) -> Option<Cached> {
        self.cache.remove(&query.id());
        let (world_sender, mut world_receiver) = mpsc::unbounded();
        let (direct_world_sender, mut direct_world_receiver) = mpsc::unbounded();
        let (deps_sender, mut deps_receiver) = mpsc::unbounded();

        let dependents = parent_context
            .as_ref()
            .map(|it| it.0.dependents.clone())
            .unwrap_or_default()
            .insert(query.id().clone());
        let continuity = parent_context.as_ref().map_or_else(
            || self.new_continuity().clone(),
            |it| it.0.continuity.clone(),
        );
        let context = ExecutionContext(Arc::new(ExecutionView {
            current: query.id().clone(),
            parent: parent_context.map(|it| it.0),
            dependents,
            continuity,
            world_dependencies: world_sender,
            direct_dependencies: deps_sender,
            direct_world_dependencies: direct_world_sender,
        }));

        let result = query.body(&context).await;
        context.0.world_dependencies.close_channel();
        context.0.direct_dependencies.close_channel();
        context.0.direct_world_dependencies.close_channel();

        let mut world_dependencies = CacheMap::empty();
        let mut direct_world_dependencies = CacheMap::empty();
        let mut direct_dependencies = CacheMap::empty();

        let deps_unique = loop {
            let Some(dep) = world_receiver.next().await else {
                break true;
            };
            let Ok(sum) = world_dependencies.non_overriding_union(&dep) else {
                break false;
            };
            world_dependencies = sum;
        } && loop {
            let Some(dep) = deps_receiver.next().await else {
                break true;
            };
            let Ok(sum) = direct_dependencies.non_overriding_union(&dep) else {
                break false;
            };
            direct_dependencies = sum;
        } && loop {
            let Some(dep) = direct_world_receiver.next().await else {
                break true;
            };
            let Ok(sum) = direct_world_dependencies.non_overriding_union(&dep) else {
                break false;
            };
            direct_world_dependencies = sum;
        };

        deps_unique.then(|| Cached {
            result: result.map(|it| stamp_with_fingerprint(it.into_object())),
            world_state: world_dependencies,
            deps_state: direct_dependencies,
            direct_world_state: direct_world_dependencies,
        })
    }

    async fn recheck(reactor: &Arc<Reactor>, id: &QueryId, deps_state: CacheMap) -> bool {
        if deps_state.is_empty() {
            false
        } else {
            let (world_sender, mut world_receiver) = mpsc::unbounded::<CacheMap>();
            let (direct_world_sender, mut direct_world_receiver) = mpsc::unbounded::<CacheMap>();
            let iter = deps_state.iter().map(|state| async {
                let Some(query) = reactor.past_queries.get(&state.0) else {
                    return false;
                };
                let q = query.clone();
                let id = query.id();
                drop(query);
                let result = reactor.do_execute(q, None).await;

                let res = if let Ok((f, _)) = result {
                    f == state.1
                } else {
                    false
                };
                if res {
                    let world_ref = reactor.cache.get(&id).unwrap().world_state.clone();
                    let world = world_ref.clone();
                    let direct_world_ref =
                        reactor.cache.get(&id).unwrap().direct_world_state.clone();
                    let direct_world = direct_world_ref.clone();
                    drop(world_ref);
                    drop(direct_world_ref);
                    if world_sender.unbounded_send(world).is_err() {
                        return false;
                    };
                    if direct_world_sender.unbounded_send(direct_world).is_err() {
                        return false;
                    }
                }
                res
            });

            let stream = iter.collect::<FuturesUnordered<_>>();
            let results = stream.collect::<Vec<_>>().await;
            let res = results.iter().all(|&a| a);
            if res {
                drop(world_sender);
                drop(direct_world_sender);

                let mut world_state = CacheMap::empty();
                while let Some(dep) = world_receiver.next().await {
                    if let Ok(new_state) = world_state.non_overriding_union(&dep) {
                        world_state = new_state;
                    } else {
                        return false;
                    }
                }

                let mut direct_world_state = CacheMap::empty();
                while let Some(dep) = direct_world_receiver.next().await {
                    if let Ok(new_state) = direct_world_state.non_overriding_union(&dep) {
                        direct_world_state = new_state;
                    } else {
                        return false;
                    }
                }

                reactor.cache.entry(id.clone()).and_modify(move |c| {
                    c.world_state = world_state;
                    c.direct_world_state = direct_world_state;
                });
            }
            let val = reactor.cache.get(&id).unwrap();
            println!(
                "Recheck for {}: {} ({:?})",
                id,
                res,
                val.result.as_ref().unwrap()
            );
            res
        }
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
        async move {
            loop {
                let continuity = self.new_continuity();
                let res = Arc::clone(&continuity).drive(query.clone()).await?;
                let world_state = &self
                    .cache
                    .get(&query.id())
                    .context("Cache corrupted")?
                    .world_state;
                if self.verify(world_state) {
                    break Ok(res);
                }
            }
        }
    }

    async fn trace(&self) -> Vec<String> {
        let mut lock = self.trace.lock().await;
        while let Some(message) = lock.1.next().await {
            lock.0.push(message);
        }
        Vec::clone(&lock.0)
    }
}

struct DoExecute<Q> {
    reactor: Arc<Reactor>,
    query: Option<Q>,
    query_id: QueryId,
    parent_context: Option<ExecutionContext>,
}

impl<Q: Query> DoExecute<Q> {
    fn new(reactor: &Arc<Reactor>, query: Q, parent_context: Option<ExecutionContext>) -> Self {
        DoExecute {
            reactor: Arc::clone(reactor),
            query_id: query.id(),
            query: Some(query),
            parent_context,
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
        if let Some(query) = self.as_mut().query.take() {
            let mut entry = self.reactor.current.entry(query.id()).or_default();
            let len = entry.len();
            entry.push(cx.waker().clone());
            if len == 0 {
                self.reactor
                    .start_processing(query, self.parent_context.clone());
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
}

impl Continuity {
    pub fn new(reactor: Arc<Reactor>) -> Self {
        Continuity {
            reactor,
            fresh_queries: DashSet::default(),
        }
    }

    pub fn drive<Q, T>(self: Arc<Self>, query: Q) -> impl Future<Output = Result<T::Boxed>>
    where
        Q: Query<Response = T>,
        T: QueryResponse,
    {
        async move {
            let (_, res) = self.do_execute(query, None).await?;
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
                .do_execute(query, parent_view.into().map(ExecutionContext))
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
    world_dependencies: UnboundedSender<PerMap<QueryId, Fingerprint>>,
    direct_world_dependencies: UnboundedSender<PerMap<QueryId, Fingerprint>>,
    direct_dependencies: UnboundedSender<PerMap<QueryId, Fingerprint>>,
}

#[derive(Clone)]
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
                .unbounded_send(PerMap::empty().insert(param.query_id().clone(), *fingerprint))?;
            self.0
                .direct_world_dependencies
                .unbounded_send(PerMap::empty().insert(param.query_id().clone(), *fingerprint))?;
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
                .unbounded_send(PerMap::empty().insert(id.clone(), fingerprint))?;
            Ok(result)
        }
    }
}
