use std::sync::Arc;

use anyhow::Result;
use data::{Object, QueryId, Param};

mod serialization;
mod data;
mod execution;

#[cfg(test)]
mod tests;

#[trait_variant::make(Send)]
pub trait Query {
    type Response;

    async fn body(&self, ctx: &mut impl ExecutionContext) -> Result<Self::Response>;
    fn id(&self) -> QueryId;
}

#[trait_variant::make(Send)]
trait ExecutionContext {
    fn get_param<T: Object>(&self, param: &Param<T>) -> Result<Arc<T>>;
    async fn run<T, Q>(& self, query: Q) -> Result<Arc<T>>
    where
        Q: Query<Response = T>;
}

trait Executor {
    fn with_param<T: Object>(self, param: &Param<T>, value: T) -> Self;
    fn execute<T, Q>(&mut self, query: Q) -> Result<Arc<T>>
    where
        Q: Query<Response = T>;
    fn trace(&self) -> &[String];
}
