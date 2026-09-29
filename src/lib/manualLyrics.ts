import type { FinalLyrics, MatchInfo, SearchResult } from "./ipc";
import { parseLrc } from "./lrc";

/** Contents of an imported file: timestamped LRC becomes synced lyrics, anything else plain. */
export function lyricsFromFile(text: string): FinalLyrics {
  const trimmed = text.replace(/^﻿/, "").trim();
  if (!trimmed) throw new Error("The file is empty.");
  if (parseLrc(trimmed).lines.length > 0) return { status: "synced", lrc: trimmed };
  return { status: "plain", text: trimmed };
}

export function fileMatchInfo(fileName: string): MatchInfo {
  return { source: "file", id: 0, trackName: fileName, artistName: "", durationSecs: null };
}

/** Best lyrics a search hit offers: synced over plain; instrumental if flagged. */
export function lyricsFromResult(r: SearchResult): FinalLyrics | null {
  if (r.instrumental) return { status: "instrumental" };
  if (r.syncedLyrics) return { status: "synced", lrc: r.syncedLyrics };
  if (r.plainLyrics) return { status: "plain", text: r.plainLyrics };
  return null;
}

export function resultMatchInfo(r: SearchResult): MatchInfo {
  return {
    source: "lrclib",
    id: r.id,
    trackName: r.trackName,
    artistName: r.artistName,
    durationSecs: r.durationSecs,
  };
}

/**
 * Most useful hits first: ones with lyrics, then a duration within 3 s of the playing
 * track, then synced over plain, then the smallest duration gap. Stable otherwise.
 */
export function rankResults(results: readonly SearchResult[], playingMs: number): SearchResult[] {
  const gap = (r: SearchResult) =>
    playingMs > 0 && r.durationSecs !== null ? Math.abs(r.durationSecs - playingMs / 1000) : Infinity;
  const score = (r: SearchResult): number[] => {
    const hasLyrics = lyricsFromResult(r) !== null;
    const g = gap(r);
    return [hasLyrics ? 0 : 1, g <= 3 ? 0 : 1, r.syncedLyrics ? 0 : 1, g];
  };
  return results
    .map((r, i) => ({ r, i, s: score(r) }))
    .sort((a, b) => {
      for (let k = 0; k < a.s.length; k++) {
        if (a.s[k] !== b.s[k]) return a.s[k] - b.s[k];
      }
      return a.i - b.i;
    })
    .map(({ r }) => r);
}

/** Lyrics text for previewing, without LRC timestamps or ID tags. */
export function previewText(lyrics: FinalLyrics): string {
  switch (lyrics.status) {
    case "synced":
      return parseLrc(lyrics.lrc)
        .lines.map((l) => l.text || "♪")
        .join("\n");
    case "plain":
      return lyrics.text;
    case "instrumental":
      return "♪ Instrumental";
    case "notFound":
      return "";
  }
}
