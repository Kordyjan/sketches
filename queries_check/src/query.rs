use anyhow::Result;
use async_std::sync::RwLock;
use divisors_fixed::Divisors;
use futures::future::{join_all, TryJoinAll};
use queries::{data::QueryId, execution::ExecutionContext, Param, Query};
use rand::{rng, Rng};
use std::ops::BitXor;
use std::sync::Arc;

pub static INPUT: Param<Vec<u64>> = Param::new("input");

#[derive(Clone, Copy)]
pub struct NonlockingProcess(pub usize);

impl Query for NonlockingProcess {
    type Response = u64;

    fn body(&self, ctx: &ExecutionContext) -> impl Future<Output = Result<Self::Response>> + Send {
        async {
            let this = ctx.get_param(&INPUT).await?[self.0];
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
                .fold(this, |acc, v| mix(acc, *v));
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
    type Response = u64;

    fn body(&self, ctx: &ExecutionContext) -> impl Future<Output = Result<Self::Response>> + Send {
        async move {
            let this = ctx.get_param(&INPUT).await?[self.0];
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
                .fold(this, |acc, v| mix(acc, *v));
            Ok(res)
        }
    }

    fn id(&self) -> QueryId {
        QueryId::new(format!("Process({})", self.0))
    }
}

pub fn mix(a: u64, b: u64) -> u64 {
    a.rotate_left(1).bitxor(b)
}
