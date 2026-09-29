//! Client for LRCLIB (https://lrclib.net), a free, keyless database of synced lyrics.

use std::time::Duration;

use serde::Deserialize;

use super::normalize::{match_key, TrackQuery};

const BASE_URL: &str = "https://lrclib.net/api";
/// LRCLIB asks clients to identify themselves.
const USER_AGENT: &str = concat!(
    "KoncoNembang/",
    env!("CARGO_PKG_VERSION"),
    " (Windows lyrics overlay)"
);
const TIMEOUT: Duration = Duration::from_secs(15);

/// `/api/get` matches duration within about 2 s; search results get a wider window because
/// music videos often have a longer intro/outro than the album track.
const CLOSE_DURATION_SECS: f64 = 3.0;
const MAX_DURATION_DIFF_SECS: f64 = 20.0;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: i64,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    #[serde(default)]
    pub instrumental: bool,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

impl Record {
    pub fn has_synced(&self) -> bool {
        self.synced_lyrics.as_deref().is_some_and(|s| !s.trim().is_empty())
    }

    fn has_any(&self) -> bool {
        self.instrumental
            || self.has_synced()
            || self.plain_lyrics.as_deref().is_some_and(|s| !s.trim().is_empty())
    }
}

pub struct Client {
    http: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { http })
    }

    /// Exact lookup by title, artist, album and duration. `Ok(None)` when LRCLIB has no match.
    pub async fn get(&self, q: &TrackQuery) -> Result<Option<Record>, String> {
        let Some(duration) = q.duration_secs else {
            return Ok(None);
        };
        let duration = duration.to_string();
        let mut params = vec![
            ("track_name", q.title.as_str()),
            ("artist_name", q.artist.as_str()),
            ("duration", duration.as_str()),
        ];
        if let Some(album) = &q.album {
            params.push(("album_name", album.as_str()));
        }

        let resp = self
            .http
            .get(format!("{BASE_URL}/get"))
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let resp = resp.error_for_status().map_err(|e| e.to_string())?;
        resp.json::<Record>().await.map(Some).map_err(|e| e.to_string())
    }

    /// Looser search, first by title + artist, then free text.
    pub async fn search(&self, q: &TrackQuery) -> Result<Vec<Record>, String> {
        let by_fields = self
            .search_with(&[
                ("track_name", q.search_title.as_str()),
                ("artist_name", q.artist.as_str()),
            ])
            .await?;
        if !by_fields.is_empty() {
            return Ok(by_fields);
        }
        let text = format!("{} {}", q.artist, q.search_title);
        self.search_text(text.trim()).await
    }

    /// Free-text search, as typed by the user in the search window.
    pub async fn search_text(&self, text: &str) -> Result<Vec<Record>, String> {
        self.search_with(&[("q", text)]).await
    }

    async fn search_with(&self, params: &[(&str, &str)]) -> Result<Vec<Record>, String> {
        self.http
            .get(format!("{BASE_URL}/search"))
            .query(params)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.to_string())?
            .json::<Vec<Record>>()
            .await
            .map_err(|e| e.to_string())
    }
}

/// Choose the most plausible record: same song, close duration, synced lyrics preferred.
pub fn pick_best<'a>(records: &'a [Record], q: &TrackQuery) -> Option<&'a Record> {
    let title_key = match_key(&q.search_title);
    let artist_key = match_key(&q.artist);

    records
        .iter()
        .filter(|r| r.has_any())
        .filter_map(|r| {
            let rec_title = match_key(&r.track_name);
            let rec_artist = match_key(&r.artist_name);
            let title_match = !title_key.is_empty()
                && (rec_title == title_key || rec_title.contains(&title_key));
            let artist_match = !artist_key.is_empty()
                && (rec_artist.contains(&artist_key) || artist_key.contains(&rec_artist));
            // Free-text search can return other songs; demand at least one strong signal.
            if !title_match && !artist_match {
                return None;
            }

            let diff = match (q.duration_secs, r.duration) {
                (Some(want), Some(got)) => (want as f64 - got).abs(),
                _ => 0.0,
            };
            if diff > MAX_DURATION_DIFF_SECS {
                return None;
            }

            // Lower is better, compared left to right.
            let score = (
                diff > CLOSE_DURATION_SECS,
                !r.has_synced(),
                !title_match,
                !artist_match,
                (diff * 1000.0) as u64,
            );
            Some((score, r))
        })
        .min_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, r)| r)
}

#[cfg(test)]
mod tests {
    use super::super::normalize::build_query;
    use super::*;

    fn rec(id: i64, track: &str, artist: &str, dur: f64, synced: bool) -> Record {
        Record {
            id,
            track_name: track.into(),
            artist_name: artist.into(),
            album_name: None,
            duration: Some(dur),
            instrumental: false,
            plain_lyrics: Some("la la".into()),
            synced_lyrics: synced.then(|| "[00:01.00]la la".into()),
        }
    }

    #[test]
    fn prefers_close_duration_then_synced() {
        let q = build_query("the cure", "Olivia Rodrigo", "", 297_091);
        // Search results seen in Tahap 0: lyric video, live versions, a longer cut.
        let records = [
            rec(1, "the cure", "Olivia Rodrigo", 315.0, true),
            rec(2, "Olivia Rodrigo - the cure (Jimmy Kimmel Live! 2026)", "Olivia Rodrigo", 296.0, true),
            rec(3, "the cure", "Olivia Rodrigo", 297.0, false),
            rec(4, "the cure", "Olivia Rodrigo", 297.0, true),
        ];
        assert_eq!(pick_best(&records, &q).map(|r| r.id), Some(4));
    }

    #[test]
    fn rejects_far_durations() {
        let q = build_query("Song", "Artist", "Album", 200_000);
        let records = [rec(1, "Song", "Artist", 260.0, true)];
        assert!(pick_best(&records, &q).is_none());
    }

    #[test]
    fn accepts_video_with_longer_intro_when_nothing_closer() {
        let q = build_query("DENNY CAKNAN - NEGORO ANGIN (Official Music Video)", "DENNY CAKNAN", "", 415_061);
        let records = [rec(7, "Negoro Angin", "Denny Caknan", 400.0, true)];
        assert_eq!(pick_best(&records, &q).map(|r| r.id), Some(7));
    }

    #[test]
    fn rejects_unrelated_free_text_results() {
        let q = build_query("Kangen", "Dewa 19", "", 0);
        let records = [rec(1, "Something Else", "Other Band", 240.0, true)];
        assert!(pick_best(&records, &q).is_none());
    }

    #[test]
    fn skips_records_without_lyrics() {
        let q = build_query("Song", "Artist", "Album", 200_000);
        let mut empty = rec(1, "Song", "Artist", 200.0, false);
        empty.plain_lyrics = None;
        let records = [empty, rec(2, "Song", "Artist", 201.0, false)];
        assert_eq!(pick_best(&records, &q).map(|r| r.id), Some(2));
    }

    #[test]
    fn deserializes_lrclib_json() {
        let json = r#"{"id":35341260,"name":"the cure","trackName":"the cure","artistName":"Olivia Rodrigo",
            "albumName":"you seem pretty sad for a girl so in love","duration":297.0,"instrumental":false,
            "plainLyrics":"a\nb","syncedLyrics":"[00:13.99] a\n[00:20.38] b"}"#;
        let r: Record = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, 35341260);
        assert!(r.has_synced());
    }

    #[test]
    fn deserializes_instrumental_with_nulls() {
        let json = r#"{"id":1,"trackName":"Intro","artistName":"X","albumName":null,"duration":60.0,
            "instrumental":true,"plainLyrics":null,"syncedLyrics":null}"#;
        let r: Record = serde_json::from_str(json).unwrap();
        assert!(r.instrumental && !r.has_synced());
    }
}
