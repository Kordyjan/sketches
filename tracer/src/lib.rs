use anyhow::Result;
use full::Tracer;
use futures::channel::mpsc;
use output::{ChapterMarker, Output};
use tracer_types::Message;

mod full;
mod output;

pub fn create_full(path: &'static str) -> Result<(Output, Tracer, ChapterMarker)> {
    let (sender, receiver) = mpsc::unbounded::<Message>();
    Ok((
        Output::new(path, receiver)?,
        Tracer(sender.clone()),
        ChapterMarker(sender),
    ))
}
