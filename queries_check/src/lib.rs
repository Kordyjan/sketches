use crate::query::{NonlockingProcess, INPUT};
use async_global_executor::spawn;
use futures::future::TryJoinAll;
use queries::execution::Reactor;
use queries::Executor;
use rand::{distr, Rng};
use std::iter;
use std::sync::Arc;

pub(crate) mod query;

async fn nonlocking_scenario(rng: &mut impl Rng, input_size: usize, process_count: usize) {
    let input: Vec<u64> = iter::repeat_with(|| rng.random::<u64>())
        .take(input_size)
        .collect();
    let reactor = Arc::new(Reactor::new());
    reactor.set_param(&INPUT, input);
    iter::repeat_with(|| rng.sample(distr::Uniform::new(0, input_size).unwrap()))
        .take(process_count)
        .map(|n| {
            let reactor = Arc::clone(&reactor);
            spawn(async move { reactor.execute(NonlockingProcess(n)).await })
        })
        .collect::<TryJoinAll<_>>()
        .await
        .unwrap();
}

struct Step;

enum Op {
    Replace(u64, u64),
    Calculate(u64),
    CalculateNonlocking(u64),
}

#[cfg(test)]
mod test;
