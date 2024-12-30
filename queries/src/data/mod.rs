use std::{any::Any, borrow::Cow, marker::PhantomData, sync::Arc};

use crate::serialization::{Reader, Writer};
use anyhow::Result;

pub mod instances;

pub trait Object: ObjectDowncast + Send + Sync + 'static {
    fn write(&self, writer: &mut dyn Writer);
}

pub trait ReadObject: Object {
    fn read(reader: &mut impl Reader) -> Result<Self>
    where
        Self: Sized;
}

pub trait ObjectDowncast {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<U: Object> ObjectDowncast for U {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self as Arc<dyn Any + Send + Sync>
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Param<T> {
    id: QueryId,
    phantom: PhantomData<T>
}
 
impl<T> Param<T> {
    pub const fn new(s: &'static str) -> Self {
        Self { id: QueryId::new_static(s), phantom: PhantomData }
    }

    pub(crate) fn query_id(&self) -> &QueryId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryId(Cow<'static, str>);

impl QueryId {
    pub const fn new_static(s: &'static str) -> Self {
        QueryId(Cow::Borrowed(s))
    }

    pub fn new(s: impl ToOwned<Owned = String>) -> Self {
        QueryId(Cow::Owned(s.to_owned()))
    }
}


