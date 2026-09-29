import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Keep in sync with src-tauri/src/overlay.rs and src-tauri/src/media/mod.rs.
export const EVENT_EDIT_MODE = "overlay://edit-mode";
export const EVENT_NOW_PLAYING = "media://now-playing";
export const EVENT_LYRICS = "lyrics://changed";

/** Mirrors `media::NowPlaying` in Rust. */
export interface NowPlaying {
  sourceApp: string;
  title: string;
  artist: string;
  album: string;
  durationMs: number;
  /** Position at `positionUpdatedAtMs`; extrapolate with `anchorFromNowPlaying`. */
  positionMs: number;
  /** Unix epoch ms when `positionMs` was valid; 0 if unknown. */
  positionUpdatedAtMs: number;
  playing: boolean;
  rate: number;
}

/** Mirrors `lyrics::MatchInfo` in Rust. */
export interface MatchInfo {
  source: string;
  id: number;
  trackName: string;
  artistName: string;
  durationSecs: number | null;
}

/** Mirrors `lyrics::Lyrics` in Rust (serde internally tagged on `status`). */
export type Lyrics =
  | { status: "loading" }
  | { status: "synced"; lrc: string }
  | { status: "plain"; text: string }
  | { status: "instrumental" }
  | { status: "notFound" }
  | { status: "error"; message: string };

/** Mirrors `lyrics::LyricsPayload`; `title`/`artist` are the raw metadata it belongs to. */
export type LyricsPayload = Lyrics & {
  title: string;
  artist: string;
  matched: MatchInfo | null;
};

export function getEditMode(): Promise<boolean> {
  return invoke<boolean>("get_edit_mode");
}

export function onEditMode(handler: (edit: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>(EVENT_EDIT_MODE, (event) => handler(event.payload));
}

export function getNowPlaying(): Promise<NowPlaying | null> {
  return invoke<NowPlaying | null>("get_now_playing");
}

export function onNowPlaying(handler: (np: NowPlaying | null) => void): Promise<UnlistenFn> {
  return listen<NowPlaying | null>(EVENT_NOW_PLAYING, (event) => handler(event.payload));
}

export function getLyrics(): Promise<LyricsPayload | null> {
  return invoke<LyricsPayload | null>("get_lyrics");
}

export function onLyrics(handler: (lyrics: LyricsPayload | null) => void): Promise<UnlistenFn> {
  return listen<LyricsPayload | null>(EVENT_LYRICS, (event) => handler(event.payload));
}
