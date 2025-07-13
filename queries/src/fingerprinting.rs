use std::{hash::Hasher, sync::Arc};

use rustc_stable_hash::{FromStableHash, SipHasher128Hash, StableSipHasher128};

use crate::{data::Object, serialization::Writer};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Fingerprint([u64; 2]);

impl std::fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}~{:04x}", self.0[0] >> 48, self.0[1] & 0xffff)
    }
}

impl FromStableHash for Fingerprint {
    type Hash = SipHasher128Hash;

    fn from(hash: Self::Hash) -> Self {
        Fingerprint(hash.0)
    }
}

pub fn stamp_with_fingerprint(obj: Arc<dyn Object>) -> (Fingerprint, Arc<dyn Object>) {
    let mut fingerprinter = StableSipHasher128::new();
    fingerprinter.write_object(obj.as_ref());
    (fingerprinter.finish(), obj)
}

impl Writer for StableSipHasher128 {
    fn write(&mut self, data: &[u8]) {
        Hasher::write(self, data);
    }

    fn write_object(&mut self, object: &dyn Object) {
        object.write(self);
    }

    fn write_arc(&mut self, _arc: &Arc<dyn Object>) {
        todo!()
    }
}
