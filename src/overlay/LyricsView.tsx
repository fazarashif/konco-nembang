import type { LyricLine } from "../lib/lrc";

interface LyricsViewProps {
  lines: readonly LyricLine[];
  activeIndex: number;
  /** 1 = active line only, 2 = active + next, 3 = previous + active + next. */
  visibleLines: 1 | 2 | 3;
}

const GAP_MARK = "♪";

function textAt(lines: readonly LyricLine[], index: number): string {
  const line = lines[index];
  if (!line) return "";
  return line.text || GAP_MARK;
}

export function LyricsView({ lines, activeIndex, visibleLines }: LyricsViewProps) {
  const showPrev = visibleLines === 3;
  const showNext = visibleLines >= 2;

  return (
    <div className="lyrics">
      {showPrev && <p className="line dim">{textAt(lines, activeIndex - 1)}</p>}
      {/* Keyed by index so each new line remounts and plays the enter animation. */}
      <p key={activeIndex} className="line active">
        {activeIndex < 0 ? GAP_MARK : textAt(lines, activeIndex)}
      </p>
      {showNext && <p className="line dim">{textAt(lines, activeIndex + 1)}</p>}
    </div>
  );
}
