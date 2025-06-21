use anyhow::Result;
use cache::TracingCache;
use futures::channel::mpsc;
use futures::channel::mpsc::UnboundedSender;
use output::Output;
use tracer_types::Message;

mod cache;
mod output;

pub fn create_cache(
    path: &'static str,
) -> Result<(Output, TracingCache, UnboundedSender<Message>)> {
    let (sender, receiver) = mpsc::unbounded::<Message>();
    Ok((
        Output::new(path, receiver)?,
        TracingCache::new(sender.clone()),
        sender,
    ))
}
