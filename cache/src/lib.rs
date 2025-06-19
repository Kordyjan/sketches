use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::borrow::Cow;
use std::fmt::Display;

pub mod cache;
pub mod data;
pub mod fingerprinting;
pub mod serialization;

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

pub type QDashMap<V> = DashMap<QueryId, V, FxBuildHasher>;
