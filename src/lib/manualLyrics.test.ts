import { describe, expect, it } from "vitest";
import type { SearchResult } from "./ipc";
import { lyricsFromFile, lyricsFromResult, previewText, rankResults } from "./manualLyrics";

const result = (overrides: Partial<SearchResult>): SearchResult => ({
  id: 1,
  trackName: "Song",
  artistName: "Artist",
  albumName: null,
  durationSecs: 200,
  instrumental: false,
  syncedLyrics: null,
  plainLyrics: null,
  ...overrides,
});

describe("lyricsFromFile", () => {
  it("treats timestamped text as synced", () => {
    expect(lyricsFromFile("﻿[ar:X]\n[00:01.00]a\n")).toEqual({ status: "synced", lrc: "[ar:X]\n[00:01.00]a" });
  });

  it("treats text without timestamps as plain", () => {
    expect(lyricsFromFile("line one\nline two")).toEqual({ status: "plain", text: "line one\nline two" });
  });

  it("rejects empty files", () => {
    expect(() => lyricsFromFile("  \n")).toThrow("empty");
  });
});

describe("lyricsFromResult", () => {
  it("prefers synced lyrics", () => {
    expect(lyricsFromResult(result({ syncedLyrics: "[00:01.00]a", plainLyrics: "a" }))?.status).toBe("synced");
  });

  it("falls back to plain lyrics", () => {
    expect(lyricsFromResult(result({ plainLyrics: "a" }))).toEqual({ status: "plain", text: "a" });
  });

  it("honors the instrumental flag", () => {
    expect(lyricsFromResult(result({ instrumental: true }))).toEqual({ status: "instrumental" });
  });

  it("returns null when there is nothing to use", () => {
    expect(lyricsFromResult(result({}))).toBeNull();
  });
});

describe("rankResults", () => {
  const synced = "[00:01.00]a";

  it("puts the closest synced match first", () => {
    // Order seen live for a 3:35 track: +20 s, -9 s, +20 s, exact, -9 s.
    const results = [
      result({ id: 1, durationSecs: 236, syncedLyrics: synced }),
      result({ id: 2, durationSecs: 207, syncedLyrics: synced }),
      result({ id: 3, durationSecs: 236, syncedLyrics: synced }),
      result({ id: 4, durationSecs: 216, syncedLyrics: synced }),
      result({ id: 5, durationSecs: 207, syncedLyrics: synced }),
    ];
    expect(rankResults(results, 215_686).map((r) => r.id)).toEqual([4, 2, 5, 1, 3]);
  });

  it("prefers synced within the close window over a slightly closer plain entry", () => {
    const results = [
      result({ id: 1, durationSecs: 200, plainLyrics: "a" }),
      result({ id: 2, durationSecs: 202, syncedLyrics: synced }),
    ];
    expect(rankResults(results, 200_000).map((r) => r.id)).toEqual([2, 1]);
  });

  it("sinks entries without lyrics and keeps order when duration is unknown", () => {
    const results = [result({ id: 1 }), result({ id: 2, plainLyrics: "a" }), result({ id: 3, plainLyrics: "b" })];
    expect(rankResults(results, 0).map((r) => r.id)).toEqual([2, 3, 1]);
  });
});

describe("previewText", () => {
  it("strips timestamps and marks gaps", () => {
    expect(previewText({ status: "synced", lrc: "[00:01.00]a\n[00:02.00]\n[00:03.00]b" })).toBe("a\n♪\nb");
  });
});
