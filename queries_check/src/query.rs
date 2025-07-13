use anyhow::Result;
use async_std::sync::RwLock;
use divisors_fixed::Divisors;
use futures::future::{TryJoinAll, join_all};
use queries::QueryId;
use queries::{Query, data::Param, execution::ExecutionContext};
use rand::{Rng, rng};
use std::future::Future;
use std::sync::Arc;

pub static INPUT: Param<Vec<String>> = Param::new("input");

#[derive(Clone, Copy)]
pub struct NonlockingProcess(pub usize);

impl Query for NonlockingProcess {
    type Response = String;

    fn body(&self, ctx: &ExecutionContext) -> impl Future<Output = Result<Self::Response>> + Send {
        async {
            let this = ctx.get_param(&INPUT).await?[self.0].clone();
            let futures = self
                .0
                .divisors()
                .into_iter()
                .filter(|n| *n != self.0)
                .map(|n| ctx.run(NonlockingProcess(n)));
            let result = join_all(futures)
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .fold(this, |acc, v| mix(acc, v.as_ref()));
            Ok(result)
        }
    }

    fn id(&self) -> QueryId {
        QueryId::new(format!("NonlockingProcess({})", self.0))
    }
}

#[derive(Clone)]
pub struct Process(pub usize, pub Arc<RwLock<()>>);

impl Query for Process {
    type Response = String;

    fn body(&self, ctx: &ExecutionContext) -> impl Future<Output = Result<Self::Response>> + Send {
        async move {
            let this = ctx.get_param(&INPUT).await?[self.0].clone();
            let divisors = (self.0 + 1).divisors();
            let len = divisors.len();
            if len == 1 {
                return Ok(this);
            }
            let wait_before: usize = rng().random_range(..(len - 1));
            let res = divisors[..(len - 1)]
                .iter()
                .enumerate()
                .map(|(i, n)| async move {
                    if wait_before == i {
                        let guard = self.1.read().await;
                        let res = ctx.run(Process(*n - 1, self.1.clone())).await;
                        drop(guard);
                        res
                    } else {
                        ctx.run(Process(*n - 1, self.1.clone())).await
                    }
                })
                .collect::<TryJoinAll<_>>()
                .await?
                .into_iter()
                .fold(this, |acc, v| mix(acc, v.as_ref()));
            Ok(res)
        }
    }

    fn id(&self) -> QueryId {
        QueryId::new(format!("Process({})", self.0))
    }
}

pub fn mix(a: String, b: &str) -> String {
    a + b
}
