/** The parts of a KeyboardEvent needed to build an accelerator. */
export interface KeyCombo {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "ShiftLeft",
  "ShiftRight",
  "MetaLeft",
  "MetaRight",
  "OSLeft",
  "OSRight",
]);

const FUNCTION_KEY = /^F([1-9]|1[0-9]|2[0-4])$/;

/** `KeyL` -> `L`, `Digit0` -> `0`; other codes (Equal, BracketLeft, F5, ...) stay as-is. */
function keyName(code: string): string {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  return code;
}

/**
 * Convert a key press into an accelerator the backend's global-hotkey parser accepts,
 * e.g. `Ctrl+Alt+L`. Returns null while only modifiers are held, or for combinations that
 * would hijack normal typing (a plain key or Shift+key); function keys may stand alone.
 */
export function comboToAccelerator(e: KeyCombo): string | null {
  if (MODIFIER_CODES.has(e.code) || e.code === "" || e.code === "Unidentified") return null;
  const key = keyName(e.code);
  const hasStrongModifier = e.ctrlKey || e.altKey || e.metaKey;
  if (!hasStrongModifier && !FUNCTION_KEY.test(key)) return null;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  parts.push(key);
  return parts.join("+");
}

const DISPLAY_NAMES: Record<string, string> = {
  Equal: "=",
  Minus: "-",
  BracketLeft: "[",
  BracketRight: "]",
  Backquote: "`",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Super: "Win",
  Control: "Ctrl",
};

/** Human-friendly label: `Ctrl+Alt+Equal` -> `Ctrl + Alt + =`. */
export function formatAccelerator(accel: string): string {
  if (!accel.trim()) return "Not set";
  return accel
    .split("+")
    .map((part) => {
      const token = part.trim();
      const normalized = token.charAt(0).toUpperCase() + token.slice(1);
      return DISPLAY_NAMES[normalized] ?? keyName(normalized);
    })
    .join(" + ");
}
