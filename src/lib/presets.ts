import type { Preset } from "./ipc";

/** Values each preset uses when the user has not overridden them (mirror kn-tokens.css). */
export const PRESET_DEFAULTS: Record<Preset, { textOpacity: number; backgroundOpacity: number; activeColor: string }> =
  {
    minimal: { textOpacity: 1, backgroundOpacity: 0, activeColor: "#ffffff" },
    pill: { textOpacity: 1, backgroundOpacity: 0.74, activeColor: "#ffffff" },
    karaoke: { textOpacity: 1, backgroundOpacity: 0, activeColor: "#f8c153" },
    subtle: { textOpacity: 0.8, backgroundOpacity: 0, activeColor: "#ffffff" },
  };

export const PRESET_OPTIONS: { value: Preset; label: string; hint: string }[] = [
  { value: "minimal", label: "Minimal", hint: "Outlined text only. Reads on any background." },
  { value: "pill", label: "Pill", hint: "Dark backdrop behind each line. Best over bright pages." },
  { value: "karaoke", label: "Karaoke", hint: "Brass active line with a progress bar." },
  { value: "subtle", label: "Subtle", hint: "Smaller and softer, for working." },
];
