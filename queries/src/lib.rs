use crate::execution::ExecutionContext;
use anyhow::Result;
use dashmap::DashMap;
use data::{ErasedResponse, Object, Param, QueryResponse};
use futures::future::BoxFuture;
use rustc_hash::FxBuildHasher;
use std::{any::Any, borrow::Cow, fmt::Display, future::Future, sync::Arc};

pub mod cache;
pub mod data;
pub mod execution;
pub mod fingerprinting;
pub mod serialization;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryId(Cow<'static, str>);

impl QueryId {
    #[must_use]
    pub const fn new_static(s: &'static str) -> Self {
        QueryId(Cow::Borrowed(s))
    }

    pub fn new(s: impl ToOwned<Owned = String>) -> Self {
        QueryId(Cow::Owned(s.to_owned()))
    }
}

impl Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", &*self.0)
    }
}

pub type QDashMap<V> = DashMap<QueryId, V, FxBuildHasher>;
#[trait_variant::make(Send)]
pub trait Query: Any + Send + Sync + Clone + 'static {
    type Response: QueryResponse;

    async fn body(&self, ctx: &ExecutionContext) -> Result<Self::Response>;
    fn id(&self) -> QueryId;
}

type QueryFn =
    dyn Fn(&ExecutionContext) -> BoxFuture<Result<ErasedResponse>> + Send + Sync + 'static;

#[derive(Clone)]
pub(crate) struct ErasedQuery(QueryId, Arc<QueryFn>);

impl Query for ErasedQuery {
    type Response = ErasedResponse;

    fn body(&self, ctx: &ExecutionContext) -> impl Future<Output = Result<Self::Response>> + Send {
        async { self.1(ctx).await }
    }

    fn id(&self) -> QueryId {
        self.0.clone()
    }
}

pub trait Executor {
    fn set_param<T: Object>(&self, param: &Param<T>, value: T);
    fn execute<T, Q>(&self, query: Q) -> impl Future<Output = Result<T::Boxed>> + Send
    where
        Q: Query<Response = T>,
        T: QueryResponse;
    fn trace(&self) -> impl Future<Output = Vec<String>>;
}
