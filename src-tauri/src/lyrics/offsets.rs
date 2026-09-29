//! Per-track sync offsets, persisted to `<app data>/offsets.json`. Keyed like the lyrics
//! cache (artist + title + duration), so a music video and its album version keep separate
//! offsets.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct OffsetStore {
    path: Option<PathBuf>,
    map: Mutex<HashMap<String, i32>>,
}

impl OffsetStore {
    pub fn load(path: Option<PathBuf>) -> Self {
        let map = path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default();
        Self {
            path,
            map: Mutex::new(map),
        }
    }

    pub fn get(&self, key: &str) -> i32 {
        self.map.lock().unwrap().get(key).copied().unwrap_or(0)
    }

    /// Setting 0 removes the entry.
    pub fn set(&self, key: &str, offset_ms: i32) {
        let mut map = self.map.lock().unwrap();
        if offset_ms == 0 {
            map.remove(key);
        } else {
            map.insert(key.to_string(), offset_ms);
        }
        let Some(path) = &self.path else {
            return;
        };
        let result = serde_json::to_string(&*map)
            .map_err(|e| e.to_string())
            .and_then(|json| {
                if let Some(dir) = path.parent() {
                    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                }
                fs::write(path, json).map_err(|e| e.to_string())
            });
        if let Err(err) = result {
            eprintln!("[lyrics] could not save offsets: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persists_and_removes_zero() {
        let path = std::env::temp_dir().join(format!("konco-nembang-offsets-{}.json", std::process::id()));
        let _ = fs::remove_file(&path);

        let store = OffsetStore::load(Some(path.clone()));
        store.set("a", 750);
        store.set("b", -250);
        assert_eq!(OffsetStore::load(Some(path.clone())).get("a"), 750);

        store.set("a", 0);
        let reloaded = OffsetStore::load(Some(path.clone()));
        assert_eq!(reloaded.get("a"), 0);
        assert_eq!(reloaded.get("b"), -250);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn works_without_a_path() {
        let store = OffsetStore::load(None);
        store.set("a", 100);
        assert_eq!(store.get("a"), 100);
    }
}
