import { describe, expect, it } from "vitest";
import { parseLrc } from "./lrc";

describe("parseLrc", () => {
  it("parses basic timestamped lines", () => {
    const { lines } = parseLrc("[00:01.00]satu\n[00:02.50]dua\n[01:03.25]tiga");
    expect(lines).toEqual([
      { timeMs: 1000, text: "satu" },
      { timeMs: 2500, text: "dua" },
      { timeMs: 63250, text: "tiga" },
    ]);
  });

  it("handles 1, 2 and 3 digit fractions and missing fractions", () => {
    const { lines } = parseLrc("[00:01.5]a\n[00:01.05]b\n[00:01.005]c\n[00:02]d\n[00:03:25]e");
    expect(lines.map((l) => l.timeMs)).toEqual([1005, 1050, 1500, 2000, 3250]);
  });

  it("expands lines with multiple timestamps and sorts them", () => {
    const { lines } = parseLrc("[00:10.00][00:30.00]reff\n[00:20.00]bait");
    expect(lines).toEqual([
      { timeMs: 10000, text: "reff" },
      { timeMs: 20000, text: "bait" },
      { timeMs: 30000, text: "reff" },
    ]);
  });

  it("reads ID tags into meta and ignores unknown lines", () => {
    const { lines, meta } = parseLrc("[ar:Artis]\n[ti:Judul]\n[length: 03:20]\nbukan lirik\n[00:01.00]x");
    expect(meta).toEqual({ ar: "Artis", ti: "Judul", length: "03:20" });
    expect(lines).toHaveLength(1);
  });

  it("applies a positive offset by showing lyrics sooner", () => {
    const { lines, offsetMs } = parseLrc("[offset:+500]\n[00:02.00]x\n[00:00.20]y");
    expect(offsetMs).toBe(500);
    expect(lines.map((l) => l.timeMs)).toEqual([0, 1500]);
  });

  it("applies a negative offset by showing lyrics later", () => {
    const { lines } = parseLrc("[offset:-250]\n[00:01.00]x");
    expect(lines[0].timeMs).toBe(1250);
  });

  it("keeps empty lines as instrumental gaps", () => {
    const { lines } = parseLrc("[00:01.00]x\n[00:05.00]\n[00:09.00]y");
    expect(lines[1]).toEqual({ timeMs: 5000, text: "" });
  });

  it("strips enhanced LRC word timestamps", () => {
    const { lines } = parseLrc("[00:01.00]<00:01.00>kata <00:01.50>demi <00:02.00>kata");
    expect(lines[0].text).toBe("kata demi kata");
  });

  it("handles CRLF line endings and surrounding whitespace", () => {
    const { lines } = parseLrc("[00:01.00]  a  \r\n[00:02.00]b\r\n");
    expect(lines).toEqual([
      { timeMs: 1000, text: "a" },
      { timeMs: 2000, text: "b" },
    ]);
  });

  it("returns no lines for plain text", () => {
    expect(parseLrc("lirik tanpa waktu\nbaris dua").lines).toEqual([]);
  });
});
