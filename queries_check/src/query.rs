use anyhow::Result;
use async_std::sync::RwLock;
use divisors_fixed::Divisors;
use futures::future::join_all;
use queries::{data::QueryId, execution::ExecutionContext, Param, Query};
use std::ops::BitXor;

pub static INPUT: Param<Vec<u64>> = Param::new("input");
pub static LOCK: Param<RwLock<()>> = Param::new("lock");

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
        QueryId::new(format!("Process({})", self.0))
    }
}

fn mix(a: u64, b: u64) -> u64 {
    a.rotate_left(1).bitxor(b)
}
