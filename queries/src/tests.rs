use crate::data::Param;
use crate::execution::ExecutionContext;
use crate::{Executor, Query, QueryId, execution::Reactor};
use anyhow::Result;
use async_global_executor::block_on;
use futures::future::try_join_all;
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

static INPUT: Param<Vec<u64>> = Param::new("input");

fn configuration() -> ProptestConfig {
    ProptestConfig {
        timeout: 60 * 1000,
        max_shrink_iters: 8 * 1024,
        ..ProptestConfig::default()
    }
}

#[ntest::timeout(3000)]
#[test]
fn queries_are_not_executed_when_no_changes_to_direct_dependencies_() {
    let mut values = vec![1, 2, 3];
    let (inc, dec) = (0, 1);
    let sum: u64 = values.iter().sum::<u64>() * 2;

    // sleep(Duration::from_millis(50));
    let ctx = Arc::new(Reactor::new());
    ctx.set_param(&INPUT, values.clone());
    let result_sum = block_on(ctx.execute(Double));
    assert_eq!(sum, *result_sum.unwrap());

    values[inc] += 1;
    values[dec] -= 1;
    ctx.set_param(&INPUT, values.clone());
    let result_sum = block_on(ctx.execute(Double));
    let res = *result_sum.unwrap();
    assert_eq!(sum, res);

    let doubling_num = block_on(ctx.trace())
        .iter()
        .filter(|s| s == &"[Double]")
        .count();
    assert_eq!(1, doubling_num);
}

#[test]
fn queries_are_executed() {
    let ctx = Arc::new(Reactor::new());
    ctx.set_param(&INPUT, vec![1, 2, 3]);
    let result = block_on(ctx.execute(Length));
    assert_eq!(3, *result.unwrap());
}

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn queries_can_have_dependencies(values in vec(0u64..1024, 0..20)) {
        let sum: u64 = values.iter().sum();

        let ctx = Arc::new(Reactor::new());
        ctx.set_param(&INPUT, values);
        let result = block_on( ctx.execute(Sum));
        prop_assert_eq!(sum, *result.unwrap());
    }

    #[test]
    fn trace_is_written(values in vec(0u64..1024, 0..10)) {
        let expected_middle: HashSet<String> = (0..values.len()).map(|n| format!("[RefRead({n})]")).collect();
        let len = values.len();
        let ctx = Arc::new(Reactor::new());
            ctx.set_param(&INPUT, values);
        let _ = block_on(ctx.execute(Sum));
        let trace = block_on(ctx.trace());

        assert_eq!(len + 2, trace.len());
        assert_eq!(trace[0], "[Length]");
        assert_eq!(trace[len + 1], "[Sum]");
        let middle: HashSet<String> = trace[1..=len].iter().map(ToOwned::to_owned).collect();
        assert_eq!(middle, expected_middle);
    }

    #[test]
    fn queries_results_are_cached(values in vec(0u64..1024, 0..10)) {
        let sum: u64 = values.iter().sum();
        let len = values.len();

        let ctx = Arc::new(Reactor::new());
        ctx.set_param(&INPUT, values);
        let result_sum = block_on(ctx.execute(Sum));
        let result_len = block_on(ctx.execute(Length));
        prop_assert_eq!(sum, *result_sum.unwrap());
        prop_assert_eq!(len, *result_len.unwrap());

        let len_num = block_on(ctx.trace()).iter().filter(|s| s == &"[Length]").count();
        prop_assert_eq!(1, len_num);
    }

    #[test]
    fn queries_are_not_executed_when_no_changes_to_direct_dependencies(
        (mut values, (inc, dec)) in list_with_picks()
    ) {
        let sum: u64 = values.iter().sum::<u64>() * 2;

        // sleep(Duration::from_millis(50));
        let ctx = Arc::new(Reactor::new());
        ctx.set_param(&INPUT, values.clone());
        let result_sum = block_on(ctx.execute(Double));
        prop_assert_eq!(sum, *result_sum.unwrap());

        values[inc] += 1;
        values[dec] -= 1;
        ctx.set_param(&INPUT, values.clone());
        let result_sum = block_on(ctx.execute(Double));
        let res = *result_sum.unwrap();
        assert_eq!(sum, res);

        let doubling_num = block_on(ctx.trace()).iter().filter(|s| s == &"[Double]").count();
        assert_eq!(1, doubling_num);
    }

    #[test]
    fn queries_are_executed_when_direct_dependencies_changed(
        (mut values, (inc, _)) in list_with_picks()
    ) {
        let res = values.iter().sum::<u64>() * 2;

        let ctx = Arc::new(Reactor::new());
        ctx.set_param(&INPUT, values.clone());
        let result_sum = block_on(ctx.execute(Double));
        prop_assert_eq!(res, *result_sum.unwrap());

        values[inc] += 1;
        ctx.set_param(&INPUT, values.clone());
        let result_sum = block_on(ctx.execute(Double));
        prop_assert_eq!(res + 2, *result_sum.unwrap());

        let doubling_num = block_on(ctx.trace()).iter().filter(|s| s == &"[Double]").count();
        prop_assert_eq!(2, doubling_num);
    }

}

prop_compose! {
    fn list_with_picks()(len in 2usize..10)
        (values in vec(1u64..1024, len), picks in (0usize..len, 0usize..len))
    -> (Vec<u64>, (usize, usize)) {
        (values, picks)
    }
}

#[derive(Clone)]
struct Length;

impl Query for Length {
    type Response = usize;

    async fn body(&self, ctx: &ExecutionContext) -> Result<usize> {
        let res = ctx.get_param(&INPUT).await?.len();
        Ok(res)
    }

    fn id(&self) -> QueryId {
        QueryId::new_static("Length")
    }
}

#[derive(Clone)]
struct RefRead(usize);

impl Query for RefRead {
    type Response = u64;

    async fn body(&self, ctx: &ExecutionContext) -> Result<u64> {
        let res = ctx.get_param(&INPUT).await?[self.0];
        Ok(res)
    }

    fn id(&self) -> QueryId {
        QueryId::new(format!("RefRead({:?})", self.0))
    }
}

#[derive(Clone)]
struct Sum;

impl Query for Sum {
    type Response = u64;

    async fn body(&self, ctx: &ExecutionContext) -> Result<u64> {
        let length = *ctx.run(Length).await?;
        let futures = (0..length).map(|n| ctx.run(RefRead(n))).collect::<Vec<_>>();
        let res = try_join_all(futures).await?.into_iter().map(|n| *n).sum();
        Ok(res)
    }

    fn id(&self) -> QueryId {
        QueryId::new_static("Sum")
    }
}

#[derive(Clone)]
struct Double;

impl Query for Double {
    type Response = u64;

    async fn body(&self, ctx: &ExecutionContext) -> Result<u64> {
        Ok(*ctx.run(Sum).await? * 2)
    }

    fn id(&self) -> QueryId {
        QueryId::new_static("Double")
    }
}
