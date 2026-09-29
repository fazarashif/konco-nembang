import type { LyricLine } from "../lib/lrc";
import type { PlaybackAnchor } from "../lib/syncClock";
import { LyricsView } from "./LyricsView";
import { useActiveLine } from "./useActiveLine";

interface SyncedLyricsProps {
  lines: readonly LyricLine[];
  anchor: PlaybackAnchor;
  /** Shown before the first lyric line starts. */
  introText: string;
}

export function SyncedLyrics({ lines, anchor, introText }: SyncedLyricsProps) {
  const activeIndex = useActiveLine(lines, anchor);
  return <LyricsView lines={lines} activeIndex={activeIndex} visibleLines={3} introText={introText} />;
}
