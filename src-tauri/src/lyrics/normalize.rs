//! Turn raw media metadata into a lyrics query. Spotify and YT Music report clean metadata;
//! plain YouTube videos report `Artist - Title (Official Video)` with the channel as artist.

/// Words that mark a bracketed group or `| ...` suffix as video noise, not part of the title.
const NOISE_WORDS: &[&str] = &[
    "official", "video", "audio", "lyric", "lyrics", "lirik", "mv", "m/v", "visualizer",
    "visualiser", "hd", "4k", "hq", "clip", "explicit", "videoclip",
];

/// Channel-name decorations that are not part of the artist name.
const CHANNEL_SUFFIXES: &[&str] = &[" - Topic", "VEVO", " Official", " official", " OFFICIAL"];

const DASHES: &[&str] = &[" - ", " – ", " — ", " -- "];

#[derive(Clone, Debug, PartialEq)]
pub struct TrackQuery {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_secs: Option<u32>,
    /// Title without `(feat. ...)`, for the looser search fallback.
    pub search_title: String,
}

/// Lowercase alphanumerics only, for fuzzy comparisons.
pub fn match_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_noise(group: &str) -> bool {
    let lower = group.to_lowercase();
    lower
        .split(|c: char| !(c.is_alphanumeric() || c == '/'))
        .any(|token| NOISE_WORDS.contains(&token))
}

fn collapse_spaces(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn trim_separators(s: &str) -> String {
    s.trim_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '–' | '—' | '|' | ':'))
        .to_string()
}

/// Remove `(Official Video)`, `[MV]`, `【Lyrics】`, `| Official Audio`, `#hashtags`.
/// Groups that are not noise, such as `(Live Acoustic)` or `(feat. X)`, are kept.
pub fn strip_noise(title: &str) -> String {
    let pairs = [('(', ')'), ('[', ']'), ('【', '】'), ('{', '}')];
    let mut out = String::with_capacity(title.len());
    let mut rest = title;

    while let Some((start, open)) = rest.char_indices().find(|(_, c)| pairs.iter().any(|p| p.0 == *c)) {
        let close = pairs.iter().find(|p| p.0 == open).unwrap().1;
        let after_open = start + open.len_utf8();
        let Some(len) = rest[after_open..].find(close) else {
            break;
        };
        let inner = &rest[after_open..after_open + len];
        let end = after_open + len + close.len_utf8();
        out.push_str(&rest[..start]);
        if !is_noise(inner) {
            out.push_str(&rest[start..end]);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);

    // `Title | Official Video` style suffixes.
    let mut parts: Vec<&str> = out.split(" | ").collect();
    while parts.len() > 1 && is_noise(parts[parts.len() - 1]) {
        parts.pop();
    }
    let joined = parts.join(" | ");

    let without_tags: Vec<&str> = joined.split_whitespace().filter(|w| !w.starts_with('#')).collect();
    trim_separators(&collapse_spaces(&without_tags.join(" ")))
}

/// `Title (feat. X)` / `Title ft. X` -> `Title`.
pub fn strip_featuring(title: &str) -> String {
    let lower = title.to_lowercase();
    let cut = ["(feat.", "[feat.", "(ft.", "[ft.", " feat. ", " ft. ", "(featuring", " featuring "]
        .iter()
        .filter_map(|m| lower.find(m))
        .min();
    match cut {
        Some(i) => trim_separators(&title[..i]),
        None => title.to_string(),
    }
}

pub fn clean_channel(artist: &str) -> String {
    let mut s = artist.trim().to_string();
    for suffix in CHANNEL_SUFFIXES {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped.to_string();
        }
    }
    trim_separators(&s)
}

fn split_dash(s: &str) -> Option<(String, String)> {
    DASHES.iter().filter_map(|d| s.find(d).map(|i| (i, d.len()))).min().and_then(|(i, len)| {
        let left = s[..i].trim();
        let right = s[i + len..].trim();
        (!left.is_empty() && !right.is_empty()).then(|| (left.to_string(), right.to_string()))
    })
}

pub fn build_query(title: &str, artist: &str, album: &str, duration_ms: u64) -> TrackQuery {
    let album = album.trim();
    let mut title = strip_noise(title);
    let mut artist = clean_channel(artist);

    // Only videos (no album) use the `Artist - Title` convention; on Spotify / YT Music a dash
    // is part of the real title, e.g. `Song - Remastered 2011`.
    if album.is_empty() {
        if let Some((left, right)) = split_dash(&title) {
            let channel = match_key(&artist);
            let right_is_channel = !channel.is_empty()
                && match_key(&right).starts_with(&channel)
                && !match_key(&left).contains(&channel);
            if right_is_channel {
                // `Title - Artist (Live Acoustic)`: the channel already is the artist.
                title = left;
            } else {
                artist = left;
                title = right;
            }
        }
    }

    TrackQuery {
        search_title: strip_featuring(&title),
        title,
        artist,
        album: (!album.is_empty()).then(|| album.to_string()),
        duration_secs: (duration_ms > 0).then(|| ((duration_ms + 500) / 1000) as u32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_metadata_passes_through() {
        // YT Music, from the Tahap 2 log.
        let q = build_query("So Far Away", "Avenged Sevenfold", "Nightmare", 326_826);
        assert_eq!(q.title, "So Far Away");
        assert_eq!(q.artist, "Avenged Sevenfold");
        assert_eq!(q.album.as_deref(), Some("Nightmare"));
        assert_eq!(q.duration_secs, Some(327));
    }

    #[test]
    fn album_track_keeps_dash_in_title() {
        let q = build_query("Hotel California - 2013 Remaster", "Eagles", "Hotel California", 390_000);
        assert_eq!(q.title, "Hotel California - 2013 Remaster");
        assert_eq!(q.artist, "Eagles");
    }

    #[test]
    fn youtube_artist_dash_title() {
        // Opera GX YouTube, from the Tahap 2 log.
        let q = build_query("DENNY CAKNAN - NEGORO ANGIN (Official Music Video)", "DENNY CAKNAN", "", 415_061);
        assert_eq!(q.artist, "DENNY CAKNAN");
        assert_eq!(q.title, "NEGORO ANGIN");
        assert_eq!(q.album, None);
    }

    #[test]
    fn youtube_title_dash_artist_is_swapped() {
        let q = build_query("NEGORO ANGIN - BAGUS FAISAL (Live Acoustic)", "Bagus Faisal", "", 325_541);
        assert_eq!(q.title, "NEGORO ANGIN");
        assert_eq!(q.artist, "Bagus Faisal");
    }

    #[test]
    fn channel_name_is_replaced_by_title_prefix() {
        let q = build_query("NDX AKA - Kimcil Kepolen (Official Music Video)", "NDX A.K.A. Official", "", 0);
        assert_eq!(q.artist, "NDX AKA");
        assert_eq!(q.title, "Kimcil Kepolen");
        assert_eq!(q.duration_secs, None);
    }

    #[test]
    fn topic_and_vevo_channels() {
        assert_eq!(clean_channel("Olivia Rodrigo - Topic"), "Olivia Rodrigo");
        assert_eq!(clean_channel("AvengedSevenfoldVEVO"), "AvengedSevenfold");
        assert_eq!(clean_channel("Dewa 19 Official"), "Dewa 19");
    }

    #[test]
    fn strips_various_noise_groups() {
        assert_eq!(strip_noise("Separuh Aku [Official Lyric Video]"), "Separuh Aku");
        assert_eq!(strip_noise("Kangen 【MV】 (HD)"), "Kangen");
        assert_eq!(strip_noise("Hati-Hati di Jalan | Official Audio"), "Hati-Hati di Jalan");
        assert_eq!(strip_noise("Lagu (Lirik) #shorts #viral"), "Lagu");
        assert_eq!(strip_noise("Pupus (Video Klip Resmi)"), "Pupus");
    }

    #[test]
    fn keeps_meaningful_groups() {
        assert_eq!(strip_noise("Song (Live Acoustic)"), "Song (Live Acoustic)");
        assert_eq!(strip_noise("Song (feat. Someone)"), "Song (feat. Someone)");
        assert_eq!(strip_noise("Song (Remix)"), "Song (Remix)");
    }

    #[test]
    fn unbalanced_brackets_do_not_panic() {
        assert_eq!(strip_noise("Song (Official"), "Song (Official");
    }

    #[test]
    fn featuring_is_removed_for_search() {
        assert_eq!(strip_featuring("Song (feat. Someone)"), "Song");
        assert_eq!(strip_featuring("Song ft. Someone"), "Song");
        assert_eq!(strip_featuring("Feathers"), "Feathers");
        let q = build_query("Song (feat. X)", "A", "Album", 1_000);
        assert_eq!(q.title, "Song (feat. X)");
        assert_eq!(q.search_title, "Song");
    }

    #[test]
    fn title_without_dash_on_youtube_is_kept() {
        let q = build_query("Download & Install YouTube Music App on Windows PC or Laptop Now", "Zimist Coded", "", 132_281);
        assert_eq!(q.artist, "Zimist Coded");
        assert!(q.title.starts_with("Download & Install"));
    }

    #[test]
    fn match_key_ignores_case_and_punctuation() {
        assert_eq!(match_key("NDX A.K.A."), "ndxaka");
        assert_eq!(match_key("Hati-Hati"), "hatihati");
    }
}
