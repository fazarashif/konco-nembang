import { useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import type { LyricLine } from "../lib/lrc";

/** Progress through the active line, drawn by a CSS animation (Karaoke preset only). */
export interface LineProgress {
  durationMs: number;
  elapsedMs: number;
  playing: boolean;
}

interface LyricsViewProps {
  lines: readonly LyricLine[];
  activeIndex: number;
  /** 1 = active line only, 2 = active + next, 3 = previous + active + next. */
  visibleLines: 1 | 2 | 3;
  /** Active-line text before the first lyric starts (e.g. the track title). */
  introText?: string;
  progress?: LineProgress | null;
}

const GAP_MARK = "♪";

function textAt(lines: readonly LyricLine[], index: number): string {
  const line = lines[index];
  if (!line) return "";
  return line.text || GAP_MARK;
}

/**
 * Long-line rule from the design: a line that wraps is first shrunk to 85% (never below
 * 28 px); if it still does not fit it wraps to at most two balanced lines. Never truncated.
 */
function ActiveLine({ text }: { text: string }) {
  const span = useRef<HTMLSpanElement>(null);
  const [long, setLong] = useState(false);

  useLayoutEffect(() => {
    // An inline span that wraps reports one client rect per visual line.
    if (span.current && span.current.getClientRects().length > 1) setLong(true);
  }, [text]);

  return (
    <p className={long ? "line active enter long" : "line active enter"}>
      <span ref={span}>{text}</span>
    </p>
  );
}

function ContextLine({ text }: { text: string }) {
  return (
    <p className="line dim">
      <span>{text || " "}</span>
    </p>
  );
}

function ProgressBar({ progress }: { progress: LineProgress }) {
  const style = {
    "--kn-progress-ms": `${Math.max(1, progress.durationMs)}ms`,
    "--kn-progress-delay": `${-Math.max(0, progress.elapsedMs)}ms`,
    "--kn-progress-state": progress.playing ? "running" : "paused",
  } as CSSProperties;
  return (
    <div className="kn-progress" aria-hidden>
      <div>
        <div className="kn-progress-fill" style={style} />
      </div>
    </div>
  );
}

export function LyricsView({ lines, activeIndex, visibleLines, introText, progress }: LyricsViewProps) {
  const showPrev = visibleLines === 3;
  const showNext = visibleLines >= 2;
  const active = activeIndex < 0 ? introText || GAP_MARK : textAt(lines, activeIndex);

  return (
    // Keyed by line so the stack slides in and the active line fades in on every change.
    <div key={activeIndex} className="lyrics enter">
      {showPrev && <ContextLine text={textAt(lines, activeIndex - 1)} />}
      <ActiveLine text={active} />
      {progress && <ProgressBar progress={progress} />}
      {showNext && <ContextLine text={textAt(lines, activeIndex + 1)} />}
    </div>
  );
}
