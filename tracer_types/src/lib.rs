use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    NewChapter {
        data: String,
        fingerprint: String,
    },
    Push {
        key: String,
        entry: CacheEntry,
        stack: Vec<String>,
    },
    Pull {
        key: String,
        reason: String,
        stack: Vec<String>,
    },
    Remove {
        key: String,
        stack: Vec<String>,
    },
    Modify {
        key: String,
        entry: CacheEntry,
        stack: Vec<String>,
    },
    BodyExecuted {
        key: String,
        stack: Vec<String>,
    },
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
