import { useEffect, useState } from "react";
import type { LyricLine } from "../lib/lrc";
import { activeLineIndex, msUntilNextLine, positionAt, type PlaybackAnchor } from "../lib/syncClock";

// Fire slightly after the boundary so the new line is already active when we recompute.
const BOUNDARY_SLACK_MS = 5;
const MIN_WAIT_MS = 16;

/**
 * Index of the line being sung. Wakes up only at line boundaries (no per-frame polling),
 * so the overlay costs next to nothing while a game is running.
 *
 * `offsetMs` shifts the lyrics: positive shows each line earlier.
 */
export function useActiveLine(
  lines: readonly LyricLine[],
  anchor: PlaybackAnchor,
  offsetMs = 0,
): number {
  const [index, setIndex] = useState(-1);

  useEffect(() => {
    let timer: number | undefined;

    const tick = () => {
      const position = positionAt(anchor, performance.now()) + offsetMs;
      const current = activeLineIndex(lines, position);
      setIndex(current);
      if (!anchor.playing) return;

      const wait = msUntilNextLine(lines, current, position);
      if (wait === null) return;
      const rate = anchor.rate ?? 1;
      timer = window.setTimeout(tick, Math.max(MIN_WAIT_MS, wait / rate + BOUNDARY_SLACK_MS));
    };

    tick();
    return () => window.clearTimeout(timer);
  }, [lines, anchor, offsetMs]);

  return index;
}
