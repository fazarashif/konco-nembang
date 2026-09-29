//! Lyrics lookup for the current track: cache first, then LRCLIB. Results are emitted as
//! `lyrics://changed` and tagged with the track's raw title/artist so the overlay can ignore
//! answers that arrive after the user already skipped to another song.

mod cache;
mod lrclib;
mod normalize;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::media::NowPlaying;
use cache::Cache;
use normalize::TrackQuery;

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
}

struct Inner {
    generation: u64,
    current: Option<LyricsPayload>,
}

pub struct LyricsState {
    inner: Mutex<Inner>,
    client: Option<Arc<lrclib::Client>>,
    cache: Option<Arc<Cache>>,
}

impl LyricsState {
    pub fn new<R: Runtime>(app: &AppHandle<R>) -> Self {
        let client = lrclib::Client::new()
            .map_err(|e| eprintln!("[lyrics] http client unavailable: {e}"))
            .ok()
            .map(Arc::new);
        let cache = app
            .path()
            .app_data_dir()
            .map(|dir| Arc::new(Cache::new(dir.join("lyrics"))))
            .map_err(|e| eprintln!("[lyrics] no app data dir, cache disabled: {e}"))
            .ok();
        Self {
            inner: Mutex::new(Inner {
                generation: 0,
                current: None,
            }),
            client,
            cache,
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
    let generation = {
        let mut inner = state.inner.lock().unwrap();
        inner.generation += 1;
        inner.generation
    };

    let Some(track) = track else {
        publish(app, generation, None);
        return;
    };

    let payload = |lyrics: Lyrics, matched: Option<MatchInfo>| LyricsPayload {
        title: track.title.clone(),
        artist: track.artist.clone(),
        lyrics,
        matched,
    };
    publish(app, generation, Some(payload(Lyrics::Loading, None)));

    let Some(client) = state.client.clone() else {
        let message = "HTTP client unavailable".to_string();
        publish(app, generation, Some(payload(Lyrics::Error { message }, None)));
        return;
    };
    let cache = state.cache.clone();
    let query = normalize::build_query(&track.title, &track.artist, &track.album, track.duration_ms);
    let (title, artist) = (track.title.clone(), track.artist.clone());
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
            eprintln!("[lyrics] {query:?} -> {status} {matched:?}");
        }
        publish(
            &app,
            generation,
            Some(LyricsPayload {
                title,
                artist,
                lyrics,
                matched,
            }),
        );
    });
}
