use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    NewChapter { desc: String },
    Comment { content: String },
    Push { key: String, entry: CacheEntry },
    Pull { key: String },
    Remove { key: String },
    Modify { key: String, entry: CacheEntry },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    value: String,
    fingerprint: String,
    world_state: HashMap<String, String>,
    deps_state: HashMap<String, String>,
    direct_world_stet: HashMap<String, String>,
}
