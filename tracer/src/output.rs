use anyhow::Result;
use futures::{StreamExt, channel::mpsc::UnboundedReceiver};
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use tracer_types::Message;

pub struct Output {
    writer: BufWriter<File>,
    receiver: UnboundedReceiver<Message>,
}

impl Output {
    pub(crate) fn new(path: &'static str, receiver: UnboundedReceiver<Message>) -> Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Output { writer, receiver })
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(m) = self.receiver.next().await {
            let serialized = serde_json::to_string(&m)?;
            self.writer.write_all(serialized.as_bytes())?;
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}
