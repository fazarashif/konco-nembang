//! On-disk lyrics cache: one JSON file per track in `<app data>/lyrics/`.
//! Found lyrics never expire; "not found" expires so newly added lyrics get picked up.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::normalize::{match_key, TrackQuery};
use super::{Lyrics, MatchInfo};

const NOT_FOUND_TTL_MS: i64 = 3 * 24 * 60 * 60 * 1000;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    saved_at_ms: i64,
    lyrics: Lyrics,
    matched: Option<MatchInfo>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// FNV-1a: stable across Rust versions, unlike `DefaultHasher`, so file names stay valid.
fn fnv1a64(data: &str) -> u64 {
    data.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    })
}

pub fn cache_key(q: &TrackQuery) -> String {
    let id = format!(
        "{}\n{}\n{}",
        match_key(&q.artist),
        match_key(&q.title),
        q.duration_secs.unwrap_or(0)
    );
    format!("{:016x}", fnv1a64(&id))
}

pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, q: &TrackQuery) -> PathBuf {
        self.dir.join(format!("{}.json", cache_key(q)))
    }

    pub fn get(&self, q: &TrackQuery) -> Option<(Lyrics, Option<MatchInfo>)> {
        let data = fs::read_to_string(self.path(q)).ok()?;
        let entry: Entry = serde_json::from_str(&data).ok()?;
        if matches!(entry.lyrics, Lyrics::NotFound) && now_ms() - entry.saved_at_ms > NOT_FOUND_TTL_MS {
            return None;
        }
        Some((entry.lyrics, entry.matched))
    }

    /// Only final answers are stored; `Loading` and network `Error`s are not.
    pub fn put(&self, q: &TrackQuery, lyrics: &Lyrics, matched: &Option<MatchInfo>) {
        if matches!(lyrics, Lyrics::Loading | Lyrics::Error { .. }) {
            return;
        }
        let entry = Entry {
            saved_at_ms: now_ms(),
            lyrics: lyrics.clone(),
            matched: matched.clone(),
        };
        let Ok(json) = serde_json::to_string(&entry) else {
            return;
        };
        let path = self.path(q);
        let tmp = path.with_extension("json.tmp");
        // Write-then-rename so a crash never leaves a half-written file behind.
        let result = fs::create_dir_all(&self.dir)
            .and_then(|_| fs::write(&tmp, json))
            .and_then(|_| fs::rename(&tmp, &path));
        if let Err(err) = result {
            eprintln!("[lyrics] cache write failed: {err}");
        }
    }

    /// Forget a track so the next lookup goes back to LRCLIB.
    pub fn remove(&self, q: &TrackQuery) {
        let _ = fs::remove_file(self.path(q));
    }
}

#[cfg(test)]
mod tests {
    use super::super::normalize::build_query;
    use super::*;

    fn temp_cache(name: &str) -> Cache {
        let dir = std::env::temp_dir().join(format!("konco-nembang-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Cache::new(dir)
    }

    #[test]
    fn key_ignores_case_punctuation_and_album() {
        let a = build_query("The Cure", "Olivia Rodrigo", "Album A", 297_091);
        let b = build_query("the cure!", "olivia rodrigo", "", 297_400);
        assert_eq!(cache_key(&a), cache_key(&b));
        let c = build_query("the cure", "Olivia Rodrigo", "", 200_000);
        assert_ne!(cache_key(&a), cache_key(&c));
    }

    #[test]
    fn round_trips_synced_lyrics() {
        let cache = temp_cache("roundtrip");
        let q = build_query("Song", "Artist", "Album", 100_000);
        let lyrics = Lyrics::Synced { lrc: "[00:01.00]a".into() };
        cache.put(&q, &lyrics, &None);
        assert_eq!(cache.get(&q).map(|(l, _)| l), Some(lyrics));
        let _ = fs::remove_dir_all(&cache.dir);
    }

    #[test]
    fn remove_forgets_entry() {
        let cache = temp_cache("remove");
        let q = build_query("Song", "Artist", "Album", 100_000);
        cache.put(&q, &Lyrics::Instrumental, &None);
        assert!(cache.get(&q).is_some());
        cache.remove(&q);
        assert!(cache.get(&q).is_none());
        let _ = fs::remove_dir_all(&cache.dir);
    }

    #[test]
    fn does_not_store_errors() {
        let cache = temp_cache("errors");
        let q = build_query("Song", "Artist", "Album", 100_000);
        cache.put(&q, &Lyrics::Error { message: "offline".into() }, &None);
        assert!(cache.get(&q).is_none());
        let _ = fs::remove_dir_all(&cache.dir);
    }

    #[test]
    fn expired_not_found_is_ignored() {
        let cache = temp_cache("ttl");
        let q = build_query("Song", "Artist", "Album", 100_000);
        let entry = Entry {
            saved_at_ms: now_ms() - NOT_FOUND_TTL_MS - 1,
            lyrics: Lyrics::NotFound,
            matched: None,
        };
        fs::create_dir_all(&cache.dir).unwrap();
        fs::write(cache.path(&q), serde_json::to_string(&entry).unwrap()).unwrap();
        assert!(cache.get(&q).is_none());
        let _ = fs::remove_dir_all(&cache.dir);
    }
}
