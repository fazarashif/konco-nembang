import { useEffect, useMemo, useState, type CSSProperties, type ReactNode } from "react";
import { DEFAULT_AUTO_HIDE } from "../lib/autoHide";
import { getEditMode, onEditMode, type LyricsPayload, type NowPlaying, type Settings } from "../lib/ipc";
import { parseLrc, type LyricLine } from "../lib/lrc";
import { plainToLines } from "../lib/plainLyrics";
import { formatAccelerator } from "../lib/shortcut";
import { useLyrics } from "../lib/useLyrics";
import { useSettings } from "../lib/useSettings";
import { SyncedLyrics } from "./SyncedLyrics";
import { Toast } from "./Toast";
import { TrackInfo } from "./TrackInfo";
import { useAutoHide } from "./useAutoHide";
import { useNowPlaying } from "./useNowPlaying";

const STATUS_NOTE: Record<LyricsPayload["status"], string | undefined> = {
  loading: "Searching for lyrics…",
  notFound: "No lyrics found",
  instrumental: "♪ Instrumental",
  error: "Couldn't load lyrics",
  synced: undefined,
  plain: undefined,
};

const ALIGN_ITEMS = { left: "flex-start", center: "center", right: "flex-end" } as const;

/** Lyrics only count if they belong to the track playing now (answers can arrive late). */
function belongsTo(lyrics: LyricsPayload | null, track: NowPlaying | null): lyrics is LyricsPayload {
  return !!lyrics && !!track && lyrics.title === track.title && lyrics.artist === track.artist;
}

function toLines(lyrics: LyricsPayload, durationMs: number): LyricLine[] {
  if (lyrics.status === "synced") return parseLrc(lyrics.lrc).lines;
  if (lyrics.status === "plain") return plainToLines(lyrics.text, durationMs);
  return [];
}

/** Appearance settings become CSS custom properties consumed by overlay.css. */
function styleVars(s: Settings | null): CSSProperties {
  if (!s) return {};
  return {
    "--kn-scale": s.textScale,
    "--kn-align": ALIGN_ITEMS[s.align],
    "--kn-text-align": s.align,
    "--kn-text-opacity": s.textOpacity,
    "--kn-bg-opacity": s.backgroundOpacity,
    "--kn-active-color": s.activeColor,
  } as CSSProperties;
}

export function OverlayApp() {
  const { track, anchor } = useNowPlaying();
  const rawLyrics = useLyrics();
  const settings = useSettings();
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

  let content: ReactNode = null;
  if (track && anchor) {
    content =
      lines.length > 0 ? (
        <SyncedLyrics
          lines={lines}
          anchor={anchor}
          offsetMs={offsetMs}
          visibleLines={settings?.visibleLines ?? 3}
          introText={track.title}
        />
      ) : (
        <TrackInfo track={track} anchor={anchor} note={STATUS_NOTE[lyrics?.status ?? "loading"]} />
      );
  } else if (editMode) {
    // Stay invisible when nothing plays; only hint at it while positioning the overlay.
    content = (
      <div className="lyrics">
        <p className="line dim">Nothing is playing</p>
      </div>
    );
  }

  const matched = lyrics?.matched;
  const frameClass = [
    "frame",
    editMode && "edit",
    hidden && "hidden",
    settings && !settings.animations && "no-anim",
    settings && settings.backgroundOpacity > 0 && "with-bg",
  ]
    .filter(Boolean)
    .join(" ");

  const lockKey = settings?.shortcuts.toggleEdit ? formatAccelerator(settings.shortcuts.toggleEdit) : null;
  return (
    <>
      <div className={frameClass} style={styleVars(settings)} data-tauri-drag-region>
        {editMode && (
          <div className="badge" data-tauri-drag-region>
            EDIT MODE: drag to move, drag the edges to resize.
            {lockKey ? ` Press ${lockKey} to lock.` : " Use the tray menu to lock."}
            {matched && ` · LRCLIB: ${matched.trackName} — ${matched.artistName}`}
            {lyrics?.status === "plain" && " (unsynced)"}
          </div>
        )}
        {content}
      </div>
      {/* Outside the frame so it still shows while the lyrics are auto-hidden. */}
      <Toast />
    </>
  );
}
