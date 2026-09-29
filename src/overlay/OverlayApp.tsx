import { useEffect, useMemo, useState, type CSSProperties, type ReactNode } from "react";
import { DEFAULT_AUTO_HIDE } from "../lib/autoHide";
import { getEditMode, onEditMode, type LyricsPayload, type NowPlaying, type Settings } from "../lib/ipc";
import { parseLrc, type LyricLine } from "../lib/lrc";
import { plainToLines } from "../lib/plainLyrics";
import { formatAccelerator } from "../lib/shortcut";
import { useLyrics } from "../lib/useLyrics";
import { useNowPlaying } from "../lib/useNowPlaying";
import { useSettings } from "../lib/useSettings";
import { SyncedLyrics } from "./SyncedLyrics";
import { Toast } from "./Toast";
import { TrackInfo } from "./TrackInfo";
import { useAutoHide } from "./useAutoHide";

const STATUS_NOTE: Record<LyricsPayload["status"], string | undefined> = {
  loading: "Searching for lyrics…",
  notFound: "No lyrics found",
  instrumental: "♪ Instrumental",
  error: "Couldn't reach LRCLIB, retrying…",
  synced: undefined,
  plain: undefined,
};

const ALIGN_ITEMS = { left: "flex-start", center: "center", right: "flex-end" } as const;

/** The design sizes the overlay for a 170 px tall window; resizing scales from there. */
const DESIGN_HEIGHT = 170;
const windowScale = () => Math.min(3, Math.max(0.6, window.innerHeight / DESIGN_HEIGHT));

/** Lyrics only count if they belong to the track playing now (answers can arrive late). */
function belongsTo(lyrics: LyricsPayload | null, track: NowPlaying | null): lyrics is LyricsPayload {
  return !!lyrics && !!track && lyrics.title === track.title && lyrics.artist === track.artist;
}

function toLines(lyrics: LyricsPayload, durationMs: number): LyricLine[] {
  if (lyrics.status === "synced") return parseLrc(lyrics.lrc).lines;
  if (lyrics.status === "plain") return plainToLines(lyrics.text, durationMs);
  return [];
}

/**
 * Settings become CSS custom properties on .frame. Preset tokens (kn-tokens.css) supply
 * everything else; a null override leaves the preset's value alone.
 */
function styleVars(s: Settings | null, winK: number): CSSProperties {
  const vars: Record<string, string | number> = { "--kn-win-k": winK };
  if (s) {
    vars["--kn-scale"] = s.textScale;
    vars["--kn-align"] = ALIGN_ITEMS[s.align];
    vars["--kn-text-align"] = s.align;
    if (s.textOpacity !== null) vars["--kn-text-opacity"] = s.textOpacity;
    if (s.backgroundOpacity !== null) vars["--kn-bg-opacity"] = s.backgroundOpacity;
    if (s.activeColor !== null) vars["--kn-active-color"] = s.activeColor;
  }
  return vars as CSSProperties;
}

function useWindowScale(): number {
  const [k, setK] = useState(windowScale);
  useEffect(() => {
    const onResize = () => setK(windowScale());
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);
  return k;
}

export function OverlayApp() {
  const { track, anchor } = useNowPlaying();
  const rawLyrics = useLyrics();
  const settings = useSettings();
  const winK = useWindowScale();
  const [editMode, setEditMode] = useState(false);

  useEffect(() => {
    getEditMode().then(setEditMode);
    const unlisten = onEditMode(setEditMode);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const lyrics = belongsTo(rawLyrics, track) ? rawLyrics : null;
  const durationMs = track?.durationMs ?? 0;
  const lines = useMemo(() => (lyrics ? toLines(lyrics, durationMs) : []), [lyrics, durationMs]);
  const offsetMs = (settings?.globalOffsetMs ?? 0) + (lyrics?.offsetMs ?? 0);
  const preset = settings?.preset ?? "minimal";
  const paused = !!anchor && !anchor.playing;

  const hidden = useAutoHide(
    {
      editMode,
      hasTrack: !!track,
      playing: anchor?.playing ?? false,
      lyricsStatus: lyrics?.status ?? null,
    },
    settings ?? DEFAULT_AUTO_HIDE,
    track ? `${track.sourceApp}\n${track.title}\n${track.artist}` : "",
  );

  const chips: string[] = [];
  if (paused && track) chips.push("⏸ Paused");
  if (lyrics?.status === "plain") chips.push("≈ Approximate timing");

  let content: ReactNode = null;
  if (track && anchor) {
    if (lines.length > 0) {
      content = (
        <SyncedLyrics
          lines={lines}
          anchor={anchor}
          offsetMs={offsetMs}
          visibleLines={settings?.visibleLines ?? 3}
          introText={track.title}
          showProgress={preset === "karaoke" && lyrics?.status === "synced"}
        />
      );
    } else {
      const note = STATUS_NOTE[lyrics?.status ?? "loading"];
      if (note) chips.push(note);
      content = <TrackInfo track={track} anchor={anchor} />;
    }
  } else if (editMode) {
    // Stay invisible when nothing plays; only hint at it while positioning the overlay.
    chips.push("Nothing is playing");
  }

  const matched = lyrics?.matched;
  const frameClass = [
    "frame",
    editMode && "edit",
    hidden && "hidden",
    paused && !editMode && "paused",
    settings && !settings.animations && "no-anim",
    settings?.backgroundOpacity && "with-bg",
  ]
    .filter(Boolean)
    .join(" ");

  const lockKey = settings?.shortcuts.toggleEdit ? formatAccelerator(settings.shortcuts.toggleEdit) : null;
  return (
    <>
      <div className={frameClass} data-kn-preset={preset} style={styleVars(settings, winK)} data-tauri-drag-region>
        {editMode && (
          <>
            <div className="badge" data-tauri-drag-region>
              <span>EDIT MODE</span>
              <span>
                Drag to move, drag the edges to resize.
                {lockKey ? ` ${lockKey} to lock.` : " Lock it from the tray menu."}
              </span>
              {matched && (
                <span>
                  {matched.source === "file" ? "File" : "LRCLIB"}: {matched.trackName}
                  {matched.artistName && ` — ${matched.artistName}`}
                  {lyrics?.status === "plain" && " (unsynced)"}
                </span>
              )}
            </div>
            <i className="kn-handle tl" />
            <i className="kn-handle tr" />
            <i className="kn-handle bl" />
            <i className="kn-handle br" />
          </>
        )}
        {chips.length > 0 && (
          <div className="kn-chip-row">
            {chips.map((chip) => (
              <span key={chip} className="kn-chip">
                {chip}
              </span>
            ))}
          </div>
        )}
        {content}
      </div>
      {/* Outside the frame so it still shows while the lyrics are dimmed or auto-hidden. */}
      <Toast />
    </>
  );
}
