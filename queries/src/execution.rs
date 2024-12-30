use std::sync::Arc;

use crate::{
    data::{Object, Param, QueryId},
    Executor,
};
use anyhow::Result;
use rustc_hash::FxHashMap as HashMap;

pub struct Reactor {
    params: HashMap<QueryId, Arc<dyn Object>>,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            params: HashMap::default(),
        }
    }
}

impl Reactor {
    fn invalidate(&mut self) {}
}

impl Executor for Reactor {
    fn with_param<T: Object>(mut self, param: &Param<T>, value: T) -> Self {
        self.params.insert(param.query_id().clone(), Arc::new(value));
        self.invalidate();
        self
    }

    fn execute<T, Q>(&mut self, query: Q) -> Result<Arc<T>>
    where
        Q: crate::Query<Response = T>,
    {
        todo!()
    }

    fn trace(&self) -> &[String] {
        todo!()
    }
}
