import { getVersion } from "@tauri-apps/api/app";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  adjustTrackOffset,
  openSearch,
  resetOverlayPosition,
  updateSettings,
  type Settings,
  type Shortcuts,
} from "../lib/ipc";
import { useLyrics } from "../lib/useLyrics";
import { useSettings } from "../lib/useSettings";
import { Row, Section, Segmented, ShortcutInput, Slider, Toggle } from "./controls";

const percent = (v: number) => `${Math.round(v * 100)}%`;
const seconds = (ms: number) => `${ms > 0 ? "+" : ""}${(ms / 1000).toFixed(2)} s`;

const SHORTCUT_LABELS: { key: keyof Shortcuts; label: string; hint?: string }[] = [
  { key: "toggleEdit", label: "Move / resize overlay", hint: "Press again to lock it in place" },
  { key: "toggleVisible", label: "Show / hide overlay" },
  { key: "openSettings", label: "Open settings" },
  { key: "offsetEarlier", label: "Show lyrics earlier", hint: "When lyrics lag behind the singing" },
  { key: "offsetLater", label: "Show lyrics later", hint: "When lyrics run ahead of the singing" },
  { key: "offsetReset", label: "Reset track offset" },
];

export function SettingsApp() {
  const saved = useSettings();
  const lyrics = useLyrics();
  const [draft, setDraft] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  // Saves still in flight. While a slider is being dragged, echoes of older values would
  // otherwise snap it back for a moment.
  const pending = useRef(0);

  // Follow the backend (also picks up changes made elsewhere).
  useEffect(() => {
    if (saved && pending.current === 0) setDraft(saved);
  }, [saved]);

  const commit = useCallback(
    (next: Settings) => {
      setDraft(next);
      pending.current += 1;
      updateSettings(next)
        .then((result) => {
          setError(null);
          setWarnings(result.warnings);
        })
        .catch((err) => {
          setError(String(err));
          if (saved) setDraft(saved);
        })
        .finally(() => {
          pending.current -= 1;
        });
    },
    [saved],
  );

  if (!draft) return <main className="settings loading">Loading…</main>;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => commit({ ...draft, [key]: value });
  const setShortcut = (key: keyof Shortcuts, value: string) =>
    commit({ ...draft, shortcuts: { ...draft.shortcuts, [key]: value } });

  return (
    <main className="settings">
      <header>
        <h1>Konco Nembang</h1>
        <p>Settings apply instantly.</p>
      </header>

      {error && <div className="banner error">{error}</div>}
      {warnings.map((w) => (
        <div key={w} className="banner warning">
          {w}
        </div>
      ))}

      <Section title="Appearance">
        <Row label="Lines shown">
          <Segmented
            value={draft.visibleLines}
            options={[
              { value: 1, label: "1" },
              { value: 2, label: "2" },
              { value: 3, label: "3" },
            ]}
            onChange={(v) => set("visibleLines", v)}
          />
        </Row>
        <Row label="Text size" hint="Resizing the overlay also scales the text">
          <Slider
            value={draft.textScale}
            min={0.5}
            max={2.5}
            step={0.05}
            format={percent}
            onChange={(v) => set("textScale", v)}
          />
        </Row>
        <Row label="Alignment">
          <Segmented
            value={draft.align}
            options={[
              { value: "left", label: "Left" },
              { value: "center", label: "Center" },
              { value: "right", label: "Right" },
            ]}
            onChange={(v) => set("align", v)}
          />
        </Row>
        <Row label="Text opacity">
          <Slider
            value={draft.textOpacity}
            min={0.2}
            max={1}
            step={0.05}
            format={percent}
            onChange={(v) => set("textOpacity", v)}
          />
        </Row>
        <Row label="Background" hint="Dark backdrop for busy screens">
          <Slider
            value={draft.backgroundOpacity}
            min={0}
            max={0.9}
            step={0.05}
            format={(v) => (v === 0 ? "Off" : percent(v))}
            onChange={(v) => set("backgroundOpacity", v)}
          />
        </Row>
        <Row label="Active line color">
          <input
            type="color"
            className="color"
            value={draft.activeColor.length === 4 ? expandHex(draft.activeColor) : draft.activeColor}
            onChange={(e) => set("activeColor", e.target.value)}
          />
        </Row>
        <Row label="Animations">
          <Toggle checked={draft.animations} onChange={(v) => set("animations", v)} />
        </Row>
      </Section>

      <Section title="Behavior">
        <Row label="Hide when paused" hint={hideHint(draft.hidePausedAfterMs)}>
          <DelayToggle value={draft.hidePausedAfterMs} fallbackMs={3000} onChange={(v) => set("hidePausedAfterMs", v)} />
        </Row>
        <Row label="Hide when no lyrics" hint={hideHint(draft.hideNoLyricsAfterMs)}>
          <DelayToggle
            value={draft.hideNoLyricsAfterMs}
            fallbackMs={6000}
            onChange={(v) => set("hideNoLyricsAfterMs", v)}
          />
        </Row>
        <Row label="Follow music from">
          <Segmented
            value={draft.mediaSource}
            options={[
              { value: "auto", label: "Any app" },
              { value: "spotify", label: "Spotify" },
              { value: "browser", label: "Browsers" },
            ]}
            onChange={(v) => set("mediaSource", v)}
          />
        </Row>
      </Section>

      <Section title="Sync">
        <Row label="Global offset" hint="Applies to every track. Positive shows lyrics earlier.">
          <Slider
            value={draft.globalOffsetMs}
            min={-5000}
            max={5000}
            step={50}
            format={seconds}
            onChange={(v) => set("globalOffsetMs", v)}
          />
        </Row>
        <Row label="Hotkey step">
          <Segmented
            value={draft.offsetStepMs}
            options={[
              { value: 100, label: "0.1 s" },
              { value: 250, label: "0.25 s" },
              { value: 500, label: "0.5 s" },
            ]}
            onChange={(v) => set("offsetStepMs", v)}
          />
        </Row>
        <Row
          label="This track"
          hint={lyrics ? `${lyrics.title}${lyrics.artist ? ` — ${lyrics.artist}` : ""}` : "Nothing is playing"}
        >
          <div className="offset-track">
            <button type="button" disabled={!lyrics} onClick={() => adjustTrackOffset(-draft.offsetStepMs)}>
              Later
            </button>
            <output>{seconds(lyrics?.offsetMs ?? 0)}</output>
            <button type="button" disabled={!lyrics} onClick={() => adjustTrackOffset(draft.offsetStepMs)}>
              Earlier
            </button>
            <button type="button" disabled={!lyrics || !lyrics.offsetMs} onClick={() => adjustTrackOffset(null)}>
              Reset
            </button>
          </div>
        </Row>
        <Row label="Wrong or missing lyrics?" hint="Pick other lyrics or import an .lrc file">
          <button type="button" onClick={() => openSearch()}>
            Search lyrics…
          </button>
        </Row>
      </Section>

      <Section title="Shortcuts">
        {SHORTCUT_LABELS.map(({ key, label, hint }) => (
          <Row key={key} label={label} hint={hint}>
            <ShortcutInput value={draft.shortcuts[key]} onChange={(v) => setShortcut(key, v)} />
          </Row>
        ))}
      </Section>

      <Section title="General">
        <Row label="Start with Windows">
          <Toggle checked={draft.autostart} onChange={(v) => set("autostart", v)} />
        </Row>
        <Row label="Overlay position" hint="Move it back to the bottom center at the default size">
          <button type="button" onClick={() => resetOverlayPosition()}>
            Reset
          </button>
        </Row>
      </Section>

      <Section title="About">
        <p className="about">
          Konco Nembang {version}
          <br />
          Lyrics from LRCLIB (lrclib.net), a free community lyrics database.
          <br />
          Only the title, artist, album and duration of the playing track are sent to LRCLIB.
        </p>
      </Section>
    </main>
  );
}

function expandHex(short: string): string {
  return "#" + [...short.slice(1)].map((c) => c + c).join("");
}

function hideHint(ms: number | null): string {
  return ms === null ? "Never" : `After ${ms / 1000} s`;
}

function DelayToggle({
  value,
  fallbackMs,
  onChange,
}: {
  value: number | null;
  fallbackMs: number;
  onChange: (ms: number | null) => void;
}) {
  return (
    <div className="delay">
      <Toggle checked={value !== null} onChange={(on) => onChange(on ? fallbackMs : null)} />
      {value !== null && (
        <select value={value} onChange={(e) => onChange(Number(e.target.value))}>
          {[1000, 3000, 6000, 10000, 30000].map((ms) => (
            <option key={ms} value={ms}>
              {ms / 1000} s
            </option>
          ))}
        </select>
      )}
    </div>
  );
}
