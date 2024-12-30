use anyhow::Result;
use futures::future::try_join_all;
use proptest::collection::vec;
use proptest::prelude::*;

use crate::{
    data::Param, execution::Reactor, ExecutionContext, Executor, Query, QueryId
};

static INPUT: Param<Vec<u64>> = Param::new("input");

proptest! {
    #[test]
    fn queries_can_have_dependencies(values in vec(0u64..1024, 0..10)) {
        let sum: u64 = values.iter().sum();

        let mut ctx = Reactor::new().with_param(&INPUT, values);
        let result = ctx.execute(Sum);
        prop_assert_eq!(sum, *result.unwrap());
    }

    fn trace_is_written(values in vec(0u64..1024, 0..10)) {
        let len = values.len();
        let mut ctx = Reactor::new().with_param(&INPUT, values);
        let _ = ctx.execute(Sum);
        let trace = ctx.trace();
        assert_eq!(len + 2, trace.len());
        assert_eq!(trace[0], "Length");
        for i in 0..len {
            assert_eq!(trace[i + 1], format!("RefRead({i})"));
        }
        assert_eq!(trace[len + 1], "Sum");
    }

    fn queries_results_are_cached_within_single_run(values in vec(0u64..1024, 0..10)) {
        let sum: u64 = values.iter().sum();
        let len = values.len();

        let mut ctx = Reactor::new().with_param(&INPUT, values);
        let result_sum = ctx.execute(Sum);
        let result_len = ctx.execute(Length);
        prop_assert_eq!(sum, *result_sum.unwrap());
        prop_assert_eq!(len, *result_len.unwrap());

        let len_num = ctx.trace().iter().filter(|s| s == &"Length").count();
        assert_eq!(1, len_num);
    }
}

struct Length;

impl Query for Length {
    type Response = usize;

    async fn body(&self, ctx: &mut impl ExecutionContext) -> Result<usize> {
        let res = ctx.get_param(&INPUT)?.len();
        Ok(res)
    }

    fn id(&self) -> QueryId {
        QueryId::new_static("Length")
    }
}

struct RefRead(usize);

impl Query for RefRead {
    type Response = u64;

    async fn body(&self, ctx: &mut impl ExecutionContext) -> Result<u64> {
        let res = ctx.get_param(&INPUT)?[self.0];
        Ok(res)
    }

    fn id(&self) -> QueryId {
        QueryId::new(format!("RefRead({:?})", self.0))
    }
}

struct Sum;

impl Query for Sum {
    type Response = u64;

    async fn body(&self, ctx: &mut impl ExecutionContext) -> Result<u64> {
        let length = *ctx.run(Length).await?;
        let futures = (0..length)
            .map(|n| ctx.run(RefRead(n)))
            .collect::<Vec<_>>();
        let res = try_join_all(futures).await?
            .into_iter()
            .map(|n| *n)
            .sum();
        Ok(res)
    }

    fn id(&self) -> QueryId {
        QueryId::new_static("Sum")
    }
}

struct Double;

impl Query for Double {
    type Response = u64;

    async fn body(&self, ctx: &mut impl ExecutionContext) -> Result<u64> {
        Ok(*ctx.run(Sum).await? * 2)
    }

    fn id(&self) -> QueryId {
        QueryId::new_static("Double")
    }
}
