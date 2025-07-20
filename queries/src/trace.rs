use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    lock::Mutex,
};

use crate::{
    execution::{Cached, ExecutionContext},
    QueryId,
};

pub trait Trace {
    fn cache_push(&self, key: &QueryId, entry: &Cached, context: Option<&ExecutionContext>);
    fn cache_pull(&self, key: &QueryId, reason: &'static str, context: Option<&ExecutionContext>);
    fn cache_remove(&self, key: &QueryId, context: Option<&ExecutionContext>);
    fn cache_modify(&self, key: &QueryId, entry: &Cached, context: Option<&ExecutionContext>);
    fn body_run(&self, key: &QueryId, context: Option<&ExecutionContext>);
}

pub struct NoOpTrace;

impl Trace for NoOpTrace {
    fn cache_push(&self, _key: &QueryId, _entry: &Cached, _context: Option<&ExecutionContext>) {}
    fn cache_pull(
        &self,
        _key: &QueryId,
        _reason: &'static str,
        _context: Option<&ExecutionContext>,
    ) {
    }
    fn cache_remove(&self, _key: &QueryId, _context: Option<&ExecutionContext>) {}
    fn cache_modify(&self, _key: &QueryId, _entry: &Cached, _context: Option<&ExecutionContext>) {}
    fn body_run(&self, _key: &QueryId, _context: Option<&ExecutionContext>) {}
}

pub struct BodyExecutionTrace(UnboundedSender<String>);

pub struct BodyExecutionTraceReader(Mutex<(Vec<String>, UnboundedReceiver<String>)>);

#[must_use]
pub fn body_execution() -> (BodyExecutionTrace, BodyExecutionTraceReader) {
    let (sender, receiver) = mpsc::unbounded();
    (
        BodyExecutionTrace(sender),
        BodyExecutionTraceReader(Mutex::new((Vec::new(), receiver))),
    )
}

impl BodyExecutionTraceReader {
    #[must_use]
    pub async fn get_trace(&self) -> Vec<String> {
        let (ref mut vec, ref mut receiver) = *self.0.lock().await;
        while let Ok(Some(name)) = receiver.try_next() {
            vec.push(name);
        }
        vec.clone()
    }
}

impl Trace for BodyExecutionTrace {
    fn cache_push(&self, _key: &QueryId, _entry: &Cached, _context: Option<&ExecutionContext>) {}
    fn cache_pull(
        &self,
        _key: &QueryId,
        _reason: &'static str,
        _context: Option<&ExecutionContext>,
    ) {
    }
    fn cache_remove(&self, _key: &QueryId, _context: Option<&ExecutionContext>) {}
    fn cache_modify(&self, _key: &QueryId, _entry: &Cached, _context: Option<&ExecutionContext>) {}

    fn body_run(&self, key: &QueryId, _context: Option<&ExecutionContext>) {
        self.0
            .unbounded_send(format!("{key}"))
            .expect("Trace channel is broken");
    }
}
