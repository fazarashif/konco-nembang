//! Now-playing detection. A background thread polls the platform media source and
//! emits `media://now-playing` to the webviews whenever the snapshot changes.

#[cfg(windows)]
mod gsmtc;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::settings::MediaSourceFilter;
use crate::tray;

pub const EVENT_NOW_PLAYING: &str = "media://now-playing";

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const RETRY_INTERVAL: Duration = Duration::from_secs(5);
/// YT Music drops its session for a moment between tracks; only report "nothing playing"
/// once it has been gone this long, so the overlay does not flicker.
const MISSING_GRACE: Duration = Duration::from_secs(2);

/// What is playing right now. Field names are camelCase on the wire (see `src/lib/ipc.ts`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    /// App id reported by the OS, e.g. `Spotify.exe` or `Chrome._crx_...`.
    pub source_app: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    /// Track position at `position_updated_at_ms`. The player only refreshes this every
    /// few seconds, so the frontend extrapolates from it.
    pub position_ms: u64,
    /// Unix epoch ms when `position_ms` was valid; 0 if the player did not say.
    pub position_updated_at_ms: i64,
    pub playing: bool,
    pub rate: f64,
}

impl NowPlaying {
    /// Same song, ignoring position and play state.
    pub fn same_track(&self, other: &NowPlaying) -> bool {
        self.source_app == other.source_app
            && self.title == other.title
            && self.artist == other.artist
            && self.album == other.album
            && self.duration_ms == other.duration_ms
    }
}

/// Minimal view of a media session used to decide which one to follow.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionCandidate {
    pub app: String,
    pub playing: bool,
}

/// Pick the session to follow: a playing session wins; among several, prefer the one we
/// already follow, then the OS "current" one. When nothing plays, stay on the one we followed
/// so pausing does not jump to an unrelated tab.
pub fn pick_session(
    candidates: &[SessionCandidate],
    current_app: Option<&str>,
    previous_app: Option<&str>,
) -> Option<usize> {
    let find = |app: Option<&str>, playing_only: bool| {
        let app = app?;
        candidates
            .iter()
            .position(|c| c.app == app && (!playing_only || c.playing))
    };

    if candidates.iter().any(|c| c.playing) {
        return find(previous_app, true)
            .or_else(|| find(current_app, true))
            .or_else(|| candidates.iter().position(|c| c.playing));
    }
    find(previous_app, false)
        .or_else(|| find(current_app, false))
        .or(if candidates.is_empty() { None } else { Some(0) })
}

/// Whether the user's source setting lets us follow the player with this app id.
pub fn source_allowed(filter: MediaSourceFilter, app_id: &str) -> bool {
    let is_spotify = app_id.to_ascii_lowercase().contains("spotify");
    match filter {
        MediaSourceFilter::Auto => true,
        MediaSourceFilter::Spotify => is_spotify,
        // Everything else reporting to the OS is a browser tab or installed web app.
        MediaSourceFilter::Browser => !is_spotify,
    }
}

/// Platform-specific source of now-playing snapshots.
pub trait MediaSource {
    /// `previous_app` is the session followed on the last call, for sticky selection.
    fn snapshot(
        &mut self,
        previous_app: Option<&str>,
        filter: MediaSourceFilter,
    ) -> Result<Option<NowPlaying>, String>;
}

#[derive(Default)]
pub struct MediaState {
    now_playing: Mutex<Option<NowPlaying>>,
    filter: AtomicU8,
}

fn filter_to_u8(f: MediaSourceFilter) -> u8 {
    match f {
        MediaSourceFilter::Auto => 0,
        MediaSourceFilter::Spotify => 1,
        MediaSourceFilter::Browser => 2,
    }
}

fn filter_from_u8(v: u8) -> MediaSourceFilter {
    match v {
        1 => MediaSourceFilter::Spotify,
        2 => MediaSourceFilter::Browser,
        _ => MediaSourceFilter::Auto,
    }
}

pub fn set_source_filter<R: Runtime>(app: &AppHandle<R>, filter: MediaSourceFilter) {
    app.state::<MediaState>()
        .filter
        .store(filter_to_u8(filter), Ordering::SeqCst);
}

#[tauri::command]
pub fn get_now_playing(state: State<'_, MediaState>) -> Option<NowPlaying> {
    state.now_playing.lock().unwrap().clone()
}

#[cfg(windows)]
fn create_source() -> Result<Box<dyn MediaSource>, String> {
    gsmtc::GsmtcSource::new().map(|s| Box::new(s) as Box<dyn MediaSource>)
}

#[cfg(not(windows))]
fn create_source() -> Result<Box<dyn MediaSource>, String> {
    Err("media detection is only implemented on Windows".into())
}

fn log_change(snapshot: &Option<NowPlaying>) {
    if !cfg!(debug_assertions) {
        return;
    }
    match snapshot {
        Some(np) => eprintln!(
            "[media] {} {} - {} ({}) pos={}ms updated_at={} dur={}ms",
            if np.playing { "PLAY " } else { "PAUSE" },
            np.title,
            np.artist,
            np.source_app,
            np.position_ms,
            np.position_updated_at_ms,
            np.duration_ms
        ),
        None => eprintln!("[media] no session"),
    }
}

fn publish<R: Runtime>(app: &AppHandle<R>, previous: &Option<NowPlaying>, snapshot: &Option<NowPlaying>) {
    *app.state::<MediaState>().now_playing.lock().unwrap() = snapshot.clone();
    log_change(snapshot);
    let _ = app.emit(EVENT_NOW_PLAYING, snapshot);

    let track_changed = match (previous, snapshot) {
        (Some(a), Some(b)) => !a.same_track(b),
        (None, None) => false,
        _ => true,
    };
    if track_changed {
        crate::lyrics::on_track_changed(app, snapshot.as_ref());
        let tip = tray::tooltip_for(
            snapshot.as_ref().map(|np| np.title.as_str()),
            snapshot.as_ref().map(|np| np.artist.as_str()),
        );
        tray::set_tooltip(app, &tip);
    }
}

pub fn spawn_watcher<R: Runtime>(app: AppHandle<R>) {
    thread::Builder::new()
        .name("media-watcher".into())
        .spawn(move || {
            #[cfg(windows)]
            gsmtc::init_thread();

            let mut last: Option<NowPlaying> = None;
            let mut missing_since: Option<Instant> = None;
            loop {
                let mut source = match create_source() {
                    Ok(source) => source,
                    Err(err) => {
                        eprintln!("[media] source unavailable: {err}");
                        thread::sleep(RETRY_INTERVAL);
                        continue;
                    }
                };

                loop {
                    let previous_app = last.as_ref().map(|np| np.source_app.clone());
                    let filter = filter_from_u8(app.state::<MediaState>().filter.load(Ordering::SeqCst));
                    match source.snapshot(previous_app.as_deref(), filter) {
                        Ok(snapshot) => {
                            let within_grace = if snapshot.is_none() && last.is_some() {
                                missing_since.get_or_insert_with(Instant::now).elapsed()
                                    < MISSING_GRACE
                            } else {
                                missing_since = None;
                                false
                            };
                            if !within_grace && snapshot != last {
                                publish(&app, &last, &snapshot);
                                last = snapshot;
                            }
                        }
                        Err(err) => {
                            // Recreate the source; the OS media service may have restarted.
                            eprintln!("[media] snapshot failed: {err}");
                            break;
                        }
                    }
                    thread::sleep(POLL_INTERVAL);
                }
                thread::sleep(RETRY_INTERVAL);
            }
        })
        .expect("failed to spawn media watcher thread");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(app: &str, playing: bool) -> SessionCandidate {
        SessionCandidate {
            app: app.into(),
            playing,
        }
    }

    #[test]
    fn source_filter() {
        let spotify = "Spotify.exe";
        let ytm = "Chrome._crx_cinhimbnkkghhklpknlkffjgod";
        assert!(source_allowed(MediaSourceFilter::Auto, spotify));
        assert!(source_allowed(MediaSourceFilter::Auto, ytm));
        assert!(source_allowed(MediaSourceFilter::Spotify, spotify));
        assert!(!source_allowed(MediaSourceFilter::Spotify, ytm));
        assert!(!source_allowed(MediaSourceFilter::Browser, spotify));
        assert!(source_allowed(MediaSourceFilter::Browser, ytm));
        for f in [MediaSourceFilter::Auto, MediaSourceFilter::Spotify, MediaSourceFilter::Browser] {
            assert_eq!(filter_from_u8(filter_to_u8(f)), f);
        }
    }

    #[test]
    fn no_sessions() {
        assert_eq!(pick_session(&[], Some("a"), Some("a")), None);
    }

    #[test]
    fn playing_beats_current_paused() {
        // Found in Tahap 0: Opera (paused) + YT Music in Chrome (playing).
        let sessions = [c("opera", false), c("chrome", true)];
        assert_eq!(pick_session(&sessions, Some("opera"), None), Some(1));
    }

    #[test]
    fn several_playing_prefers_previous_then_current() {
        let sessions = [c("a", true), c("b", true), c("c", true)];
        assert_eq!(pick_session(&sessions, Some("b"), Some("c")), Some(2));
        assert_eq!(pick_session(&sessions, Some("b"), Some("x")), Some(1));
        assert_eq!(pick_session(&sessions, None, None), Some(0));
    }

    #[test]
    fn previous_paused_loses_to_other_playing() {
        let sessions = [c("spotify", false), c("chrome", true)];
        assert_eq!(pick_session(&sessions, None, Some("spotify")), Some(1));
    }

    #[test]
    fn all_paused_sticks_to_previous() {
        let sessions = [c("opera", false), c("chrome", false)];
        assert_eq!(pick_session(&sessions, Some("opera"), Some("chrome")), Some(1));
    }

    #[test]
    fn all_paused_without_previous_uses_current_then_first() {
        let sessions = [c("a", false), c("b", false)];
        assert_eq!(pick_session(&sessions, Some("b"), None), Some(1));
        assert_eq!(pick_session(&sessions, Some("gone"), Some("gone")), Some(0));
    }
}
