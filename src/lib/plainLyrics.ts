import type { LyricLine } from "./lrc";

/** Used when the track length is unknown. */
const FALLBACK_LINE_MS = 4000;

/**
 * Give unsynced lyrics approximate timings by spreading the lines evenly over the track,
 * so the overlay still scrolls instead of showing a wall of text.
 */
export function plainToLines(text: string, durationMs: number): LyricLine[] {
  const texts = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  if (texts.length === 0) return [];

  const step = durationMs > 0 ? durationMs / texts.length : FALLBACK_LINE_MS;
  return texts.map((line, i) => ({ timeMs: Math.round(i * step), text: line }));
}
