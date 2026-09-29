export interface LyricLine {
  /** Time the line starts, in ms from the beginning of the track (offset tag already applied). */
  timeMs: number;
  /** Line text; empty string marks an instrumental gap. */
  text: string;
}

export interface ParsedLrc {
  lines: LyricLine[];
  /** Value of the `[offset:]` tag in ms (already applied to `lines`). */
  offsetMs: number;
  /** Other ID tags such as `ar`, `ti`, `al`, `length`. */
  meta: Record<string, string>;
}

// [mm:ss], [mm:ss.x], [mm:ss.xx], [mm:ss.xxx], also [mm:ss:xx] seen in some files.
const TIME_TAG = /\[(\d{1,3}):(\d{1,2})(?:[.:](\d{1,3}))?\]/g;
const LEADING_TIME_TAGS = /^(?:\s*\[\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?\])+/;
const ID_TAG = /^\s*\[([a-zA-Z#]+):(.*)\]\s*$/;
// Enhanced LRC word timing, e.g. `<00:12.34>`; the overlay syncs per line only.
const WORD_TAG = /<\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?>/g;

function toMs(min: string, sec: string, frac: string | undefined): number {
  // The fraction is a decimal part of a second: "5" = 500 ms, "05" = 50 ms, "005" = 5 ms.
  const fracMs = frac ? Math.round(Number(`0.${frac}`) * 1000) : 0;
  return Number(min) * 60_000 + Number(sec) * 1000 + fracMs;
}

export function parseLrc(source: string): ParsedLrc {
  const meta: Record<string, string> = {};
  const lines: LyricLine[] = [];

  for (const raw of source.split(/\r?\n/)) {
    const leading = raw.match(LEADING_TIME_TAGS);
    if (!leading) {
      const id = raw.match(ID_TAG);
      if (id) meta[id[1].toLowerCase()] = id[2].trim();
      continue;
    }

    const text = raw.slice(leading[0].length).replace(WORD_TAG, "").trim();
    for (const tag of leading[0].matchAll(TIME_TAG)) {
      lines.push({ timeMs: toMs(tag[1], tag[2], tag[3]), text });
    }
  }

  // Per the LRC spec a positive offset makes lyrics appear sooner.
  const offsetMs = Number.parseInt(meta.offset ?? "0", 10) || 0;
  if (offsetMs !== 0) {
    for (const line of lines) line.timeMs = Math.max(0, line.timeMs - offsetMs);
  }

  // Array.prototype.sort is stable, so lines sharing a timestamp keep file order.
  lines.sort((a, b) => a.timeMs - b.timeMs);
  return { lines, offsetMs, meta };
}
