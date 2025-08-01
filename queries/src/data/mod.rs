use crate::serialization::{Reader, Writer};
use anyhow::Result;
use std::fmt::Debug;
use std::{any::Any, borrow::Cow, fmt::Display, marker::PhantomData, sync::Arc};

pub mod instances;

pub trait QueryResponse: Send + Sync + 'static {
    type Boxed: Send + Sync + 'static;

    fn into_object(self) -> Arc<dyn Object>;

    fn downcast(object: Arc<dyn Object>) -> Result<Self::Boxed>;
}

pub struct ErasedResponse(pub Arc<dyn Object>);

impl QueryResponse for ErasedResponse {
    type Boxed = Arc<dyn Object>;
    fn into_object(self) -> Arc<dyn Object> {
        self.0
    }

    fn downcast(object: Arc<dyn Object>) -> Result<Self::Boxed> {
        Ok(object)
    }
}

impl<T: Object> QueryResponse for Arc<T> {
    type Boxed = Arc<T>;
    fn into_object(self) -> Arc<dyn Object> {
        self
    }

    fn downcast(object: Arc<dyn Object>) -> Result<Self::Boxed> {
        object
            .as_any()
            .downcast::<T>()
            .map_err(|_| anyhow::anyhow!("invalid type"))
    }
}

impl<T: Object> QueryResponse for T {
    type Boxed = Arc<T>;

    fn into_object(self) -> Arc<dyn Object> {
        Arc::new(self)
    }

    fn downcast(object: Arc<dyn Object>) -> Result<Self::Boxed> {
        object
            .as_any()
            .downcast::<T>()
            .map_err(|_| anyhow::anyhow!("invalid type"))
    }
}

pub trait Object: ObjectDowncast + Debug + Send + Sync + 'static {
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
    phantom: PhantomData<T>,
}

impl<T> Param<T> {
    pub const fn new(s: &'static str) -> Self {
        Self {
            id: QueryId::new_static(s),
            phantom: PhantomData,
        }
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

impl Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", &*self.0)
    }
}
