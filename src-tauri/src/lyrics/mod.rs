//! Lyrics lookup for the current track: cache first, then LRCLIB. Results are emitted as
//! `lyrics://changed` and tagged with the track's raw title/artist so the overlay can ignore
//! answers that arrive after the user already skipped to another song.

mod cache;
mod lrclib;
mod normalize;
mod offsets;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::media::NowPlaying;
use crate::overlay;
use crate::settings::MAX_OFFSET_MS;
use cache::Cache;
use normalize::TrackQuery;
use offsets::OffsetStore;

pub const EVENT_LYRICS: &str = "lyrics://changed";

/// Skipping through a playlist should not fire a request per track.
const DEBOUNCE: Duration = Duration::from_millis(400);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Lyrics {
    Loading,
    Synced { lrc: String },
    Plain { text: String },
    Instrumental,
    NotFound,
    Error { message: String },
}

/// Which LRCLIB entry was used; shown in edit mode to help diagnose wrong matches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchInfo {
    pub source: String,
    pub id: i64,
    pub track_name: String,
    pub artist_name: String,
    pub duration_secs: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsPayload {
    /// Raw metadata of the track this answer belongs to.
    pub title: String,
    pub artist: String,
    #[serde(flatten)]
    pub lyrics: Lyrics,
    pub matched: Option<MatchInfo>,
    /// Per-track sync offset; positive shows lyrics earlier. Added to the global offset.
    pub offset_ms: i32,
}

struct Inner {
    generation: u64,
    current: Option<LyricsPayload>,
    /// Offset/cache key of the current track.
    current_key: Option<String>,
}

pub struct LyricsState {
    inner: Mutex<Inner>,
    client: Option<Arc<lrclib::Client>>,
    cache: Option<Arc<Cache>>,
    offsets: OffsetStore,
}

impl LyricsState {
    pub fn new<R: Runtime>(app: &AppHandle<R>) -> Self {
        let client = lrclib::Client::new()
            .map_err(|e| eprintln!("[lyrics] http client unavailable: {e}"))
            .ok()
            .map(Arc::new);
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| eprintln!("[lyrics] no app data dir, cache and offsets disabled: {e}"))
            .ok();
        Self {
            inner: Mutex::new(Inner {
                generation: 0,
                current: None,
                current_key: None,
            }),
            client,
            cache: data_dir.as_ref().map(|dir| Arc::new(Cache::new(dir.join("lyrics")))),
            offsets: OffsetStore::load(data_dir.map(|dir| dir.join("offsets.json"))),
        }
    }
}

#[tauri::command]
pub fn get_lyrics(state: State<'_, LyricsState>) -> Option<LyricsPayload> {
    state.inner.lock().unwrap().current.clone()
}

fn record_to_lyrics(r: &lrclib::Record) -> (Lyrics, Option<MatchInfo>) {
    let lyrics = if r.instrumental {
        Lyrics::Instrumental
    } else if r.has_synced() {
        Lyrics::Synced {
            lrc: r.synced_lyrics.clone().unwrap_or_default(),
        }
    } else {
        match r.plain_lyrics.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => Lyrics::Plain { text: text.to_string() },
            _ => Lyrics::NotFound,
        }
    };
    let matched = MatchInfo {
        source: "lrclib".into(),
        id: r.id,
        track_name: r.track_name.clone(),
        artist_name: r.artist_name.clone(),
        duration_secs: r.duration,
    };
    (lyrics, Some(matched))
}

async fn resolve(
    client: &lrclib::Client,
    cache: Option<&Cache>,
    q: &TrackQuery,
) -> (Lyrics, Option<MatchInfo>) {
    if let Some(hit) = cache.and_then(|c| c.get(q)) {
        return hit;
    }

    let exact = match client.get(q).await {
        Ok(record) => record.filter(|r| r.has_synced() || r.instrumental),
        Err(message) => return (Lyrics::Error { message }, None),
    };
    let result = match exact {
        Some(record) => record_to_lyrics(&record),
        // The exact match may lack synced lyrics while another version has them.
        None => match client.search(q).await {
            Ok(records) => lrclib::pick_best(&records, q)
                .map(record_to_lyrics)
                .unwrap_or((Lyrics::NotFound, None)),
            Err(message) => return (Lyrics::Error { message }, None),
        },
    };

    if let Some(cache) = cache {
        cache.put(q, &result.0, &result.1);
    }
    result
}

fn publish<R: Runtime>(app: &AppHandle<R>, generation: u64, payload: Option<LyricsPayload>) {
    let state = app.state::<LyricsState>();
    let mut inner = state.inner.lock().unwrap();
    if inner.generation != generation {
        return; // A newer track superseded this answer.
    }
    inner.current = payload.clone();
    drop(inner);
    let _ = app.emit(EVENT_LYRICS, payload);
}

/// Call when the track identity changes (not on play/pause/seek).
pub fn on_track_changed<R: Runtime>(app: &AppHandle<R>, track: Option<&NowPlaying>) {
    let state = app.state::<LyricsState>();
    let query = track.map(|t| normalize::build_query(&t.title, &t.artist, &t.album, t.duration_ms));
    let key = query.as_ref().map(cache::cache_key);
    let generation = {
        let mut inner = state.inner.lock().unwrap();
        inner.generation += 1;
        inner.current_key = key.clone();
        inner.generation
    };

    let (Some(track), Some(query), Some(key)) = (track, query, key) else {
        publish(app, generation, None);
        return;
    };

    let offset_ms = state.offsets.get(&key);
    let (title, artist) = (track.title.clone(), track.artist.clone());
    let payload = move |lyrics: Lyrics, matched: Option<MatchInfo>| LyricsPayload {
        title: title.clone(),
        artist: artist.clone(),
        lyrics,
        matched,
        offset_ms,
    };
    publish(app, generation, Some(payload(Lyrics::Loading, None)));

    let Some(client) = state.client.clone() else {
        let message = "HTTP client unavailable".to_string();
        publish(app, generation, Some(payload(Lyrics::Error { message }, None)));
        return;
    };
    let cache = state.cache.clone();
    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(DEBOUNCE).await;
        if app.state::<LyricsState>().inner.lock().unwrap().generation != generation {
            return;
        }
        let (lyrics, matched) = resolve(&client, cache.as_deref(), &query).await;
        if cfg!(debug_assertions) {
            let status = match &lyrics {
                Lyrics::Synced { .. } => "synced",
                Lyrics::Plain { .. } => "plain",
                Lyrics::Instrumental => "instrumental",
                Lyrics::NotFound => "not found",
                Lyrics::Error { .. } => "error",
                Lyrics::Loading => "loading",
            };
            eprintln!("[lyrics] {query:?} -> {status} {matched:?} offset={offset_ms}ms");
        }
        // Keep an offset the user adjusted while the lookup was running.
        let mut answer = payload(lyrics, matched);
        answer.offset_ms = app.state::<LyricsState>().offsets.get(&key);
        publish(&app, generation, Some(answer));
    });
}

/// Settings window counterpart of the offset hotkeys.
#[tauri::command]
pub fn adjust_track_offset<R: Runtime>(app: AppHandle<R>, delta_ms: Option<i32>) {
    adjust_offset(&app, delta_ms);
}

/// Nudge the current track's offset by `delta_ms`, or reset it with `None`.
pub fn adjust_offset<R: Runtime>(app: &AppHandle<R>, delta_ms: Option<i32>) {
    let state = app.state::<LyricsState>();
    let mut inner = state.inner.lock().unwrap();
    let Some(key) = inner.current_key.clone() else {
        drop(inner);
        overlay::toast(app, "No track playing");
        return;
    };

    let offset_ms = match delta_ms {
        Some(delta) => (state.offsets.get(&key) + delta).clamp(-MAX_OFFSET_MS, MAX_OFFSET_MS),
        None => 0,
    };
    state.offsets.set(&key, offset_ms);
    if let Some(current) = inner.current.as_mut() {
        current.offset_ms = offset_ms;
    }
    let payload = inner.current.clone();
    drop(inner);

    let _ = app.emit(EVENT_LYRICS, payload);
    overlay::toast(app, &offset_toast(offset_ms));
}

fn offset_toast(offset_ms: i32) -> String {
    if offset_ms == 0 {
        return "Lyrics offset reset for this track".into();
    }
    let secs = offset_ms.unsigned_abs() as f64 / 1000.0;
    let direction = if offset_ms > 0 { "earlier" } else { "later" };
    format!("Lyrics {secs:.2} s {direction} for this track")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_toast_wording() {
        assert_eq!(offset_toast(0), "Lyrics offset reset for this track");
        assert_eq!(offset_toast(250), "Lyrics 0.25 s earlier for this track");
        assert_eq!(offset_toast(-1500), "Lyrics 1.50 s later for this track");
    }
}
