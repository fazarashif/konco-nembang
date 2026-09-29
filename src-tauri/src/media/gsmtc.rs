//! Windows implementation via Global System Media Transport Controls (GSMTC): the same API
//! behind the media flyout. Spotify and Chromium/Firefox (Media Session API) all report here.

use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

use super::{pick_session, MediaSource, NowPlaying, SessionCandidate};

/// 100 ns ticks between 1601-01-01 (Windows epoch) and 1970-01-01 (Unix epoch).
const UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;
const TICKS_PER_MS: i64 = 10_000;

/// Join the multithreaded apartment so WinRT calls work on the watcher thread.
pub fn init_thread() {
    // Fails harmlessly if the thread is already initialised.
    let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
}

fn ticks_to_ms(ticks: i64) -> u64 {
    (ticks.max(0) / TICKS_PER_MS) as u64
}

/// Windows `DateTime` to Unix epoch ms; 0 means the player never set it.
fn datetime_to_unix_ms(ticks: i64) -> i64 {
    if ticks <= 0 {
        0
    } else {
        (ticks - UNIX_EPOCH_TICKS) / TICKS_PER_MS
    }
}

fn err(e: windows::core::Error) -> String {
    e.message().to_string()
}

pub struct GsmtcSource {
    manager: Manager,
}

impl GsmtcSource {
    pub fn new() -> Result<Self, String> {
        let manager = Manager::RequestAsync().and_then(|op| op.join()).map_err(err)?;
        Ok(Self { manager })
    }

    fn read(session: &Session) -> windows::core::Result<NowPlaying> {
        let props = session.TryGetMediaPropertiesAsync()?.join()?;
        let timeline = session.GetTimelineProperties()?;
        let playback = session.GetPlaybackInfo()?;

        let start = timeline.StartTime()?.Duration;
        let end = timeline.EndTime()?.Duration;
        let rate = playback
            .PlaybackRate()
            .and_then(|r| r.Value())
            .unwrap_or(1.0);

        Ok(NowPlaying {
            source_app: session.SourceAppUserModelId()?.to_string(),
            title: props.Title()?.to_string(),
            artist: props.Artist()?.to_string(),
            album: props.AlbumTitle()?.to_string(),
            duration_ms: ticks_to_ms(end - start),
            position_ms: ticks_to_ms(timeline.Position()?.Duration - start),
            position_updated_at_ms: datetime_to_unix_ms(timeline.LastUpdatedTime()?.UniversalTime),
            playing: playback.PlaybackStatus()? == PlaybackStatus::Playing,
            rate,
        })
    }
}

impl MediaSource for GsmtcSource {
    fn snapshot(&mut self, previous_app: Option<&str>) -> Result<Option<NowPlaying>, String> {
        let sessions = self.manager.GetSessions().map_err(err)?;
        let count = sessions.Size().map_err(err)?;

        let mut list = Vec::with_capacity(count as usize);
        let mut candidates = Vec::with_capacity(count as usize);
        for i in 0..count {
            let session = sessions.GetAt(i).map_err(err)?;
            // A session can vanish between listing and reading; just skip it.
            let Ok(app) = session.SourceAppUserModelId() else {
                continue;
            };
            let playing = session
                .GetPlaybackInfo()
                .and_then(|p| p.PlaybackStatus())
                .map(|s| s == PlaybackStatus::Playing)
                .unwrap_or(false);
            candidates.push(SessionCandidate {
                app: app.to_string(),
                playing,
            });
            list.push(session);
        }

        // No current session is a normal state, not an error.
        let current_app = self
            .manager
            .GetCurrentSession()
            .and_then(|s| s.SourceAppUserModelId())
            .map(|s| s.to_string())
            .ok();

        let Some(index) = pick_session(&candidates, current_app.as_deref(), previous_app) else {
            return Ok(None);
        };
        match Self::read(&list[index]) {
            Ok(np) => Ok(Some(np)),
            // The chosen session closed mid-read; report nothing this tick.
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_windows_datetime_to_unix_ms() {
        // 2026-09-29T00:00:00Z = 1790640000000 ms since the Unix epoch.
        let ticks = UNIX_EPOCH_TICKS + 1_790_640_000_000 * TICKS_PER_MS;
        assert_eq!(datetime_to_unix_ms(ticks), 1_790_640_000_000);
    }

    #[test]
    fn unset_datetime_is_zero() {
        assert_eq!(datetime_to_unix_ms(0), 0);
    }

    #[test]
    fn negative_timespan_clamps_to_zero() {
        assert_eq!(ticks_to_ms(-5), 0);
        assert_eq!(ticks_to_ms(297_091 * TICKS_PER_MS), 297_091);
    }
}
