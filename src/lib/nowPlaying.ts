import type { NowPlaying } from "./ipc";
import type { PlaybackAnchor } from "./syncClock";

/**
 * Convert a backend snapshot (wall-clock timestamp) into an anchor on the local
 * `performance.now()` clock, which is monotonic and unaffected by system clock changes.
 */
export function anchorFromNowPlaying(
  np: NowPlaying,
  perfNowMs: number,
  unixNowMs: number,
): PlaybackAnchor {
  // Unknown update time: assume the position is fresh as of now.
  const ageMs = np.positionUpdatedAtMs > 0 ? Math.max(0, unixNowMs - np.positionUpdatedAtMs) : 0;
  return {
    positionMs: np.positionMs,
    atMs: perfNowMs - ageMs,
    playing: np.playing,
    rate: np.rate > 0 ? np.rate : 1,
  };
}

/** `m:ss`, or `h:mm:ss` for long tracks. */
export function formatTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
}
