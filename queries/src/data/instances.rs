use anyhow::{Context, Result, bail};

use crate::serialization::{Reader, Writer};

use super::{Object, ReadObject};

impl Object for u64 {
    fn write(&self, writer: &mut dyn Writer) {
        writer.write(&u64::to_be_bytes(*self));
    }
}

impl ReadObject for u64 {
    fn read(reader: &mut impl Reader) -> Result<Self>
    where
        Self: Sized,
    {
        let slice = reader.read(8);
        if slice.len() != 8 {
            bail!("missing bytes to read u64");
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(slice);
        Ok(u64::from_be_bytes(buf))
    }
}

impl Object for usize {
    fn write(&self, writer: &mut dyn Writer) {
        writer.write(&u64::to_be_bytes(*self as u64));
    }
}

impl ReadObject for usize {
    fn read(reader: &mut impl Reader) -> Result<Self> {
        let slice = reader.read(size_of::<usize>());
        if slice.len() != size_of::<u64>() {
            bail!("missing bytes to read usize");
        }
        let mut buf = [0u8; size_of::<usize>()];
        buf.copy_from_slice(slice);
        Ok(usize::from_be_bytes(buf))
    }
}

impl<T: Object> Object for Vec<T> {
    fn write(&self, writer: &mut dyn Writer) {
        writer.write_object(&(self.len() as u64));
        for obj in self {
            writer.write_object(obj);
        }
    }
}

impl<T: ReadObject> ReadObject for Vec<T> {
    fn read(reader: &mut impl Reader) -> Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_object::<u64>().context("Reading vec length")? as usize;
        let mut res = Vec::with_capacity(len);
        for _ in 0..len {
            res.push(reader.read_object().context("Reading vec contents")?);
        }
        Ok(res)
    }
}
