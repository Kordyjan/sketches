use serde::Serialize;

#[derive(Serialize, Clone)]
pub(crate) struct Chapter {
    pub(crate) id: usize,
    pub(crate) data: String,
    pub(crate) fingerprint: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct ChapterDetail {
    pub(crate) head: Chapter,
    pub(crate) ops: Vec<OpHead>,
}

#[derive(Serialize, Clone)]
pub(crate) struct OpHead {
    pub(crate) id: usize,
    pub(crate) desc: String,
    pub(crate) is_comment: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct Snapshot(pub(crate) Vec<KeyedEntry>);

#[derive(Serialize, Clone)]
pub(crate) struct KeyedEntry {
    pub(crate) key: String,
    pub(crate) entry: CacheEntryDetail,
}

#[derive(Serialize, Clone)]
pub(crate) struct CacheEntryDetail {
    pub(crate) value: String,
    pub(crate) fingerprint: String,
    pub(crate) world_state: WorldMap,
    pub(crate) direct_world_state: WorldMap,
    pub(crate) deps_state: DepsMap,
}

#[derive(Serialize, Clone)]
pub(crate) struct WorldMap(pub(crate) Vec<WorldEntry>);

#[derive(Serialize, Clone)]
pub(crate) struct WorldEntry {
    pub(crate) freshness: Freshness,
    pub(crate) key: String,
    pub(crate) fingerprint: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct DepsMap(pub(crate) Vec<DepsEntry>);

#[derive(Serialize, Clone)]
pub(crate) struct DepsEntry {
    pub(crate) key: String,
    pub(crate) fingerprint: String,
}

#[derive(Serialize, Clone)]
pub(crate) enum Freshness {
    Fresh,
    Stale,
}
