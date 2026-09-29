import type { LyricLine } from "./lrc";

/**
 * Last known playback position. Windows only reports the position every ~20 s,
 * so the current position is extrapolated from this anchor.
 */
export interface PlaybackAnchor {
  /** Track position (ms) at the moment `atMs` was captured. */
  positionMs: number;
  /** Local clock (`performance.now()`) when `positionMs` was valid. */
  atMs: number;
  playing: boolean;
  /** Playback speed; 1 unless the player says otherwise. */
  rate?: number;
}

export function positionAt(anchor: PlaybackAnchor, nowMs: number): number {
  if (!anchor.playing) return anchor.positionMs;
  const elapsed = Math.max(0, nowMs - anchor.atMs);
  return anchor.positionMs + elapsed * (anchor.rate ?? 1);
}

/** Index of the line being sung at `positionMs`, or -1 before the first line. */
export function activeLineIndex(lines: readonly LyricLine[], positionMs: number): number {
  let lo = 0;
  let hi = lines.length - 1;
  let found = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].timeMs <= positionMs) {
      found = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return found;
}

/** Milliseconds of track time until the line after `index` starts, or null if there is none. */
export function msUntilNextLine(
  lines: readonly LyricLine[],
  index: number,
  positionMs: number,
): number | null {
  const next = lines[index + 1];
  return next ? Math.max(0, next.timeMs - positionMs) : null;
}
