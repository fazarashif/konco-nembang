import { describe, expect, it } from "vitest";
import { plainToLines } from "./plainLyrics";

describe("plainToLines", () => {
  it("spreads lines evenly over the track and drops blank lines", () => {
    expect(plainToLines("a\n\nb\r\nc\n", 30_000)).toEqual([
      { timeMs: 0, text: "a" },
      { timeMs: 10_000, text: "b" },
      { timeMs: 20_000, text: "c" },
    ]);
  });

  it("uses a fixed pace when the duration is unknown", () => {
    expect(plainToLines("a\nb", 0).map((l) => l.timeMs)).toEqual([0, 4000]);
  });

  it("returns nothing for empty text", () => {
    expect(plainToLines("  \n ", 10_000)).toEqual([]);
  });
});
