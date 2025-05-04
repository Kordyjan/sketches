use crate::data::{ErasedResponse, QueryResponse};
use crate::execution::ExecutionContext;
use anyhow::Result;
use data::{Object, Param, QueryId};
use futures::future::BoxFuture;
use std::sync::Arc;
use std::{any::Any, future::Future};

mod data;
mod execution;
mod fingerprinting;
mod serialization;

#[cfg(test)]
mod tests;

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
