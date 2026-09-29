import { describe, expect, it } from "vitest";
import { comboToAccelerator, formatAccelerator, type KeyCombo } from "./shortcut";

const combo = (code: string, mods: Partial<KeyCombo> = {}): KeyCombo => ({
  code,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...mods,
});

describe("comboToAccelerator", () => {
  it("builds modifier + key accelerators", () => {
    expect(comboToAccelerator(combo("KeyL", { ctrlKey: true, altKey: true }))).toBe("Ctrl+Alt+L");
    expect(comboToAccelerator(combo("Digit0", { ctrlKey: true, altKey: true }))).toBe("Ctrl+Alt+0");
    expect(comboToAccelerator(combo("Equal", { ctrlKey: true, shiftKey: true }))).toBe("Ctrl+Shift+Equal");
    expect(comboToAccelerator(combo("KeyK", { metaKey: true, altKey: true }))).toBe("Alt+Super+K");
  });

  it("waits while only modifiers are held", () => {
    expect(comboToAccelerator(combo("ControlLeft", { ctrlKey: true }))).toBeNull();
    expect(comboToAccelerator(combo("AltRight", { altKey: true, ctrlKey: true }))).toBeNull();
  });

  it("refuses combinations that would break typing", () => {
    expect(comboToAccelerator(combo("KeyA"))).toBeNull();
    expect(comboToAccelerator(combo("KeyA", { shiftKey: true }))).toBeNull();
  });

  it("allows bare function keys", () => {
    expect(comboToAccelerator(combo("F9"))).toBe("F9");
    expect(comboToAccelerator(combo("F13", { shiftKey: true }))).toBe("Shift+F13");
  });
});

describe("formatAccelerator", () => {
  it("renders symbols and spacing", () => {
    expect(formatAccelerator("Ctrl+Alt+Equal")).toBe("Ctrl + Alt + =");
    expect(formatAccelerator("Ctrl+Alt+Minus")).toBe("Ctrl + Alt + -");
    expect(formatAccelerator("Ctrl+Alt+L")).toBe("Ctrl + Alt + L");
    expect(formatAccelerator("control+alt+KeyL")).toBe("Ctrl + Alt + L");
    expect(formatAccelerator("Alt+Super+ArrowUp")).toBe("Alt + Win + ↑");
  });

  it("marks empty shortcuts as unbound", () => {
    expect(formatAccelerator("")).toBe("Not set");
  });
});
