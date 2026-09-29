import type { LyricLine } from "../lib/lrc";
import type { PlaybackAnchor } from "../lib/syncClock";
import { LyricsView } from "./LyricsView";
import { useActiveLine } from "./useActiveLine";

interface SyncedLyricsProps {
  lines: readonly LyricLine[];
  anchor: PlaybackAnchor;
  offsetMs: number;
  visibleLines: 1 | 2 | 3;
  /** Shown before the first lyric line starts. */
  introText: string;
}

export function SyncedLyrics({ lines, anchor, offsetMs, visibleLines, introText }: SyncedLyricsProps) {
  const activeIndex = useActiveLine(lines, anchor, offsetMs);
  return (
    <LyricsView lines={lines} activeIndex={activeIndex} visibleLines={visibleLines} introText={introText} />
  );
}
