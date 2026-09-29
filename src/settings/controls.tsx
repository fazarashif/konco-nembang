import { useEffect, useRef, useState, type ReactNode } from "react";
import { resumeHotkeys, suspendHotkeys } from "../lib/ipc";
import { comboToAccelerator, formatAccelerator } from "../lib/shortcut";

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="section">
      <h2>{title}</h2>
      <div className="rows">{children}</div>
    </section>
  );
}

export function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="row">
      <div className="row-text">
        <span className="row-label">{label}</span>
        {hint && <span className="row-hint">{hint}</span>}
      </div>
      <div className="row-control">{children}</div>
    </div>
  );
}

interface SliderProps {
  value: number;
  min: number;
  max: number;
  step: number;
  format: (value: number) => string;
  onChange: (value: number) => void;
}

export function Slider({ value, min, max, step, format, onChange }: SliderProps) {
  return (
    <div className="slider">
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
      <output>{format(value)}</output>
    </div>
  );
}

export function Toggle({ checked, onChange }: { checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="toggle">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      <span className="toggle-track" aria-hidden />
    </label>
  );
}

interface SegmentedProps<T extends string | number> {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}

export function Segmented<T extends string | number>({ value, options, onChange }: SegmentedProps<T>) {
  return (
    <div className="segmented" role="radiogroup">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={o.value === value}
          className={o.value === value ? "selected" : ""}
          onClick={() => onChange(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

/**
 * Click, then press the new combination. Esc cancels, Backspace/Delete clears.
 * Global hotkeys are suspended while recording, otherwise Windows would swallow any
 * combination that is currently registered before this page sees it.
 */
export function ShortcutInput({ value, onChange }: { value: string; onChange: (accel: string) => void }) {
  const [recording, setRecording] = useState(false);
  // A ref, so parent re-renders during recording do not resume hotkeys mid-way.
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  useEffect(() => {
    if (!recording) return;
    suspendHotkeys();
    const onChange = (accel: string) => onChangeRef.current(accel);

    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const bare = !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey;
      if (bare && e.code === "Escape") {
        setRecording(false);
      } else if (bare && (e.code === "Backspace" || e.code === "Delete")) {
        onChange("");
        setRecording(false);
      } else {
        const accel = comboToAccelerator(e);
        if (accel) {
          onChange(accel);
          setRecording(false);
        }
      }
    };
    const stop = () => setRecording(false);

    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("blur", stop);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("blur", stop);
      resumeHotkeys();
    };
  }, [recording]);

  return (
    <button
      type="button"
      className={recording ? "shortcut recording" : "shortcut"}
      onClick={() => setRecording((r) => !r)}
    >
      {recording ? "Press keys… (Esc cancel, Backspace clear)" : formatAccelerator(value)}
    </button>
  );
}
