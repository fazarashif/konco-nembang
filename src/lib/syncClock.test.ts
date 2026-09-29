import { describe, expect, it } from "vitest";
import { activeLineIndex, msUntilNextLine, positionAt } from "./syncClock";

const lines = [
  { timeMs: 1000, text: "a" },
  { timeMs: 2000, text: "b" },
  { timeMs: 2000, text: "b2" },
  { timeMs: 5000, text: "c" },
];

describe("positionAt", () => {
  it("extrapolates while playing", () => {
    expect(positionAt({ positionMs: 21203, atMs: 100, playing: true }, 1100)).toBe(22203);
  });

  it("freezes while paused", () => {
    expect(positionAt({ positionMs: 21203, atMs: 100, playing: false }, 9999)).toBe(21203);
  });

  it("respects playback rate", () => {
    expect(positionAt({ positionMs: 0, atMs: 0, playing: true, rate: 1.5 }, 1000)).toBe(1500);
  });

  it("never goes backwards if the clock is behind the anchor", () => {
    expect(positionAt({ positionMs: 500, atMs: 1000, playing: true }, 900)).toBe(500);
  });
});

describe("activeLineIndex", () => {
  it("returns -1 before the first line", () => {
    expect(activeLineIndex(lines, 999)).toBe(-1);
  });

  it("switches exactly on the line timestamp", () => {
    expect(activeLineIndex(lines, 1000)).toBe(0);
    expect(activeLineIndex(lines, 1999)).toBe(0);
  });

  it("picks the last of several lines sharing a timestamp", () => {
    expect(activeLineIndex(lines, 2000)).toBe(2);
  });

  it("stays on the last line after it starts", () => {
    expect(activeLineIndex(lines, 999_999)).toBe(3);
  });

  it("handles an empty lyric list", () => {
    expect(activeLineIndex([], 1000)).toBe(-1);
  });
});

describe("msUntilNextLine", () => {
  it("measures the gap to the next line", () => {
    expect(msUntilNextLine(lines, 0, 1200)).toBe(800);
  });

  it("counts from before the first line", () => {
    expect(msUntilNextLine(lines, -1, 250)).toBe(750);
  });

  it("returns null after the last line", () => {
    expect(msUntilNextLine(lines, 3, 6000)).toBeNull();
  });
});
