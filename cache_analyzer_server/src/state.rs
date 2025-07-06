use crate::model::Freshness::{Fresh, Stale};
use crate::model::{
    CacheEntryDetail, Chapter, ChapterDetail, DepsEntry, DepsMap, KeyedEntry, OpHead, Snapshot,
    WorldEntry, WorldMap,
};
use per_set::PerMap;
use rocket::serde::json::serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use tracer_types::{CacheEntry, Message};

pub(crate) struct ChaptersState {
    chapters: Vec<Vec<(Message, CacheState)>>,
}

impl ChaptersState {
    pub(crate) fn read(path: &'static str) -> Self {
        let file = File::open(path).expect("Failed to open trace file");
        let reader = BufReader::new(file);

        let mut chapters = Vec::new();
        let mut current_chapter_messages = Vec::new();
        let mut cache = CacheState(PerMap::empty());
        let mut input_fingerprint = String::new();

        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            if line.trim().is_empty() {
                continue;
            }

            let message: Message =
                serde_json::from_str(&line).expect("Failed to parse JSON message");

            match &message {
                Message::NewChapter { fingerprint, .. } => {
                    input_fingerprint.clone_from(fingerprint);
                    // Save previous chapter if it has messages
                    if !current_chapter_messages.is_empty() {
                        chapters.push(mem::take(&mut current_chapter_messages));
                    }
                    // Start new chapter with the NewChapter message
                    let mut stale_cache = PerMap::empty();
                    for arc in &cache.0 {
                        let mut stale_entry = arc.1.clone();
                        stale_entry
                            .world_state
                            .0
                            .iter_mut()
                            .for_each(|e| e.freshness = Stale);
                        stale_cache = stale_cache.insert(arc.0.clone(), stale_entry);
                    }
                    current_chapter_messages.push((message, CacheState(stale_cache)));
                }
                Message::End {} => {
                    // End of trace
                    break;
                }
                Message::Push { key, entry } | Message::Modify { key, entry } => {
                    cache = CacheState(
                        cache
                            .0
                            .insert(key.clone(), Self::translate(entry, &input_fingerprint)),
                    );
                }
                Message::Remove { key } => {
                    cache = CacheState(cache.0.remove(key));
                }
                _ => current_chapter_messages.push((message, cache.clone())),
            }
        }

        // Save the last chapter if it has messages
        if !current_chapter_messages.is_empty() {
            chapters.push(current_chapter_messages);
        }

        Self { chapters }
    }

    pub(crate) fn get_chapters(&self) -> Vec<Chapter> {
        self.chapters
            .iter()
            .enumerate()
            .filter_map(|(id, messages)| {
                Self::extract_chapter_from_messages(id, messages.iter().map(|(message, _)| message))
            })
            .collect()
    }

    pub(crate) fn get_chapter(&self, chapter_id: usize) -> Option<Chapter> {
        if chapter_id >= self.chapters.len() {
            return None;
        }

        Self::extract_chapter_from_messages(
            chapter_id,
            self.chapters[chapter_id].iter().map(|(message, _)| message),
        )
    }

    fn extract_chapter_from_messages<'a>(
        id: usize,
        mut messages: impl Iterator<Item = &'a Message>,
    ) -> Option<Chapter> {
        // Find the NewChapter message in this chapter
        messages.find_map(|msg| {
            if let Message::NewChapter { data, fingerprint } = msg {
                Some(Chapter {
                    id,
                    data: data.clone(),
                    fingerprint: fingerprint.clone(),
                })
            } else {
                None
            }
        })
    }

    fn translate(entry: &CacheEntry, input_fingerprint: &String) -> CacheEntryDetail {
        let world_state = entry
            .world_state
            .iter()
            .map(|(key, fingerprint)| WorldEntry {
                key: key.clone(),
                fingerprint: fingerprint.clone(),
                freshness: if input_fingerprint == fingerprint {
                    Fresh
                } else {
                    Stale
                },
            })
            .collect();
        let world_state = WorldMap(world_state);

        let deps_state = entry
            .deps_state
            .iter()
            .map(|(key, fingerprint)| DepsEntry {
                key: key.clone(),
                fingerprint: fingerprint.clone(),
            })
            .collect();
        let deps_state = DepsMap(deps_state);

        let direct_world_state = entry
            .direct_world_state
            .iter()
            .map(|(key, fingerprint)| WorldEntry {
                key: key.clone(),
                fingerprint: fingerprint.clone(),
                freshness: if input_fingerprint == fingerprint {
                    Fresh
                } else {
                    Stale
                },
            })
            .collect();
        let direct_world_state = WorldMap(direct_world_state);

        println!("!");

        CacheEntryDetail {
            value: entry.value.clone(),
            fingerprint: entry.fingerprint.clone(),
            world_state,
            direct_world_state,
            deps_state,
        }
    }

    pub(crate) fn get_ops(&self, chapter: usize) -> Vec<OpHead> {
        if chapter >= self.chapters.len() {
            return Vec::new();
        }

        self.chapters[chapter]
            .iter()
            .filter_map(|(message, _)| match message {
                Message::Pull { key } => Some((format!("Pull {key}"), false)),
                Message::Push { key, .. } => Some((format!("Push {key}"), false)),
                Message::Modify { key, .. } => Some((format!("Modify {key}"), false)),
                Message::Remove { key } => Some((format!("Remove {key}"), false)),
                Message::Comment { content } => Some((content.clone(), true)),
                _ => None, // Skip NewChapter, Comment, and End messages
            })
            .enumerate()
            .map(|(id, (desc, is_comment))| OpHead {
                id,
                desc,
                is_comment,
            })
            .collect()
    }

    pub(crate) fn get_chapter_detail(&self, chapter: usize) -> Option<ChapterDetail> {
        let head = self.get_chapter(chapter)?;
        let ops = self.get_ops(chapter);

        Some(ChapterDetail { head, ops })
    }

    pub(crate) fn get_snapshot(&self, chapter: usize, op: usize) -> Option<Snapshot> {
        // Check if chapter exists
        if chapter >= self.chapters.len() {
            return None;
        }

        // Check if operation exists within the chapter
        if op >= self.chapters[chapter].len() {
            return None;
        }

        // Get the cache state at the specified operation
        let (_, cache_state) = &self.chapters[chapter][op];

        // Convert CacheState to Snapshot
        let keyed_entries: Vec<KeyedEntry> = cache_state
            .0
            .iter()
            .map(|arc| KeyedEntry {
                key: arc.0.clone(),
                entry: arc.1.clone(),
            })
            .collect();

        Some(Snapshot(keyed_entries))
    }
}

#[derive(Clone)]
struct CacheState(PerMap<String, CacheEntryDetail>);
