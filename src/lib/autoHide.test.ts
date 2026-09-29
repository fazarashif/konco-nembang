import { describe, expect, it } from "vitest";
import { autoHideDelay, DEFAULT_AUTO_HIDE, type OverlayCondition } from "./autoHide";

const playingWithLyrics: OverlayCondition = {
  editMode: false,
  hasTrack: true,
  playing: true,
  lyricsStatus: "synced",
};

describe("autoHideDelay", () => {
  it("stays visible while a track with lyrics plays", () => {
    expect(autoHideDelay(playingWithLyrics, DEFAULT_AUTO_HIDE)).toBeNull();
  });

  it("hides immediately when nothing plays", () => {
    expect(autoHideDelay({ ...playingWithLyrics, hasTrack: false }, DEFAULT_AUTO_HIDE)).toBe(0);
  });

  it("never hides in edit mode", () => {
    const c = { ...playingWithLyrics, editMode: true, hasTrack: false, playing: false };
    expect(autoHideDelay(c, DEFAULT_AUTO_HIDE)).toBeNull();
  });

  it("hides after the pause delay", () => {
    expect(autoHideDelay({ ...playingWithLyrics, playing: false }, DEFAULT_AUTO_HIDE)).toBe(3000);
  });

  it.each(["notFound", "instrumental", "error"] as const)("hides when lyrics are %s", (status) => {
    expect(autoHideDelay({ ...playingWithLyrics, lyricsStatus: status }, DEFAULT_AUTO_HIDE)).toBe(6000);
  });

  it("keeps showing while lyrics are loading or unknown", () => {
    expect(autoHideDelay({ ...playingWithLyrics, lyricsStatus: "loading" }, DEFAULT_AUTO_HIDE)).toBeNull();
    expect(autoHideDelay({ ...playingWithLyrics, lyricsStatus: null }, DEFAULT_AUTO_HIDE)).toBeNull();
  });

  it("uses the shorter delay when paused without lyrics", () => {
    const c = { ...playingWithLyrics, playing: false, lyricsStatus: "notFound" as const };
    expect(autoHideDelay(c, DEFAULT_AUTO_HIDE)).toBe(3000);
  });

  it("respects disabled rules", () => {
    const off = { hidePausedAfterMs: null, hideNoLyricsAfterMs: null };
    const c = { ...playingWithLyrics, playing: false, lyricsStatus: "notFound" as const };
    expect(autoHideDelay(c, off)).toBeNull();
  });
});
