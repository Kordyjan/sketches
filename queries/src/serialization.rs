use std::sync::Arc;

use crate::Object;
use anyhow::Result;

pub trait Writer {
    fn write(&mut self, data: &[u8]);
    fn write_object(&mut self, object: &dyn Object);
    fn write_arc(&mut self, arc: &Arc<dyn Object>);
}

pub trait Reader {
    fn read(&mut self, num: usize) -> &[u8];
    fn read_object<T: Object>(&mut self) -> Result<T>;
}
