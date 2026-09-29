import { useEffect, useMemo, useState, type ReactNode } from "react";
import { getEditMode, onEditMode, type LyricsPayload, type NowPlaying } from "../lib/ipc";
import { parseLrc, type LyricLine } from "../lib/lrc";
import { plainToLines } from "../lib/plainLyrics";
import { SyncedLyrics } from "./SyncedLyrics";
import { TrackInfo } from "./TrackInfo";
import { useLyrics } from "./useLyrics";
import { useNowPlaying } from "./useNowPlaying";

const STATUS_NOTE: Record<LyricsPayload["status"], string | undefined> = {
  loading: "Mencari lirik…",
  notFound: "Lirik tidak ditemukan",
  instrumental: "♪ Instrumental",
  error: "Gagal memuat lirik",
  synced: undefined,
  plain: undefined,
};

/** Lyrics only count if they belong to the track playing now (answers can arrive late). */
function belongsTo(lyrics: LyricsPayload | null, track: NowPlaying | null): lyrics is LyricsPayload {
  return !!lyrics && !!track && lyrics.title === track.title && lyrics.artist === track.artist;
}

function toLines(lyrics: LyricsPayload, durationMs: number): LyricLine[] {
  if (lyrics.status === "synced") return parseLrc(lyrics.lrc).lines;
  if (lyrics.status === "plain") return plainToLines(lyrics.text, durationMs);
  return [];
}

export function OverlayApp() {
  const { track, anchor } = useNowPlaying();
  const rawLyrics = useLyrics();
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

  let content: ReactNode = null;
  if (track && anchor) {
    content =
      lines.length > 0 ? (
        <SyncedLyrics lines={lines} anchor={anchor} introText={track.title} />
      ) : (
        <TrackInfo track={track} anchor={anchor} note={STATUS_NOTE[lyrics?.status ?? "loading"]} />
      );
  } else if (editMode) {
    // Stay invisible when nothing plays; only hint at it while positioning the overlay.
    content = (
      <div className="lyrics">
        <p className="line dim">Tidak ada lagu yang diputar</p>
      </div>
    );
  }

  const matched = lyrics?.matched;
  return (
    <div className={editMode ? "frame edit" : "frame"} data-tauri-drag-region>
      {editMode && (
        <div className="badge" data-tauri-drag-region>
          MODE EDIT: drag untuk pindah, tarik tepi untuk resize. Ctrl+Alt+L untuk kunci.
          {matched && ` · LRCLIB: ${matched.trackName} — ${matched.artistName}`}
          {lyrics?.status === "plain" && " (tanpa sinkron)"}
        </div>
      )}
      {content}
    </div>
  );
}
