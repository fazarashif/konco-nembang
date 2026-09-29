import { describe, expect, it } from "vitest";
import type { NowPlaying } from "./ipc";
import { anchorFromNowPlaying, formatTime } from "./nowPlaying";
import { positionAt } from "./syncClock";

const base: NowPlaying = {
  sourceApp: "Chrome._crx_cinhimbnkkghhklpknlkffjgod",
  title: "the cure",
  artist: "Olivia Rodrigo",
  album: "you seem pretty sad for a girl so in love",
  durationMs: 297_091,
  positionMs: 1_521,
  positionUpdatedAtMs: 1_000_000,
  playing: true,
  rate: 1,
};

describe("anchorFromNowPlaying", () => {
  it("backdates the anchor by the age of the snapshot", () => {
    // Tahap 0 probe: reported 00:01.521, 15.8 s old -> 00:17.322 now.
    const anchor = anchorFromNowPlaying(base, 50_000, 1_015_801);
    expect(anchor.atMs).toBe(50_000 - 15_801);
    expect(positionAt(anchor, 50_000)).toBe(17_322);
  });

  it("treats an unknown update time as fresh", () => {
    const anchor = anchorFromNowPlaying({ ...base, positionUpdatedAtMs: 0 }, 50_000, 1_015_801);
    expect(positionAt(anchor, 50_000)).toBe(1_521);
  });

  it("ignores a timestamp from the future (clock skew)", () => {
    const anchor = anchorFromNowPlaying(base, 50_000, 999_000);
    expect(anchor.atMs).toBe(50_000);
  });

  it("keeps paused tracks frozen", () => {
    const anchor = anchorFromNowPlaying({ ...base, playing: false }, 50_000, 1_015_801);
    expect(positionAt(anchor, 90_000)).toBe(1_521);
  });

  it("falls back to rate 1 for a nonsense rate", () => {
    expect(anchorFromNowPlaying({ ...base, rate: 0 }, 0, 0).rate).toBe(1);
  });
});

describe("formatTime", () => {
  it.each([
    [0, "0:00"],
    [1_521, "0:01"],
    [297_091, "4:57"],
    [3_723_000, "1:02:03"],
    [-50, "0:00"],
  ])("%i ms -> %s", (ms, expected) => {
    expect(formatTime(ms)).toBe(expected);
  });
});
