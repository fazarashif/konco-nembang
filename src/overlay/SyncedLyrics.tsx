import type { LyricLine } from "../lib/lrc";
import { positionAt, type PlaybackAnchor } from "../lib/syncClock";
import { LyricsView, type LineProgress } from "./LyricsView";
import { useActiveLine } from "./useActiveLine";

/** How long the last line counts as "being sung" for the progress bar. */
const LAST_LINE_MS = 5000;

interface SyncedLyricsProps {
  lines: readonly LyricLine[];
  anchor: PlaybackAnchor;
  offsetMs: number;
  visibleLines: 1 | 2 | 3;
  /** Shown before the first lyric line starts. */
  introText: string;
  /** Draw the per-line progress bar (Karaoke preset, synced lyrics only). */
  showProgress: boolean;
}

function lineProgress(
  lines: readonly LyricLine[],
  index: number,
  anchor: PlaybackAnchor,
  offsetMs: number,
): LineProgress | null {
  const line = lines[index];
  if (!line) return null;
  const end = lines[index + 1]?.timeMs ?? line.timeMs + LAST_LINE_MS;
  const rate = anchor.rate ?? 1;
  return {
    durationMs: (end - line.timeMs) / rate,
    elapsedMs: (positionAt(anchor, performance.now()) + offsetMs - line.timeMs) / rate,
    playing: anchor.playing,
  };
}

export function SyncedLyrics({ lines, anchor, offsetMs, visibleLines, introText, showProgress }: SyncedLyricsProps) {
  const activeIndex = useActiveLine(lines, anchor, offsetMs);
  const progress = showProgress ? lineProgress(lines, activeIndex, anchor, offsetMs) : null;
  return (
    <LyricsView
      lines={lines}
      activeIndex={activeIndex}
      visibleLines={visibleLines}
      introText={introText}
      progress={progress}
    />
  );
}
