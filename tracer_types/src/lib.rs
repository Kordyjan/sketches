use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    NewChapter { data: String, fingerprint: String },
    Comment { content: String },
    Push { key: String, entry: CacheEntry },
    Pull { key: String },
    Remove { key: String },
    Modify { key: String, entry: CacheEntry },
    End {},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub value: String,
    pub fingerprint: String,
    pub world_state: HashMap<String, String>,
    pub deps_state: HashMap<String, String>,
    pub direct_world_state: HashMap<String, String>,
}
