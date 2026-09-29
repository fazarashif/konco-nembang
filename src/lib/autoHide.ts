import type { Lyrics } from "./ipc";

export interface AutoHideSettings {
  /** Fade out this long after playback pauses; null = never. */
  hidePausedAfterMs: number | null;
  /** Fade out this long after it is clear there are no lyrics to show; null = never. */
  hideNoLyricsAfterMs: number | null;
}

// Defaults until the settings window (Tahap 5) makes these configurable.
export const DEFAULT_AUTO_HIDE: AutoHideSettings = {
  hidePausedAfterMs: 3000,
  hideNoLyricsAfterMs: 6000,
};

export interface OverlayCondition {
  editMode: boolean;
  hasTrack: boolean;
  playing: boolean;
  /** Status of lyrics for the current track, or null while unknown. */
  lyricsStatus: Lyrics["status"] | null;
}

const NO_LYRICS: ReadonlySet<Lyrics["status"]> = new Set(["notFound", "instrumental", "error"]);

/** Milliseconds until the overlay should fade out, or null to keep it visible. */
export function autoHideDelay(c: OverlayCondition, s: AutoHideSettings): number | null {
  // The user is positioning the overlay; it must stay visible.
  if (c.editMode) return null;
  if (!c.hasTrack) return 0;

  const delays: number[] = [];
  if (!c.playing && s.hidePausedAfterMs !== null) delays.push(s.hidePausedAfterMs);
  if (c.lyricsStatus && NO_LYRICS.has(c.lyricsStatus) && s.hideNoLyricsAfterMs !== null) {
    delays.push(s.hideNoLyricsAfterMs);
  }
  return delays.length > 0 ? Math.min(...delays) : null;
}
